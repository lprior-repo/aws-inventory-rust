use super::*;

#[tokio::test]
async fn absent_list_properties_still_enriches_with_the_exact_composite_identifier() -> TestResult {
    let (ctx, mut events) = context(3)?;
    let kind = descriptor("AWS::EC2::EIP")?;
    let id = "203.0.113.5|eipalloc-abcd";
    collect_type_with(
        &ctx,
        &kind,
        |_| async {
            Ok(ListResourcesOutput::builder()
                .resource_descriptions(ResourceDescription::builder().identifier(id).build())
                .build())
        },
        |received| async move {
            assert_eq!(received, id);
            Ok(get(description(
                &received,
                json!({"PublicIp":"203.0.113.5","AllocationId":"eipalloc-abcd"}),
            )))
        },
    )
    .await?;
    let collected = drain(&mut events)?;
    assert_eq!(collected.len(), 2);
    assert!(collected.iter().any(|event| matches!(event, CollectionEvent::Page(page) if page.scope.collector == "cloud_control_get" && page.resources.len() == 1)));
    assert!(collected.iter().any(|event| matches!(event, CollectionEvent::Page(page) if page.scope.collector == "cloud_control_list" && page.resources.is_empty() && page.edges.is_empty())));
    Ok(())
}

#[tokio::test]
async fn two_page_continuation_emits_terminal_page_with_every_listed_resource() -> TestResult {
    let (ctx, mut events) = context(2)?;
    let kind = descriptor("AWS::EC2::Instance")?;
    let calls = AtomicUsize::new(0);
    let seen = Mutex::new(Vec::new());
    collect_type_with(
        &ctx,
        &kind,
        |token| {
            if let Ok(mut seen) = seen.lock() {
                seen.push(token.clone());
            }
            calls.fetch_add(1, Ordering::SeqCst);
            async move {
                match token.as_deref() {
                    None => Ok(ListResourcesOutput::builder()
                        .resource_descriptions(description("i-one", json!({"InstanceId":"i-one"})))
                        .next_token("next")
                        .build()),
                    Some("next") => Ok(ListResourcesOutput::builder()
                        .resource_descriptions(description("i-two", json!({"InstanceId":"i-two"})))
                        .build()),
                    Some(other) => Err(ApiError::Permanent(format!(
                        "unexpected continuation token {other}"
                    ))),
                }
            }
        },
        |id| async move { Ok(get(description(&id, json!({"InstanceId": id})))) },
    )
    .await?;
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    let observed_tokens = match seen.lock() {
        Ok(seen) => seen.clone(),
        Err(_) => return Err("token ledger was poisoned".into()),
    };
    assert_eq!(observed_tokens, vec![None, Some("next".to_owned())]);
    let collected = drain(&mut events)?;
    let list_pages: Vec<&inventory_core::CollectionPage> = collected
        .iter()
        .filter_map(|event| match event {
            CollectionEvent::Page(page) if page.scope.collector == "cloud_control_list" => {
                Some(page)
            }
            _ => None,
        })
        .collect();
    assert_eq!(list_pages.len(), 2);
    let first = list_pages.first().ok_or("missing first list page")?;
    let second = list_pages.get(1).ok_or("missing second list page")?;
    assert_eq!(first.page_number, 1);
    assert_eq!(first.next_token.as_deref(), Some("next"));
    assert_eq!(second.page_number, 2);
    assert!(second.next_token.is_none());
    let identifiers: BTreeSet<String> = collected
        .iter()
        .filter_map(|event| match event {
            CollectionEvent::Page(page) if page.scope.collector == "cloud_control_get" => {
                Some(page.resources.iter().filter_map(|resource| {
                    resource
                        .properties
                        .get("resource_identifier")
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                }))
            }
            _ => None,
        })
        .flatten()
        .collect();
    assert_eq!(
        identifiers,
        BTreeSet::from(["i-one".to_owned(), "i-two".to_owned()])
    );
    assert!(
        !collected
            .iter()
            .any(|event| matches!(event, CollectionEvent::Failure(_)))
    );
    Ok(())
}

