//! Rules report retained configuration evidence; they do not evaluate access.
use super::{finalized, limited_rows, literal};
use anyhow::Result;
use futures_util::{StreamExt, TryStreamExt, future::BoxFuture, stream};
use inventory_engine::store::Store;
use serde_json::{Value, json};
use std::collections::BTreeMap;

struct Rule {
    id: &'static str,
    title: &'static str,
    meaning: &'static str,
    limitation: &'static str,
    sql: &'static str,
}

type RuleResult = (&'static str, Value);
type RuleFuture<'a> = BoxFuture<'a, Result<RuleResult>>;

static RULES: [Rule; 6] = [
    Rule {
        id: "internet_facing_http_listener",
        title: "Internet-facing load balancer declares an HTTP listener",
        meaning: "The observed load balancer scheme is internet-facing and an observed listener uses HTTP.",
        limitation: "An HTTP listener can redirect to HTTPS; routes, security controls and effective exposure are not evaluated.",
        sql: "SELECT lb.id AS load_balancer_id,lb.account_id,lb.region,lb.source AS load_balancer_source,
            listener.id AS listener_id,listener.source AS listener_source,
            json_extract(listener.properties,'$.port') AS port,
            json_extract(listener.properties,'$.protocol') AS protocol,
            json_extract(listener.properties,'$.default_actions') AS default_actions
            FROM scan_resources lb JOIN scan_resources listener
            ON listener.scan_id=lb.scan_id AND listener.kind='aws_elbv2_listener'
            AND json_extract(listener.properties,'$.load_balancer_arn')=lb.id
            WHERE lb.scan_id={scan} AND lb.kind='aws_elbv2_load_balancer'
            AND coalesce(json_extract(lb.properties,'$.scheme'),json_extract(lb.properties,'$.provider_properties.Scheme'))='internet-facing'
            AND upper(json_extract(listener.properties,'$.protocol'))='HTTP'
            ORDER BY lb.id,listener.id",
    },
    Rule {
        id: "default_route_to_internet_gateway",
        title: "A default route names an internet gateway",
        meaning: "An IPv4 or IPv6 default route explicitly names an igw- gateway.",
        limitation: "A declared route does not establish an attached gateway, usable route, public address or effective network access.",
        sql: "SELECT route_table_arn,route_table_id,vpc_id,account_id,region,route_index,
            destination_cidr_block,destination_ipv6_cidr_block,gateway_id,state,origin
            FROM scan_vpc_routes WHERE scan_id={scan}
            AND (destination_cidr_block='0.0.0.0/0' OR destination_ipv6_cidr_block='::/0')
            AND gateway_id GLOB 'igw-*' ORDER BY route_table_arn,route_index",
    },
    Rule {
        id: "blackhole_route",
        title: "AWS reported a blackhole route",
        meaning: "The retained route state is blackhole.",
        limitation: "This is the collected route state, not a fresh connectivity probe or proof of the underlying cause.",
        sql: "SELECT route_table_arn,route_table_id,vpc_id,account_id,region,route_index,
            destination_cidr_block,destination_ipv6_cidr_block,destination_prefix_list_id,
            gateway_id,nat_gateway_id,transit_gateway_id,vpc_peering_connection_id,
            network_interface_id,instance_id,state,origin
            FROM scan_vpc_routes WHERE scan_id={scan} AND state='blackhole'
            ORDER BY route_table_arn,route_index",
    },
    Rule {
        id: "unresolved_architecture_reference",
        title: "A relationship endpoint was not observed in this scan",
        meaning: "At least one endpoint of a collected relationship has no selected resource observation in the same scan.",
        limitation: "Disabled collectors, permissions, identity assumptions, timing and unsupported types can explain this; absence is not inferred.",
        sql: "SELECT from_id,from_kind,relation,to_id,to_kind,source,observed_at,
            source_observed,target_observed,
            json_extract(properties,'$.owner_resolution') AS owner_resolution
            FROM scan_architecture_edges WHERE scan_id={scan}
            AND (source_observed=0 OR target_observed=0)
            ORDER BY from_id,to_id,relation",
    },
    Rule {
        id: "s3_policy_reported_public",
        title: "S3 reported a bucket policy as public",
        meaning: "A successful native GetBucketPolicyStatus observation reported IsPublic=true.",
        limitation: "Account/bucket access blocks, ACLs and other authorization controls can change effective access; no anonymous request was made.",
        sql: "SELECT id AS bucket_arn,name AS bucket_name,account_id,region,source,observed_at,
            json_extract(properties,'$.policy_status') AS policy_status,
            json_extract(properties,'$.public_access_block') AS public_access_block
            FROM scan_resources WHERE scan_id={scan} AND kind='aws_s3_bucket' AND source='direct'
            AND json_extract(properties,'$.policy_status.configured')=1
            AND json_extract(properties,'$.policy_status.is_public')=1 ORDER BY id",
    },
    Rule {
        id: "collection_scope_gap",
        title: "A selected collection scope did not complete successfully",
        meaning: "A persisted scope is failed or incomplete.",
        limitation: "Coverage gaps qualify every absence-based interpretation; this is not a statement about the AWS resource itself.",
        sql: "SELECT account_id,region,collector,parent_id,source,coverage_status,last_page,complete,failed
            FROM scan_coverage WHERE scan_id={scan} AND coverage_status<>'complete'
            ORDER BY account_id,region,collector,parent_id,source",
    },
];

#[tracing::instrument(skip_all)]
pub(super) async fn report(store: &Store, scan: &str, limit: usize) -> Result<Value> {
    super::validate_limit(limit)?;
    let status = finalized(store, scan).await?;
    let scan_literal = literal(scan);
    // Six futures are created, but only one is polled at a time. Explicit
    // future types retain the CLI's Send boundary without borrowed-item HRTB
    // inference through a stream closure.
    let work: Vec<RuleFuture<'_>> = RULES
        .iter()
        .map(|rule| {
            let work: RuleFuture<'_> = Box::pin(rule_report(store, rule, &scan_literal, limit));
            work
        })
        .collect();
    let reports: BTreeMap<&str, Value> = stream::iter(work).buffered(1).try_collect().await?;
    let truncated = reports
        .values()
        .any(|value| value.get("truncated").and_then(Value::as_bool) == Some(true));
    Ok(
        json!({"status":status.as_str(),"scan_id":scan,"rule_count":RULES.len(),
        "row_limit_per_rule":limit,"truncated":truncated,"rules":reports,
        "semantics":{
            "evidence":"rules report specific retained configuration facts, not security verdicts",
            "no_findings":"zero matches does not establish safety, completeness or absence of public access",
            "missing_fields":"a missing or failed read is unknown, not false",
            "snapshot":"all resource and endpoint joins are confined to the selected finalized scan",
            "coverage":"native normalized routes and listeners are required for their rules; raw supplemental payloads may not expose equivalent fields"
        }}),
    )
}

#[tracing::instrument(skip_all, fields(rule = rule.id))]
async fn rule_report(store: &Store, rule: &Rule, scan: &str, limit: usize) -> Result<RuleResult> {
    let result = limited_rows(store, &rule.sql.replace("{scan}", scan), limit).await?;
    Ok((
        rule.id,
        json!({"title":rule.title,"meaning":rule.meaning,
        "limitation":rule.limitation,"truncated":result.truncated,"rows":result.rows}),
    ))
}
