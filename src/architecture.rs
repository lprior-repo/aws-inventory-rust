//! Bounded topology summaries from a finalized inventory snapshot.
use anyhow::{Context, Result};
use futures_util::{StreamExt, TryStreamExt, future::BoxFuture, stream};
use inventory_core::ScanStatus;
use inventory_engine::store::Store;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::PathBuf;

const MAX_PANEL_ROWS: usize = 1_000;
type PanelResult = (&'static str, Value);
type PanelFuture<'a> = BoxFuture<'a, Result<PanelResult>>;
const PANELS: [(&str, &str); 8] = [
    ("resource_counts", "SELECT kind,source,count(*) AS resources FROM scan_resources
        WHERE scan_id={scan} GROUP BY kind,source ORDER BY kind,source"),
    ("coverage", "SELECT source,coverage_status,count(*) AS scopes FROM scan_coverage
        WHERE scan_id={scan} GROUP BY source,coverage_status ORDER BY source,coverage_status"),
    ("relationships", "SELECT relation,count(*) AS relationships,
        sum(CASE WHEN source_observed=0 THEN 1 ELSE 0 END) AS unobserved_sources,
        sum(CASE WHEN target_observed=0 THEN 1 ELSE 0 END) AS unobserved_targets
        FROM scan_architecture_edges WHERE scan_id={scan} GROUP BY relation ORDER BY relation"),
    ("internet_facing_load_balancers", "SELECT lb.id AS load_balancer_arn,lb.account_id,lb.region,
        json_extract(lb.properties,'$.dns_name') AS dns_name,
        json_extract(lb.properties,'$.type') AS load_balancer_type,
        json_extract(lb.properties,'$.state') AS state,
        listener.id AS listener_arn,json_extract(listener.properties,'$.port') AS listener_port,
        json_extract(listener.properties,'$.protocol') AS listener_protocol
        FROM scan_resources lb LEFT JOIN scan_resources listener
        ON listener.scan_id=lb.scan_id AND listener.kind='aws_elbv2_listener'
        AND json_extract(listener.properties,'$.load_balancer_arn')=lb.id
        WHERE lb.scan_id={scan} AND lb.kind='aws_elbv2_load_balancer'
        AND json_extract(lb.properties,'$.scheme')='internet-facing'
        ORDER BY lb.id,listener.id"),
    ("routes", "SELECT route_table_id,vpc_id,account_id,region,destination_cidr_block,
        destination_ipv6_cidr_block,destination_prefix_list_id,gateway_id,nat_gateway_id,
        transit_gateway_id,vpc_peering_connection_id,network_interface_id,instance_id,
        instance_owner_id,vpc_endpoint_id,egress_only_internet_gateway_id,local_gateway_id,
        carrier_gateway_id,core_network_arn,odb_network_arn,ip_address,state,origin,route_json
        FROM scan_vpc_routes WHERE scan_id={scan} ORDER BY route_table_arn,route_index"),
    ("target_health", "SELECT json_extract(properties,'$.target_group_arn') AS target_group_arn,
        json_extract(properties,'$.target_health_state') AS target_health_state,count(*) AS memberships
        FROM scan_resources WHERE scan_id={scan} AND kind='aws_elbv2_target_health'
        GROUP BY target_group_arn,target_health_state ORDER BY target_group_arn,target_health_state"),
    ("unresolved_references", "SELECT from_id,from_kind,relation,to_id,to_kind,source_observed,target_observed,
        json_extract(properties,'$.owner_resolution') AS owner_resolution
        FROM scan_architecture_edges WHERE scan_id={scan} AND (source_observed=0 OR target_observed=0)
        ORDER BY from_id,relation,to_id"),
    ("failures", "SELECT category,count(*) AS failures FROM collection_failures
        WHERE scan_id={scan} GROUP BY category ORDER BY category"),
];

#[tracing::instrument(skip_all)]
pub async fn report(path: PathBuf, scan_id: String, limit: usize) -> Result<Value> {
    anyhow::ensure!(
        (1..=MAX_PANEL_ROWS).contains(&limit),
        "architecture row limit must be 1..=1000"
    );
    super::require_database(&path)?;
    let store = Store::open(path).await?;
    let result = report_open(&store, &scan_id, limit).await;
    store.close().await;
    result
}

#[tracing::instrument(skip_all)]
async fn report_open(store: &Store, scan_id: &str, limit: usize) -> Result<Value> {
    let status = store.scan_status(scan_id).await?;
    anyhow::ensure!(
        status != ScanStatus::Running,
        "architecture report requires a finalized scan"
    );
    let summary = store.summary(scan_id).await?;
    let escaped = format!("'{}'", scan_id.replace('\'', "''"));
    // Eight owned futures bound allocation and avoid the compiler's borrowed
    // stream-closure lifetime limitation at the CLI's Send future boundary.
    let work: Vec<PanelFuture<'_>> = PANELS
        .into_iter()
        .map(|(name, sql)| {
            let work: PanelFuture<'_> = Box::pin(panel(store, &escaped, name, sql, limit));
            work
        })
        .collect();
    let panels: BTreeMap<&str, Value> = stream::iter(work).buffered(1).try_collect().await?;
    Ok(
        json!({"status":status.as_str(),"scan_id":scan_id,"summary":summary,
        "row_limit_per_panel":limit,"panels":panels,
        "semantics":{
            "snapshot":"finalized observations; collection was not an atomic AWS-wide snapshot",
            "routes":"reported configuration, including blackholes; no effective reachability inference",
            "internet_facing_load_balancers":"AWS scheme plus observed listeners; not proof of public access",
            "unresolved_references":"an endpoint identity was not observed in this same scan; not proof of absence",
            "missing_fields":"supplemental or incomplete observations may omit native configuration fields"
        }}),
    )
}

#[tracing::instrument(skip_all, fields(panel = name))]
async fn panel(
    store: &Store,
    scan: &str,
    name: &'static str,
    sql: &'static str,
    limit: usize,
) -> Result<PanelResult> {
    let probe = limit
        .checked_add(1)
        .context("architecture row limit overflow")?;
    let sql = format!("{} LIMIT {probe}", sql.replace("{scan}", scan));
    let mut result = store.query(&sql, probe).await?;
    let rows = result
        .as_array_mut()
        .context("SQL result must be a row array")?;
    let truncated = rows.len() > limit;
    rows.truncate(limit);
    Ok((name, json!({"truncated":truncated,"rows":result})))
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)] // JSON string keys are infallible; arrays use `first`.
mod tests {
    use super::report_open;
    use crate::json_test::first;
    use inventory_core::{CollectionPage, CollectionScope, Edge, Resource, ScanStatus, Source};
    use inventory_engine::store::Store;
    use serde_json::json;
    use std::collections::BTreeMap;

    #[tokio::test]
    async fn refuses_mutable_snapshots() -> anyhow::Result<()> {
        let store = Store::open(":memory:").await?;
        store.begin_scan("running", 1).await?;
        assert!(report_open(&store, "running", 10).await.is_err());
        store.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn reports_partiality_missing_targets_and_real_truncation() -> anyhow::Result<()> {
        let store = Store::open(":memory:").await?;
        let scan = "quoted'scan";
        store.begin_scan(scan, 1).await?;
        let resources=(0..2).map(|i|Resource{
            id:format!("lb-{i}"),account_id:"111111111111".into(),region:"us-east-1".into(),
            kind:"aws_elbv2_load_balancer".into(),name:None,
            properties:json!({"scheme":"internet-facing","type":"application","dns_name":format!("example-{i}.invalid")}),
            tags:BTreeMap::new(),source:Source::Direct,observed_at:2,
        }).collect();
        store
            .apply_page(
                scan,
                CollectionPage {
                    scope: CollectionScope {
                        account_id: "111111111111".into(),
                        region: "us-east-1".into(),
                        collector: "fixture".into(),
                        parent_id: None,
                        source: Source::Direct,
                    },
                    page_number: 1,
                    next_token: None,
                    resources,
                    edges: vec![Edge {
                        from_id: "lb-0".into(),
                        to_id: "vpc-unobserved".into(),
                        relation: "IN_VPC".into(),
                        source: Source::Direct,
                        observed_at: 2,
                        properties: json!({}),
                    }],
                },
            )
            .await?;
        store.finish_scan(scan, ScanStatus::Partial, 3).await?;
        let report = report_open(&store, scan, 1).await?;
        assert_eq!(report["status"], json!("partial"));
        assert_eq!(
            report["panels"]["internet_facing_load_balancers"]["truncated"],
            json!(true)
        );
        assert_eq!(
            report["panels"]["unresolved_references"]["truncated"],
            json!(false)
        );
        assert_eq!(
            first(&report["panels"]["unresolved_references"]["rows"])["target_observed"],
            json!(0)
        );
        assert_eq!(report["summary"]["resources"], json!(2));
        store.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn retains_ipv6_and_newer_route_target_evidence() -> anyhow::Result<()> {
        let store = Store::open(":memory:").await?;
        store.begin_scan("routes", 1).await?;
        let routes = json!([
            {"destination_ipv6_cidr_block":"::/0",
             "egress_only_internet_gateway_id":"eigw-example","state":"active"},
            {"destination_prefix_list_id":"pl-example","gateway_id":"vpce-example",
             "state":"active"},
            {"destination_cidr_block":"10.2.0.0/16","instance_id":"i-example",
             "instance_owner_id":"222222222222","state":"blackhole"},
            {"destination_cidr_block":"10.3.0.0/16","odb_network_arn":"arn:aws:odb:us-east-1:111111111111:odb-network/example",
             "ip_address":"10.3.1.2","state":"active"}
        ]);
        store.apply_page("routes", CollectionPage {
            scope: CollectionScope {
                account_id:"111111111111".into(),region:"us-east-1".into(),
                collector:"fixture-routes".into(),parent_id:None,source:Source::Direct,
            },
            page_number:1,next_token:None,
            resources:vec![Resource {
                id:"arn:aws:ec2:us-east-1:111111111111:route-table/rtb-example".into(),
                account_id:"111111111111".into(),region:"us-east-1".into(),
                kind:"aws_vpc_route_table".into(),name:None,
                properties:json!({"route_table_id":"rtb-example","vpc_id":"vpc-example","routes":routes}),
                tags:BTreeMap::new(),source:Source::Direct,observed_at:2,
            }],edges:vec![],
        }).await?;
        store.finish_scan("routes", ScanStatus::Complete, 3).await?;
        let report = report_open(&store, "routes", 10).await?;
        let rows = report["panels"]["routes"]["rows"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("expected route rows"))?;
        assert_eq!(rows.len(), 4);
        let egress = rows
            .iter()
            .find(|row| row["destination_ipv6_cidr_block"] == "::/0")
            .ok_or_else(|| anyhow::anyhow!("missing IPv6 route"))?;
        assert_eq!(egress["egress_only_internet_gateway_id"], "eigw-example");
        let endpoint = rows
            .iter()
            .find(|row| row["gateway_id"] == "vpce-example")
            .ok_or_else(|| anyhow::anyhow!("missing endpoint route"))?;
        assert_eq!(endpoint["vpc_endpoint_id"], "vpce-example");
        let instance = rows
            .iter()
            .find(|row| row["instance_id"] == "i-example")
            .ok_or_else(|| anyhow::anyhow!("missing instance route"))?;
        assert_eq!(instance["instance_owner_id"], "222222222222");
        assert_eq!(instance["state"], "blackhole");
        let odb = rows
            .iter()
            .find(|row| row["ip_address"] == "10.3.1.2")
            .ok_or_else(|| anyhow::anyhow!("missing ODB route"))?;
        assert_eq!(
            odb["odb_network_arn"],
            "arn:aws:odb:us-east-1:111111111111:odb-network/example"
        );
        let raw: serde_json::Value = serde_json::from_str(
            odb["route_json"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("missing raw route evidence"))?,
        )?;
        assert_eq!(raw["ip_address"], "10.3.1.2");
        store.close().await;
        Ok(())
    }
}
