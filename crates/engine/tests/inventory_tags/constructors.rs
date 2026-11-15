use super::{InventoryError, StoreError, Tag, TagCoverage, TagObservation, TestResult, state, support};

#[test]
fn tags_constructor_exact_cardinality_and_utf8_bounds_preserve_literal_text() -> TestResult {
    // Given each cardinality edge independently of storage, with reversed input order.
    for count in [0, 1, 100] {
        let tags = (0..count).rev().map(|index| Tag::new(format!("key-{index:03}"), ""))
            .collect::<Result<Vec<_>, _>>()?;
        let snapshot = TagObservation::new(support::resource("one")?, TagCoverage::Complete, tags)?;
        assert_eq!(snapshot.tags().iter().map(|tag| (tag.key().to_owned(), tag.value().to_owned())).collect::<Vec<_>>(),
            (0..count).map(|index| (format!("key-{index:03}"), String::new())).collect::<Vec<_>>());
    }
    let too_many = (0..101).map(|index| Tag::new(format!("key-{index:03}"), "value"))
        .collect::<Result<Vec<_>, _>>()?;
    assert!(matches!(TagObservation::new(support::resource("one")?, TagCoverage::Complete, too_many), Err(InventoryError::TagCount)));
    for second in ["same", "conflicting"] {
        assert!(matches!(TagObservation::new(support::resource("one")?, TagCoverage::Complete,
            vec![Tag::new("duplicate", "same")?, Tag::new("duplicate", second)?]), Err(InventoryError::DuplicateTag)));
    }
    // UTF-8 limits count bytes, not characters, with exact maximum and one byte above.
    for key in ["x".to_owned(), "x".repeat(128), "é".repeat(64)] {
        let tag = Tag::new(key.clone(), "")?;
        assert_eq!((tag.key(), tag.value()), (key.as_str(), ""));
    }
    for key in [String::new(), "x".repeat(129), format!("{}x", "é".repeat(64))] {
        assert!(matches!(Tag::new(key, "value"), Err(InventoryError::TagKey)));
    }
    for value in [String::new(), "x".into(), "x".repeat(256), "é".repeat(128)] {
        let tag = Tag::new("key", value.clone())?;
        assert_eq!((tag.key(), tag.value()), ("key", value.as_str()));
    }
    for value in ["x".repeat(257), format!("{}x", "é".repeat(128)), "contains\0nul".to_owned()] {
        assert!(matches!(Tag::new("key", value), Err(InventoryError::TagValue)));
    }
    // All Unicode Cc control code points are forbidden in keys, not only ASCII NUL.
    for code in (0..=31).chain(127..=159) {
        let control = char::from_u32(code).ok_or("invalid control fixture")?;
        assert!(matches!(Tag::new(format!("before{control}after"), "value"), Err(InventoryError::TagKey)));
    }
    let preserved = Tag::new(" Owner ", "  Alice\nBob\t ")?;
    assert_eq!((preserved.key(), preserved.value()), (" Owner ", "  Alice\nBob\t "));
    assert_ne!(preserved, Tag::new(" owner ", "  Alice\nBob\t ")?);
    Ok(())
}

#[tokio::test]
async fn tag_pages_return_every_literal_pair_with_exact_cursor_and_limit_boundaries() -> TestResult {
    // Given 100 legitimate pairs, including a newline value, written through public commit.
    let (_directory, store, _) = support::fixture().await?;
    let expected = (0..100).map(|index| state::pair(&format!("key-{index:03}"), &format!("line-{index}\nnext")))
        .collect::<Vec<_>>();
    let tags = expected.iter().rev().map(|tag| Tag::new(tag.key.as_str(), tag.value.as_str()))
        .collect::<Result<Vec<_>, _>>()?;
    state::commit_resources(&store, "literal-tags", vec![support::observation("one", "same")?],
        vec![TagObservation::new(support::resource("one")?, TagCoverage::Complete, tags)?], 100).await?;
    let key = support::resource("one")?;
    let mut view = store.begin_projection_snapshot().await?;
    for limit in [0, 1001] {
        assert!(matches!(view.tags_for(&key, None, limit).await,
            Err(StoreError::Invalid("projection page size exceeds bounds"))));
    }
    // When read one pair per page, assert the actual sequence, not only counts/cursors.
    let mut cursor = None;
    for (index, expected_pair) in expected.iter().enumerate() {
        let page = view.tags_for(&key, cursor.clone(), 1).await?;
        assert_eq!(page.tags, vec![expected_pair.clone()]);
        assert_eq!(view.tags_for(&key, cursor, 1).await?, page);
        assert_eq!(page.next.is_some(), index != 99);
        cursor = page.next;
    }
    assert_eq!(cursor, None);
    let all = view.tags_for(&key, None, 1000).await?;
    assert_eq!(all.tags, expected);
    assert_eq!(all.next, None);
    view.finish().await?;
    Ok(())
}
