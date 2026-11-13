use serde_json::{Value, json};

pub fn catalog() -> Value {
    let entries = [
        (
            "aws_ec2_instance",
            "ec2",
            "DescribeInstances",
            "IN_VPC,IN_SUBNET,USES_SECURITY_GROUP,USES_INSTANCE_PROFILE",
        ),
        ("aws_vpc", "ec2", "DescribeVpcs", ""),
        ("aws_vpc_subnet", "ec2", "DescribeSubnets", "IN_VPC"),
        (
            "aws_vpc_security_group",
            "ec2",
            "DescribeSecurityGroups",
            "IN_VPC",
        ),
        (
            "aws_ec2_network_interface",
            "ec2",
            "DescribeNetworkInterfaces",
            "IN_VPC,IN_SUBNET,ATTACHED_TO,USES_SECURITY_GROUP",
        ),
        ("aws_ebs_volume", "ec2", "DescribeVolumes", "ATTACHED_TO"),
        (
            "aws_vpc_route_table",
            "ec2",
            "DescribeRouteTables",
            "IN_VPC,MAIN_ROUTE_TABLE_FOR,ASSOCIATED_WITH_SUBNET,ASSOCIATED_WITH_GATEWAY,ROUTES_TO,PROPAGATES_FROM",
        ),
        (
            "aws_vpc_internet_gateway",
            "ec2",
            "DescribeInternetGateways",
            "ATTACHED_TO_VPC",
        ),
        (
            "aws_vpc_nat_gateway",
            "ec2",
            "DescribeNatGateways",
            "IN_VPC,IN_SUBNET,USES_ROUTE_TABLE,USES_NETWORK_INTERFACE,USES_ELASTIC_IP",
        ),
        (
            "aws_vpc_endpoint",
            "ec2",
            "DescribeVpcEndpoints",
            "IN_VPC,USES_ROUTE_TABLE,USES_SUBNET,USES_NETWORK_INTERFACE,USES_SECURITY_GROUP,TARGETS_SERVICE_NETWORK,TARGETS_RESOURCE_CONFIGURATION",
        ),
        (
            "aws_vpc_network_acl",
            "ec2",
            "DescribeNetworkAcls",
            "IN_VPC,ASSOCIATED_WITH_SUBNET",
        ),
        (
            "aws_vpc_peering_connection",
            "ec2",
            "DescribeVpcPeeringConnections",
            "REQUESTER_VPC,ACCEPTER_VPC",
        ),
        (
            "aws_dynamodb_table",
            "dynamodb",
            "ListTables,DescribeTable",
            "",
        ),
        (
            "aws_lambda_function",
            "lambda",
            "ListFunctions",
            "IN_VPC,USES_SUBNET,USES_SECURITY_GROUP,ASSUMES_ROLE",
        ),
        (
            "aws_ecs_cluster",
            "ecs",
            "ListClusters,DescribeClusters",
            "",
        ),
        (
            "aws_ecs_service",
            "ecs",
            "ListServices,DescribeServices",
            "IN_CLUSTER,USES_TASK_DEFINITION,USES_SUBNET,USES_SECURITY_GROUP,USES_TARGET_GROUP,USES_CLASSIC_LOAD_BALANCER",
        ),
        (
            "aws_ecs_task",
            "ecs",
            "ListTasks,DescribeTasks",
            "IN_CLUSTER,USES_TASK_DEFINITION,RUNS_ON,USES_NETWORK_INTERFACE,IN_SUBNET",
        ),
        (
            "aws_ecs_task_definition",
            "ecs",
            "ListTaskDefinitions,DescribeTaskDefinition",
            "USES_TASK_ROLE,USES_EXECUTION_ROLE,MOUNTS_EFS,MOUNTS_FSX,MOUNTS_S3_FILES",
        ),
        (
            "aws_eks_cluster",
            "eks",
            "ListClusters,DescribeCluster",
            "IN_VPC,USES_SUBNET,USES_SECURITY_GROUP,ASSUMES_ROLE",
        ),
        (
            "aws_eks_node_group",
            "eks",
            "ListNodegroups,DescribeNodegroup",
            "IN_CLUSTER,USES_SUBNET,ASSUMES_ROLE",
        ),
        (
            "aws_elbv2_load_balancer",
            "elasticloadbalancing",
            "DescribeLoadBalancers",
            "IN_VPC,USES_SUBNET,USES_SECURITY_GROUP",
        ),
        (
            "aws_elbv2_listener",
            "elasticloadbalancing",
            "DescribeListeners",
            "LISTENER_OF,FORWARDS_TO",
        ),
        (
            "aws_elbv2_target_group",
            "elasticloadbalancing",
            "DescribeTargetGroups",
            "TARGET_GROUP_OF,IN_VPC,HAS_TARGET",
        ),
        (
            "aws_elbv2_target_health",
            "elasticloadbalancing",
            "DescribeTargetHealth",
            "TARGETS",
        ),
        (
            "aws_elbv2_rule",
            "elasticloadbalancing",
            "DescribeRules",
            "RULE_OF,FORWARDS_TO",
        ),
        (
            "aws_s3_bucket",
            "s3",
            "ListBuckets,HeadBucket,GetBucketVersioning,GetBucketEncryption,GetPublicAccessBlock,GetBucketPolicyStatus,GetBucketLogging,GetBucketReplication,GetBucketWebsite,GetBucketOwnershipControls,GetBucketNotificationConfiguration,GetBucketTagging",
            "ASSUMES_ROLE,ENCRYPTED_WITH,REPLICA_ENCRYPTED_WITH,REPLICATES_TO,LOGS_TO,REDIRECTS_TO_DNS,NOTIFIES_TOPIC,NOTIFIES_QUEUE,NOTIFIES_FUNCTION",
        ),
        (
            "aws_iam_role",
            "iam",
            "ListRoles,GetRole,ListAttachedRolePolicies",
            "ATTACHED_POLICY,USES_PERMISSIONS_BOUNDARY,DECLARES_TRUST",
        ),
        (
            "aws_iam_instance_profile",
            "iam",
            "ListInstanceProfiles,GetInstanceProfile",
            "HAS_ROLE",
        ),
        ("aws_iam_policy", "iam", "ListPolicies,GetPolicy", ""),
        (
            "aws_route53_zone",
            "route53",
            "ListHostedZones,GetHostedZone,ListTagsForResource",
            "ASSOCIATED_WITH_VPC",
        ),
        (
            "aws_route53_record",
            "route53",
            "ListResourceRecordSets",
            "IN_HOSTED_ZONE,ALIASES_DNS_NAME,USES_HEALTH_CHECK,DNS_TARGET",
        ),
        (
            "aws_cloudfront_distribution",
            "cloudfront",
            "ListDistributions,ListTagsForResource",
            "ORIGINATES_FROM,USES_VPC_ORIGIN,USES_CERTIFICATE,USES_WEB_ACL,ASSOCIATES_LAMBDA,ASSOCIATES_FUNCTION,LOGS_TO",
        ),
    ];
    let tables: Vec<_> = entries.iter().map(|(table, service, operations, relations)| json!({
        "table":table,"service":service,"selector":selector(table, service),
        "sdk_service":if *service == "elasticloadbalancing" { "elasticloadbalancingv2" } else { *service },
        "operations":operations.split(',').collect::<Vec<_>>(),
        "scope":scope(table),
        "relationship_intent":relations,
        "outgoing_relationships":relations.split(',').filter(|relation| !relation.is_empty()).collect::<Vec<_>>(),
        "tag_collection":if matches!(*table, "aws_dynamodb_table" | "aws_lambda_function") || table.starts_with("aws_elbv2_") {
            "not collected; tags_collected=false"
        } else if *table == "aws_route53_record" { "DNS record sets do not expose resource tags" }
        else if *service == "iam" { "tags returned by detail reads; details_available distinguishes list-only fallback" }
        else if matches!(*service,"s3"|"iam"|"route53"|"cloudfront") { "native detail or tag API; availability is recorded explicitly" }
        else { "tags returned by the selected list/describe operations" },
        "level":"selected configuration fields and documented relationships; not full upstream schema parity"
    })).collect();
    json!({"schema_version":6,"sqlite_schema_version":6,
        "native_resource_kind_count":tables.len(),"aws_feature_enabled":cfg!(feature="aws"),"tables":tables,
        "selectors":[
            {"selector":"ec2","aliases":["vpc","ebs"],"collector_groups":["ec2","network"]},
            {"selector":"network","aliases":[],"collector_groups":["network"]},
            {"selector":"dynamodb","aliases":["dynamo"],"collector_groups":["dynamodb"]},
            {"selector":"lambda","aliases":[],"collector_groups":["lambda"]},
            {"selector":"ecs","aliases":[],"collector_groups":["ecs","ecs-task-definitions"]},
            {"selector":"ecs-task-definitions","aliases":[],"collector_groups":["ecs-task-definitions"]},
            {"selector":"eks","aliases":[],"collector_groups":["eks"]},
            {"selector":"elbv2","aliases":["elb","load-balancing"],"collector_groups":["elbv2"]},
            {"selector":"s3","aliases":[],"collector_groups":["s3"]},
            {"selector":"iam","aliases":[],"collector_groups":["iam"]},
            {"selector":"route53","aliases":[],"collector_groups":["route53"]},
            {"selector":"cloudfront","aliases":[],"collector_groups":["cloudfront"]}],
        "default_selection":"all twelve native collector groups; Cloud Control requires --cloud-control",
        "derived_sql_views":[
            {"view":"aws_ec2_application_load_balancer","source":"aws_elbv2_load_balancer","meaning":"type-filtered application load balancers"},
            {"view":"aws_ec2_network_load_balancer","source":"aws_elbv2_load_balancer","meaning":"type-filtered network load balancers"},
            {"view":"aws_ec2_gateway_load_balancer","source":"aws_elbv2_load_balancer","meaning":"type-filtered gateway load balancers"},
            {"view":"aws_vpc_route","source":"aws_vpc_route_table","meaning":"routes expanded from the latest observed route table; not a separately collected resource kind"},
            {"view":"scan_vpc_routes","source":"scan_resources","meaning":"route rows scoped to each inventory snapshot"},
            {"view":"scan_architecture_edges","source":"scan_edges","meaning":"exact-identity joins exposing observed and unresolved endpoints"},
            {"view":"aws_cloudfront_origin","source":"aws_cloudfront_distribution","meaning":"origin configuration rows with header/path values omitted"},
            {"view":"aws_iam_instance_profile_role","source":"aws_iam_instance_profile","meaning":"roles referenced by instance profiles"},
            {"view":"aws_route53_zone_vpc","source":"aws_route53_zone","meaning":"private-zone VPC associations with explicit owner evidence"}],
        "relationship_notes":[
            "Outgoing relationships name the source resource kind, not necessarily the scope that emitted the edge.",
            "Target-group HAS_TARGET edges are emitted by the DescribeTargetHealth child scope.",
            "Repeated network routes or forwarding actions to the same target share one edge; evidence or action properties retain their distinctions.",
            "Target-health memberships use a synthetic tuple identity including target group, target ID, port, Availability Zone and QUIC server ID; arbitrary IP ownership remains unknown."],
        "optional_sources":[{"source":"aws_config","operation":"SelectAggregateResourceConfig","requires":"existing explicit or unambiguously discovered aggregator"},
        {"source":"resource_explorer","operation":"ListResources","requires":"existing explicit or validated discovered view"}],
        "sdk_catalog":{"service_crates":inventory_sdk_catalog::SERVICE_COUNT,"inspect_command":"sdk-catalog","meaning":"pinned SDK availability; not implemented collector coverage"},
        "bounds":{"max_pages_per_scope":10000,"max_items_per_scope":100000,"max_in_flight_requests":4096,
            "max_unpaginated_target_health_memberships":10000},
        "not_claimed":["all AWS resource types","full Steampipe schemas or PostgreSQL syntax","full Cartography schema compatibility",
            "Kubernetes objects inside EKS","Classic ELB or complete ELBv2 configuration",
            "native tag API enrichment for DynamoDB, Lambda or ELBv2","ELBv2 authentication details or complete listener certificate lists",
            "effective network reachability or effective IAM permissions",
            "native-equivalent supplemental configuration or relationship enrichment",
            "automatic inference of all account API quotas","native API resume from stored cursors",
            "live-account validated completeness"]})
}

