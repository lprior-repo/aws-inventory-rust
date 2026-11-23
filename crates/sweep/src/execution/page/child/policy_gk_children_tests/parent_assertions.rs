use super::*;

struct Producer {
    root: &'static str,
    operation: &'static str,
    input: &'static str,
    segment: &'static str,
}

#[derive(Clone, Copy, Debug)]
enum Duplicate {
    Equal,
    Absent,
    Foreign,
    Null,
    WrongType,
}

fn rows(
    service: &str,
    segment: &str,
) -> (
    Json,
    Json,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
) {
    if service == "identitystore" {
        (
            json!({"IdentityStoreId":"d-1111111111","IdentityStoreArn":"arn:aws:identitystore::123456789012:identitystore/d-1111111111"}),
            json!({"GroupArn":"arn:aws:identitystore::123456789012:group/group-a","GroupId":"group-a","IdentityStoreId":"d-1111111111"}),
            "IdentityStores",
            "Groups",
            "IdentityStoreId",
            "d-1111111111",
        )
    } else {
        let arn =
            format!("arn:aws:greengrass:us-east-1:123456789012:/greengrass/{segment}/parent-a");
        (
            json!({"Id":"parent-a","Arn":arn}),
            json!({"Arn":format!("{arn}/versions/v1"),"Id":"parent-a","Version":"v1"}),
            if segment == "groups" {
                "Groups"
            } else {
                "Definitions"
            },
            "Versions",
            "Id",
            "parent-a",
        )
    }
}

