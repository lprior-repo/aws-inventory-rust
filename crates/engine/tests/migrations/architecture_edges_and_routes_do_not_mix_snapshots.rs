//! Behavior: architecture_edges_and_routes_do_not_mix_snapshots.
use super::*;

#[tokio::test]
pub(super) async fn architecture_edges_and_routes_do_not_mix_snapshots() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("older", 1).await?;
    let older_route = fixture(
        "rtb",
        "aws_vpc_route_table",
        json!({"route_table_id":"rtb","vpc_id":"vpc",
        "routes":[{"destination_cidr_block":"0.0.0.0/0","gateway_id":"igw","state":"active","origin":"CreateRoute"},
                  {"destination_cidr_block":"10.0.0.0/16","gateway_id":"local","state":"active"}]}),
        2,
    );
    store
        .apply_page(
            "older",
            page(
                vec![
                    older_route,
                    fixture(
                        "igw",
                        "aws_vpc_internet_gateway",
                        json!({"internet_gateway_id":"igw"}),
                        2,
                    ),
                ],
                vec![Edge {
                    from_id: "rtb".into(),
                    to_id: "igw".into(),
                    relation: "ROUTES_TO".into(),
                    source: Source::Direct,
                    observed_at: 2,
                    properties: json!({"evidence":"route configuration"}),
                }],
            ),
        )
        .await?;
    store.finish_scan("older", ScanStatus::Complete, 3).await?;
    store.begin_scan("newer", 4).await?;
    let newer_route = fixture(
        "rtb",
        "aws_vpc_route_table",
        json!({"route_table_id":"rtb","vpc_id":"vpc",
        "routes":[{"destination_cidr_block":"0.0.0.0/0","gateway_id":"igw","state":"blackhole"}]}),
        5,
    );
    store
        .apply_page(
            "newer",
            page(
                vec![newer_route],
                vec![Edge {
                    from_id: "rtb".into(),
                    to_id: "igw".into(),
                    relation: "ROUTES_TO".into(),
                    source: Source::Direct,
                    observed_at: 5,
                    properties: json!({}),
                }],
            ),
        )
        .await?;
    store.finish_scan("newer", ScanStatus::Partial, 6).await?;
    assert_eq!(
        store
            .query("SELECT scan_id,state FROM aws_vpc_route", 10)
            .await?,
        json!([{"scan_id":"newer","state":"blackhole"}])
    );
    assert_eq!(
        store
            .query(
                "SELECT count(*) AS total FROM scan_vpc_routes WHERE scan_id='older'",
                1
            )
            .await?,
        json!([{"total":2}])
    );
    assert_eq!(
        store
            .query(
                "SELECT scan_id,target_observed FROM scan_architecture_edges ORDER BY scan_id",
                10
            )
            .await?,
        json!([{"scan_id":"newer","target_observed":0},{"scan_id":"older","target_observed":1}])
    );
    store.close().await;
    Ok(())
}
