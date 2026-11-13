//! Resource filtering, ordering and pagination behavior.

mod support;

use cmdb_view::{MAX_PAGE_LIMIT, PageCursor, PageRequest, ResourceFilter, ResourcePage, ViewError};

fn ids(page: &ResourcePage<'_>) -> Vec<String> {
    page.resources
        .iter()
        .map(|resource| resource.id.clone())
        .collect()
}

fn text_filter(text: &str) -> ResourceFilter {
    ResourceFilter {
        text: Some(text.to_owned()),
        ..ResourceFilter::new()
    }
}

#[test]
fn filters_combine_account_region_kind_and_text() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = support::snapshot()?;
    let filter = ResourceFilter {
        account_id: Some(support::ACCOUNT_B.to_owned()),
        region: Some("eu-west-1".to_owned()),
        kind: Some("aws_s3_bucket".to_owned()),
        text: Some("bucket".to_owned()),
    };
    let page = snapshot.page(&filter, PageRequest::new(0, 10)?)?;
    assert_eq!(ids(&page), vec![support::BUCKET]);
    assert_eq!(page.total_matching, 1);

    let filter = ResourceFilter {
        account_id: None,
        region: None,
        kind: Some("aws_ec2_instance".to_owned()),
        text: Some("WEB".to_owned()),
    };
    let page = snapshot.page(&filter, PageRequest::new(0, 10)?)?;
    assert_eq!(
        ids(&page),
        vec![support::INSTANCE_ONE, support::INSTANCE_TWO]
    );
    Ok(())
}

#[test]
fn text_search_is_ascii_case_insensitive_across_fields() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = support::snapshot()?;
    for needle in ["AWS_EC2", "web", "STORAGE", "eu-west", "role-one"] {
        let page = snapshot.page(&text_filter(needle), PageRequest::new(0, 10)?)?;
        assert!(!page.resources.is_empty(), "no match for {needle}");
    }
    let page = snapshot.page(&text_filter("does-not-exist"), PageRequest::new(0, 10)?)?;
    assert!(page.resources.is_empty());
    assert_eq!(page.total_matching, 0);
    let page = snapshot.page(&text_filter(""), PageRequest::new(0, 10)?)?;
    assert_eq!(page.total_matching, 4);
    Ok(())
}

#[test]
fn paginates_with_offset_and_cursor() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = support::snapshot()?;
    let first = snapshot.page(&ResourceFilter::new(), PageRequest::new(0, 3)?)?;
    assert_eq!(
        ids(&first),
        vec![
            support::INSTANCE_ONE,
            support::INSTANCE_TWO,
            support::BUCKET
        ]
    );
    assert_eq!(first.total_matching, 4);
    let cursor = first.next_cursor.as_ref().ok_or("first cursor")?;
    assert_eq!(cursor.as_str(), "3");

    let second = snapshot.page(&ResourceFilter::new(), PageRequest::after(cursor, 3)?)?;
    assert_eq!(second.offset, 3);
    assert_eq!(ids(&second), vec![support::ROLE]);
    assert_eq!(second.total_matching, 4);
    assert!(second.next_cursor.is_none());
    Ok(())
}

#[test]
fn page_total_counts_all_matches_not_just_the_page() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = support::snapshot()?;
    let filter = ResourceFilter {
        account_id: Some(support::ACCOUNT_B.to_owned()),
        ..ResourceFilter::new()
    };
    let first = snapshot.page(&filter, PageRequest::new(0, 1)?)?;
    assert_eq!(ids(&first), vec![support::BUCKET]);
    assert_eq!(first.total_matching, 2);
    let cursor = first.next_cursor.as_ref().ok_or("cursor")?;
    let second = snapshot.page(&filter, PageRequest::after(cursor, 1)?)?;
    assert_eq!(ids(&second), vec![support::ROLE]);

    let past_end = snapshot.page(&filter, PageRequest::new(10, 2)?)?;
    assert!(past_end.resources.is_empty());
    assert_eq!(past_end.total_matching, 2);
    assert!(past_end.next_cursor.is_none());
    Ok(())
}

#[test]
fn unknown_resource_lookup_is_none() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = support::snapshot()?;
    assert!(snapshot.resource(support::DANGLING).is_none());
    assert!(snapshot.resource(support::INSTANCE_ONE).is_some());
    Ok(())
}

#[test]
fn rejects_invalid_page_limit_and_cursor() -> Result<(), Box<dyn std::error::Error>> {
    assert!(matches!(
        PageRequest::new(0, 0),
        Err(ViewError::InvalidPageLimit)
    ));
    assert!(matches!(
        PageRequest::new(0, usize::MAX),
        Err(ViewError::InvalidPageLimit)
    ));
    assert!(PageRequest::new(0, MAX_PAGE_LIMIT).is_ok());
    assert!(matches!(
        PageCursor::parse("not-a-number"),
        Err(ViewError::InvalidCursor)
    ));
    assert!(matches!(
        PageCursor::parse(""),
        Err(ViewError::InvalidCursor)
    ));
    assert_eq!(PageCursor::parse("12")?.offset()?, 12);
    Ok(())
}