async fn exercise_assertion(service: &str, date: &str, producer: Producer) -> TestResult {
    let compiled = compile(service, date, &[producer.root, producer.operation])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let root = catalog
        .lookup(service, producer.root)
        .ok_or("root plan absent")?;
    let child = catalog
        .lookup(service, producer.operation)
        .ok_or("child plan absent")?;
    for duplicate in [
        Duplicate::Equal,
        Duplicate::Absent,
        Duplicate::Foreign,
        Duplicate::Null,
        Duplicate::WrongType,
    ] {
        // Given a real durable parent observation and its compiled, request-bound child work.
        let scenario = Scenario::new(&catalog).await?;
        let (parent, mut row, root_items, items, selected, owner) = rows(service, producer.segment);
        scenario.seed(root).await?;
        let lease = scenario
            .store
            .claim(&scenario.sweep, 1000, 10000)
            .await?
            .ok_or("root lease absent")?;
        let (request, _) = scenario.request(root, &lease)?;
        scenario
            .apply(
                &catalog,
                root,
                lease,
                &request,
                json!({(root_items):[parent]}),
                1,
            )
            .await?;
        let lease = scenario
            .store
            .claim(&scenario.sweep, 1000, 10000)
            .await?
            .ok_or("bound child absent")?;
        assert_eq!(lease.scope().operation().as_str(), producer.operation);
        let (request, wire) = scenario.request(child, &lease)?;
        let field = compiled
            .fields
            .id(producer.input)
            .ok_or("input field absent")?;
        assert_eq!(
            record::path_value(request.root(), &[field])?
                .map(|value| value.to_value())
                .transpose()?,
            Some(Value::Text(owner.to_owned()))
        );
        assert!(
            wire.contains(owner),
            "native child wire omitted bound owner: {wire}"
        );
        // The first row is valid: rejecting only the offending second row would leak a partial page.
        let first = row.clone();
        let object = row.as_object_mut().ok_or("summary is not an object")?;
        if service == "identitystore" {
            object.insert(
                "GroupArn".to_owned(),
                json!("arn:aws:identitystore::123456789012:group/group-b"),
            );
            object.insert("GroupId".to_owned(), json!("group-b"));
        } else {
            object.insert(
                "Arn".to_owned(),
                json!(format!(
                    "arn:aws:greengrass:us-east-1:123456789012:/greengrass/{}/parent-a/versions/v2",
                    producer.segment
                )),
            );
            object.insert("Version".to_owned(), json!("v2"));
        }
        match duplicate {
            Duplicate::Equal => {}
            Duplicate::Absent => {
                object.remove(selected);
            }
            Duplicate::Foreign => {
                object.insert(selected.to_owned(), json!("foreign-parent"));
            }
            Duplicate::Null => {
                object.insert(selected.to_owned(), Json::Null);
            }
            Duplicate::WrongType => {
                object.insert(selected.to_owned(), json!(7));
            }
        }
        // When the pinned native response passes through the real row gate and page commit.
        let result = scenario
            .apply(
                &catalog,
                child,
                lease,
                &request,
                json!({(items):[first,row]}),
                0,
            )
            .await;
        let accepted = matches!(duplicate, Duplicate::Equal | Duplicate::Absent);
        if accepted {
            result?;
        } else {
            assert!(
                result.is_err(),
                "{service}.{} accepted {duplicate:?}",
                producer.operation
            );
        }
        // Then equality and genuine absence persist both rows/edges; inconsistency commits neither.
        let reader = scenario.reader().await?;
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM resource_facets WHERE kind=?")
            .bind(child.kind().as_str())
            .fetch_one(&reader)
            .await?;
        assert_eq!(
            count,
            if accepted { 2 } else { 0 },
            "{service}.{} {duplicate:?}",
            producer.operation
        );
        let edges: Vec<(String,)> =
            sqlx::query_as("SELECT to_id FROM edges WHERE from_kind=? ORDER BY from_id")
                .bind(child.kind().as_str())
                .fetch_all(&reader)
                .await?;
        assert_eq!(edges.len(), if accepted { 2 } else { 0 });
        let parent_id = if service == "identitystore" {
            "arn:aws:identitystore::123456789012:identitystore/d-1111111111".to_owned()
        } else {
            format!(
                "arn:aws:greengrass:us-east-1:123456789012:/greengrass/{}/parent-a",
                producer.segment
            )
        };
        assert!(edges.iter().all(|(id,)| id == &parent_id));
        let parents: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM resource_facets WHERE kind=?")
            .bind(root.kind().as_str())
            .fetch_one(&reader)
            .await?;
        assert_eq!(
            parents, 1,
            "rejecting a child page must not erase its durable parent"
        );
        reader.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn given_store_bound_groups_when_duplicate_store_is_equal_absent_or_hostile_then_native_pages_commit_atomically()
-> TestResult {
    exercise_assertion(
        "identitystore",
        "2020-06-15",
        Producer {
            root: "ListIdentityStores",
            operation: "ListGroups",
            input: "IdentityStoreId",
            segment: "",
        },
    )
    .await
}

#[tokio::test]
async fn given_each_greengrass_parent_when_duplicate_id_is_equal_absent_or_hostile_then_all_version_producers_enforce_atomic_pages()
-> TestResult {
    for producer in [
        Producer {
            root: "ListConnectorDefinitions",
            operation: "ListConnectorDefinitionVersions",
            input: "ConnectorDefinitionId",
            segment: "definition/connectors",
        },
        Producer {
            root: "ListCoreDefinitions",
            operation: "ListCoreDefinitionVersions",
            input: "CoreDefinitionId",
            segment: "definition/cores",
        },
        Producer {
            root: "ListDeviceDefinitions",
            operation: "ListDeviceDefinitionVersions",
            input: "DeviceDefinitionId",
            segment: "definition/devices",
        },
        Producer {
            root: "ListFunctionDefinitions",
            operation: "ListFunctionDefinitionVersions",
            input: "FunctionDefinitionId",
            segment: "definition/functions",
        },
        Producer {
            root: "ListLoggerDefinitions",
            operation: "ListLoggerDefinitionVersions",
            input: "LoggerDefinitionId",
            segment: "definition/loggers",
        },
        Producer {
            root: "ListResourceDefinitions",
            operation: "ListResourceDefinitionVersions",
            input: "ResourceDefinitionId",
            segment: "definition/resources",
        },
        Producer {
            root: "ListSubscriptionDefinitions",
            operation: "ListSubscriptionDefinitionVersions",
            input: "SubscriptionDefinitionId",
            segment: "definition/subscriptions",
        },
        Producer {
            root: "ListGroups",
            operation: "ListGroupVersions",
            input: "GroupId",
            segment: "groups",
        },
    ] {
        exercise_assertion("greengrass", "2017-06-07", producer).await?;
    }
    Ok(())
}