fn selector<'a>(table: &str, service: &'a str) -> &'a str {
    match table {
        "aws_ecs_task_definition" => "ecs-task-definitions",
        "aws_vpc_route_table"
        | "aws_vpc_internet_gateway"
        | "aws_vpc_nat_gateway"
        | "aws_vpc_endpoint"
        | "aws_vpc_network_acl"
        | "aws_vpc_peering_connection" => "network",
        _ if table.starts_with("aws_elbv2_") => "elbv2",
        _ => service,
    }
}

fn scope(table: &str) -> &'static str {
    match table {
        "aws_s3_bucket" => {
            "account-global listing; per-bucket details use the authoritative bucket region"
        }
        "aws_iam_role"
        | "aws_iam_instance_profile"
        | "aws_iam_policy"
        | "aws_route53_zone"
        | "aws_cloudfront_distribution" => "account + global; once per target scan",
        "aws_route53_record" => {
            "account + global + hosted zone; composite name/type/set-identifier cursor"
        }
        "aws_dynamodb_table" => {
            "account + region; per-table description failures have child scopes"
        }
        "aws_ecs_service" => "account + region + cluster ARN",
        "aws_ecs_task" => "account + region + cluster ARN + desired-status filter",
        "aws_ecs_task_definition" => {
            "account + region; ACTIVE revision discovery and full task-definition ARN child scopes"
        }
        "aws_eks_node_group" => "account + region + cluster ARN",
        "aws_elbv2_listener" => {
            "account + region + load balancer ARN; application, network and gateway types"
        }
        "aws_elbv2_rule" => "account + region + application load balancer listener ARN",
        "aws_elbv2_target_health" => {
            "account + region + target group ARN; one unpaginated response"
        }
        _ => "account + region",
    }
}