#[tokio::test]
async fn denied_and_malformed_children_do_not_erase_valid_siblings() -> TestResult {
    let (ctx, mut events) = context(3)?;
    let kind = descriptor("AWS::EC2::Instance")?;
    collect_type_with(
        &ctx,
        &kind,
        |_| async {
            Ok(ListResourcesOutput::builder()
                .resource_descriptions(
                    ResourceDescription::builder()
                        .identifier("i-malformed")
                        .properties("not-json")
                        .build(),
                )
                .resource_descriptions(description(
                    "i-denied",
                    json!({"InstanceId":"i-denied","VpcId":"vpc-one","UserData":"HIDDEN"}),
                ))
                .resource_descriptions(
                    ResourceDescription::builder()
                        .identifier("i-success")
                        .build(),
                )
                .build())
        },
        |id| async move {
            if id == "i-success" {
                Ok(get(description(
                    &id,
                    json!({"InstanceId":id,"InstanceType":"t3.micro"}),
                )))
            } else {
                Err(ApiError::Denied("synthetic denied GetResource".into()))
            }
        },
    )
    .await?;
    let collected = drain(&mut events)?;
    let resources: Vec<&Resource> = collected
        .iter()
        .filter_map(|event| match event {
            CollectionEvent::Page(page) => Some(page.resources.iter()),
            _ => None,
        })
        .flatten()
        .collect();
    assert_eq!(resources.len(), 2);
    assert!(
        resources
            .iter()
            .any(|resource| resource.properties.get("resource_identifier")
                == Some(&json!("i-success"))
                && resource.properties.get("details_available") == Some(&json!(true)))
    );
    assert!(
        resources
            .iter()
            .any(|resource| resource.properties.get("resource_identifier")
                == Some(&json!("i-denied"))
                && resource.properties.get("details_available") == Some(&json!(false)))
    );
    assert!(!serde_json::to_string(&collected)?.contains("HIDDEN"));
    assert_eq!(
        collected
            .iter()
            .filter(|event| matches!(event, CollectionEvent::Failure(_)))
            .count(),
        3
    );
    assert!(
        collected
            .iter()
            .filter_map(|event| match event {
                CollectionEvent::Page(page) if page.scope.collector == "cloud_control_list" =>
                    Some(page),
                _ => None,
            })
            .all(|page| page.resources.is_empty() && page.edges.is_empty())
    );
    let denied_page = collected.iter().position(|event| matches!(event, CollectionEvent::Page(page) if page.resources.iter().any(|resource| resource.properties.get("resource_identifier") == Some(&json!("i-denied"))))).ok_or("missing denied observation")?;
    let denied_scope = match collected.get(denied_page) {
        Some(CollectionEvent::Page(page)) => &page.scope,
        _ => return Err("wrong event".into()),
    };
    assert!(collected.iter().take(denied_page).any(
        |event| matches!(event, CollectionEvent::Failure(failure) if failure.scope == *denied_scope)
    ));
    Ok(())
}

#[tokio::test]
async fn empty_page_with_token_continues_and_page_cap_remains_incomplete() -> TestResult {
    let kind = descriptor("AWS::EC2::Instance")?;
    for (max_pages, expected_failure) in [(2, false), (1, true)] {
        let (ctx, mut events) = context(max_pages)?;
        collect_type_with(
            &ctx,
            &kind,
            |token| async move {
                Ok(if token.is_none() {
                    ListResourcesOutput::builder().next_token("next").build()
                } else {
                    ListResourcesOutput::builder().build()
                })
            },
            |_| async { Err(ApiError::Permanent("unexpected read".into())) },
        )
        .await?;
        let collected = drain(&mut events)?;
        assert!(collected.iter().any(|event| matches!(event, CollectionEvent::Page(page) if page.page_number == 1 && page.next_token.as_deref() == Some("next"))));
        assert_eq!(
            collected
                .iter()
                .any(|event| matches!(event, CollectionEvent::Failure(_))),
            expected_failure
        );
    }
    Ok(())
}

#[tokio::test]
async fn repeated_identifier_across_pages_fails_the_scope_without_replaying_child() -> TestResult {
    let (ctx, mut events) = context(3)?;
    let kind = descriptor("AWS::EC2::Instance")?;
    let calls = AtomicUsize::new(0);
    collect_type_with(
        &ctx,
        &kind,
        |token| async move {
            Ok(ListResourcesOutput::builder()
                .resource_descriptions(ResourceDescription::builder().identifier("i-one").build())
                .set_next_token(token.is_none().then(|| "next".to_owned()))
                .build())
        },
        |id| {
            calls.fetch_add(1, Ordering::SeqCst);
            async move { Ok(get(description(&id, json!({"InstanceId":id})))) }
        },
    )
    .await?;
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let collected = drain(&mut events)?;
    assert!(collected.iter().any(|event| matches!(event, CollectionEvent::Failure(failure) if failure.scope.collector == "cloud_control_list")));
    assert!(!collected.iter().any(|event| matches!(event, CollectionEvent::Page(page) if page.scope.collector == "cloud_control_list" && page.next_token.is_none())));
    Ok(())
}
