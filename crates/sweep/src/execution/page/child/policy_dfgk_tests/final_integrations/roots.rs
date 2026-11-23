use super::*;
mod definitions;
use definitions::roots;

struct RootCase {
    service: &'static str,
    operation: &'static str,
    kind: &'static str,
    items: &'static str,
    token: &'static str,
    wire_token: &'static str,
    rows: [Json; 2],
    ids: [String; 2],
}


#[tokio::test]
async fn given_owned_safe_summaries_when_pages_resume_after_empty_and_nonempty_pages_then_every_kind_is_queryable_without_details()
-> TestResult {
    let catalog = catalog()?;
    for case in roots() {
        let plan = fixture::plan(&catalog, case.service, case.operation)?;
        assert_eq!(plan.kind().as_str(), case.kind);
        for empty_first in [false, true] {
            let mut scenario = Scenario::new(&catalog, "safe-integrations-roots").await?;
            scenario.seed(plan).await?;
            for ordinal in 0..2 {
                let lease = scenario
                    .store
                    .claim(&scenario.sweep, 1001, 10000)
                    .await?
                    .ok_or("root continuation absent")?;
                assert_eq!(lease.scope().operation().as_str(), case.operation);
                let (request, wire) = scenario.prepared_request(plan, &lease)?;
                assert_wire_token(
                    &wire,
                    if ordinal == 0 {
                        None
                    } else {
                        Some("secondpage")
                    },
                    case.wire_token,
                )?;
                if wire.method() != http::Method::GET {
                    let body: Json = serde_json::from_slice(wire.body())?;
                    assert_eq!(body.get("filterByState"), None);
                    assert_eq!(body.get("filterByStageArn"), None);
                }
                let row = case
                    .rows
                    .get(usize::try_from(ordinal)?)
                    .ok_or("response row absent")?
                    .clone();
                let rows = if ordinal == 0 && empty_first {
                    Vec::new()
                } else {
                    vec![row]
                };
                let response = page(
                    case.items,
                    rows,
                    if ordinal == 0 {
                        Some("secondpage")
                    } else {
                        None
                    },
                    case.token,
                );
                scenario
                    .apply(&catalog, plan, lease, &request, response)
                    .await?;
                scenario = reopen(scenario).await?;
            }
            assert!(
                scenario
                    .store
                    .claim(&scenario.sweep, 1001, 10000)
                    .await?
                    .is_none(),
                "unapproved detail was scheduled"
            );
            let actual = saved(&scenario, case.kind).await?;
            let expected = if empty_first {
                vec![case.ids.get(1).ok_or("second id absent")?.clone()]
            } else {
                case.ids.to_vec()
            };
            assert_eq!(
                actual.iter().map(|(id, _)| id.clone()).collect::<Vec<_>>(),
                expected,
                "{}",
                case.operation
            );
            for (_, bytes) in actual {
                let text = String::from_utf8_lossy(&bytes);
                for forbidden in [
                    "streamKey",
                    "AppServerPrivateKey",
                    "PrivateKeys",
                    "UpdateSignature",
                    "SigKeyCrc",
                ] {
                    assert!(!text.contains(forbidden));
                }
            }
        }
    }
    Ok(())
}

#[tokio::test]
async fn given_two_named_destinations_sharing_delivery_target_when_notification_is_collected_then_graph_joins_configuration_name()
-> TestResult {
    let catalog = catalog()?;
    let scenario = Scenario::new(&catalog, "notification-destination-name").await?;
    let destinations = fixture::plan(&catalog, "iot-managed-integrations", "ListDestinations")?;
    let notifications = fixture::plan(
        &catalog,
        "iot-managed-integrations",
        "ListNotificationConfigurations",
    )?;
    scenario.seed(destinations).await?;
    scenario.seed(notifications).await?;
    for _ in 0..2 {
        let lease = scenario
            .store
            .claim(&scenario.sweep, 1001, 10000)
            .await?
            .ok_or("configuration lease absent")?;
        let plan = fixture::plan(
            &catalog,
            "iot-managed-integrations",
            lease.scope().operation().as_str(),
        )?;
        let (request, _) = scenario.prepared_request(plan, &lease)?;
        let response = if plan.operation_name() == "ListDestinations" {
            json!({"DestinationList":[{"Name":"first","DeliveryDestinationArn":"arn:aws:kinesis:us-east-1:123456789012:stream/shared"},{"Name":"second","DeliveryDestinationArn":"arn:aws:kinesis:us-east-1:123456789012:stream/shared"}]})
        } else {
            json!({"NotificationConfigurationList":[{"EventType":"DEVICE_STATE","DestinationName":"second"}]})
        };
        scenario
            .apply(&catalog, plan, lease, &request, response)
            .await?;
    }
    let destination_rows = saved(&scenario, destinations.kind().as_str()).await?;
    assert_eq!(
        destination_rows
            .iter()
            .map(|(id, _)| id.as_str())
            .collect::<Vec<_>>(),
        ["first", "second"]
    );
    let edges = scenario.store.projection_edges(None, 100).await?;
    assert_eq!(edges.edges.len(), 1);
    let edge = edges.edges.first().ok_or("destination edge absent")?;
    assert_eq!(edge.relation.as_str(), "DELIVERS_TO");
    assert_eq!(edge.from.id.as_str(), "DEVICE_STATE");
    assert_eq!(edge.to.id.as_str(), "second");
    assert_eq!(edge.to.kind, *destinations.kind());
    Ok(())
}