pub fn sdk_catalog(service: Option<String>) -> anyhow::Result<Value> {
    let manifest: Value = serde_json::from_str(inventory_sdk_catalog::SERVICE_MANIFEST_JSON)?;
    let Some(service) = service else {
        return Ok(manifest);
    };
    let services = manifest
        .get("services")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("invalid embedded SDK catalog"))?;
    services
        .iter()
        .find(|entry| {
            entry.get("name").and_then(Value::as_str) == Some(&service)
                || entry.get("feature").and_then(Value::as_str) == Some(&service)
        })
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!("SDK service is not present in the pinned catalog: {service}")
        })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use serde_json::Value;

    use super::catalog;

    #[test]
    fn catalog_kinds_and_derived_views_match_the_sqlite_schema()
    -> Result<(), Box<dyn std::error::Error>> {
        let catalog = catalog();
        let tables = catalog
            .get("tables")
            .and_then(Value::as_array)
            .ok_or("missing native table catalog")?;
        assert_eq!(tables.len(), 32);
        assert_eq!(
            catalog
                .get("native_resource_kind_count")
                .and_then(Value::as_u64),
            Some(32)
        );
        assert_eq!(
            catalog.get("schema_version").and_then(Value::as_u64),
            Some(6)
        );
        assert_eq!(
            catalog.get("sqlite_schema_version").and_then(Value::as_u64),
            Some(6)
        );
        let migrations = [
            include_str!("../crates/engine/sql/001_inventory.sql"),
            include_str!("../crates/engine/sql/002_architecture.sql"),
            include_str!("../crates/engine/sql/003_sources.sql"),
            include_str!("../crates/engine/sql/004_global_inventory.sql"),
        ];
        let mut native_names = BTreeSet::new();
        for table in tables {
            let name = table
                .get("table")
                .and_then(Value::as_str)
                .ok_or("missing native table name")?;
            assert!(native_names.insert(name), "duplicate native kind: {name}");
            let declaration = format!("CREATE VIEW {name} AS");
            assert!(
                migrations.iter().any(|sql| sql.contains(&declaration)),
                "catalog table has no SQLite view: {name}"
            );
            let operations = table
                .get("operations")
                .and_then(Value::as_array)
                .ok_or("missing operation list")?;
            assert!(!operations.is_empty());
            assert!(
                operations
                    .iter()
                    .all(|operation| operation.as_str().is_some_and(|value| !value.is_empty()))
            );
        }
        let derived = catalog
            .get("derived_sql_views")
            .and_then(Value::as_array)
            .ok_or("missing derived view catalog")?;
        let mut derived_names = BTreeSet::new();
        for view in derived {
            let name = view
                .get("view")
                .and_then(Value::as_str)
                .ok_or("missing derived view name")?;
            assert!(derived_names.insert(name), "duplicate derived view: {name}");
            assert!(
                !native_names.contains(name),
                "derived view counted as a native kind: {name}"
            );
            let declaration = format!("CREATE VIEW {name} AS");
            assert!(
                migrations.iter().any(|sql| sql.contains(&declaration)),
                "derived view has no SQLite definition: {name}"
            );
        }
        Ok(())
    }
}
