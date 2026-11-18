use super::*;

fn identifier(prefix: &str, suffix: &str) -> String {
    format!("{prefix}-{}", suffix.repeat(32))
}
fn local(prefix: &str, second: bool) -> String {
    identifier(prefix, if second { "2" } else { "1" })
}
fn farm_ids() -> [String; 2] {
    [identifier("farm", "a"), identifier("farm", "b")]
}

#[tokio::test]
async fn given_colliding_local_ids_in_two_farms_when_all_safe_deadline_child_pages_run_then_scoped_nodes_and_farm_edges_persist()
-> TestResult {
    let mut nodes = Nodes::new();
    let mut edges = Edges::new();
    for farm in farm_ids() {
        node(&mut nodes, "AWS::Deadline::Farm", &farm);
        for second in [false, true] {
            for (kind, prefix) in [
                ("AWS::Deadline::Fleet", "fleet"),
                ("AWS::Deadline::Queue", "queue"),
                ("AWS::Deadline::Limit", "limit"),
                ("AWS::Deadline::Budget", "budget"),
                ("AWS::Deadline::StorageProfile", "sp"),
            ] {
                let id = key(&[&farm, &local(prefix, second)]);
                node(&mut nodes, kind, &id);
                edge(
                    &mut edges,
                    kind,
                    &id,
                    "AWS::Deadline::Farm",
                    &farm,
                    "IN_FARM",
                );
            }
            for (kind, prefix) in [
                ("AWS::Deadline::QueueFleetAssociation", "fleet"),
                ("AWS::Deadline::QueueLimitAssociation", "limit"),
            ] {
                let id = key(&[&farm, &local("queue", second), &local(prefix, second)]);
                node(&mut nodes, kind, &id);
                edge(
                    &mut edges,
                    kind,
                    &id,
                    "AWS::Deadline::Farm",
                    &farm,
                    "IN_FARM",
                );
            }
            for parent_page in [false, true] {
                for (kind, parent_prefix, prefix) in [
                    ("AWS::Deadline::QueueEnvironment", "queue", "queueenv"),
                    ("AWS::ApiOnly::DeadlineVolume", "fleet", "volume"),
                ] {
                    let id = key(&[
                        &farm,
                        &local(parent_prefix, parent_page),
                        &local(prefix, second),
                    ]);
                    node(&mut nodes, kind, &id);
                    edge(
                        &mut edges,
                        kind,
                        &id,
                        "AWS::Deadline::Farm",
                        &farm,
                        "IN_FARM",
                    );
                }
            }
        }
    }
    run("deadline", &["ListFarms"], 46, response, nodes, edges).await
}

fn response(operation: &str, request: &Record, second: bool) -> TestResult<(Json, Option<String>)> {
    if operation == "ListFarms" {
        let farm = identifier("farm", if second { "b" } else { "a" });
        let body = json!({"farms":[{"farmId":farm,"displayName":"farm","createdAt":"2026-01-01T00:00:00Z","createdBy":"owner"}]});
        return Ok((
            paginated(body, second, "nextToken")?,
            Some("/2023-10-12/farms".to_owned()),
        ));
    }
    let farm = string(request, "farmId")?;
    assert!(farm_ids().iter().any(|expected| expected == farm));
    let base = format!("/2023-10-12/farms/{farm}");
    if operation == "ListFleets" {
        // A response echo is not binding authority: descendants must inherit the actual farm request.
        let fleets = vec![json!({
            "farmId":identifier("farm","c"),"fleetId":local("fleet",second),"displayName":"fleet","status":"ACTIVE","workerCount":0,"minWorkerCount":0,"maxWorkerCount":1,
            "configuration":{"customerManaged":{"mode":"NO_SCALING","workerCapabilities":{"vCpuCount":{"min":1},"memoryMiB":{"min":1024},"osFamily":"linux","cpuArchitectureType":"x86_64"}}},"createdAt":"2026-01-01T00:00:00Z","createdBy":"owner"
        })];
        return Ok((
            paginated(json!({"fleets":fleets}), second, "nextToken")?,
            Some(format!("{base}/fleets")),
        ));
    }
    if operation == "ListQueues" {
        let queues = vec![
            json!({"farmId":identifier("farm","c"),"queueId":local("queue",second),"displayName":"queue","status":"IDLE","defaultBudgetAction":"NONE","createdAt":"2026-01-01T00:00:00Z","createdBy":"owner"}),
        ];
        return Ok((
            paginated(json!({"queues":queues}), second, "nextToken")?,
            Some(format!("{base}/queues")),
        ));
    }
    let (body, suffix) = match operation {
        "ListLimits" => (
            json!({"limits":[{"farmId":farm,"limitId":local("limit",second),"currentCount":0,"maxCount":1,"createdAt":"2026-01-01T00:00:00Z","createdBy":"owner","displayName":"limit","amountRequirementName":"amount.limit"}]}),
            "limits".to_owned(),
        ),
        "ListBudgets" => (
            json!({"budgets":[{"budgetId":local("budget",second),"usageTrackingResource":{"queueId":local("queue",second)},"status":"ACTIVE","displayName":"budget","approximateDollarLimit":100,"usages":{"approximateDollarUsage":0},"createdAt":"2026-01-01T00:00:00Z","createdBy":"owner"}]}),
            "budgets".to_owned(),
        ),
        "ListStorageProfiles" => (
            json!({"storageProfiles":[{"storageProfileId":local("sp",second),"displayName":"profile","osFamily":"linux"}]}),
            "storage-profiles".to_owned(),
        ),
        "ListQueueFleetAssociations" => (
            json!({"queueFleetAssociations":[{"queueId":local("queue",second),"fleetId":local("fleet",second),"status":"ACTIVE","createdAt":"2026-01-01T00:00:00Z","createdBy":"owner"}]}),
            "queue-fleet-associations".to_owned(),
        ),
        "ListQueueLimitAssociations" => (
            json!({"queueLimitAssociations":[{"queueId":local("queue",second),"limitId":local("limit",second),"status":"ACTIVE","createdAt":"2026-01-01T00:00:00Z","createdBy":"owner"}]}),
            "queue-limit-associations".to_owned(),
        ),
        "ListQueueEnvironments" => {
            let queue = string(request, "queueId")?;
            assert!(
                [local("queue", false), local("queue", true)]
                    .iter()
                    .any(|expected| expected == queue)
            );
            (
                json!({"environments":[{"queueEnvironmentId":local("queueenv",second),"name":"environment summary without script","priority":1}]}),
                format!("queues/{queue}/environments"),
            )
        }
        "ListVolumes" => {
            let fleet = string(request, "fleetId")?;
            assert!(
                [local("fleet", false), local("fleet", true)]
                    .iter()
                    .any(|expected| expected == fleet)
            );
            (
                json!({"volumes":[{"farmId":farm,"fleetId":fleet,"volumeId":local("volume",second),"state":"AVAILABLE","sizeGiB":1,"availabilityZoneId":"use1-az1"}]}),
                format!("fleets/{fleet}/volumes"),
            )
        }
        _ => return Err(format!("unexpected Deadline operation {operation}").into()),
    };
    Ok((
        paginated(body, second, "nextToken")?,
        Some(format!("{base}/{suffix}")),
    ))
}
