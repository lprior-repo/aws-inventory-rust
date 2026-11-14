use serde_json::{Value, json};

pub fn catalog() -> Value {
    let tables: Vec<_> = entries::ENTRIES.iter().map(|(table, service, operations, relations)| json!({
        "table":table,"service":service,"selector":selector(table, service),
        "sdk_service":if *service == "elasticloadbalancing" { "elasticloadbalancingv2" } else { *service },
        "operations":operations.split(',').collect::<Vec<_>>(),
        "scope":scope(table),
        "relationship_intent":relations,
        "outgoing_relationships":relations.split(',').filter(|relation| !relation.is_empty()).collect::<Vec<_>>(),
        "tag_collection":if matches!(*table, "aws_dynamodb_table" | "aws_lambda_function") || table.starts_with("aws_elbv2_") {
            "not collected; tags_collected=false"
        } else if *table == "aws_sqs_queue" { "ListQueueTags; a failed tag read keeps the queue with tags_collected=false" }
        else if *table == "aws_sns_topic" { "ListTagsForResource; a failed tag read keeps the topic with tags_collected=false" }
        else if *table == "aws_sns_subscription" { "subscriptions expose no tags in the ListSubscriptions response" }
        else if *table == "aws_kms_key" { "ListResourceTags for customer-managed keys; a failed tag read keeps the key with tags_collected=false" }
        else if *table == "aws_kms_alias" { "aliases expose no tags in the ListAliases response" }
        else if *table == "aws_step_functions_state_machine" || *table == "aws_step_functions_activity" { "ListTagsForResource; a failed tag read keeps the resource with tags_collected=false" }
        else if *table == "aws_route53_record" { "DNS record sets do not expose resource tags" }
        else if *service == "iam" { "tags returned by detail reads; details_available distinguishes list-only fallback" }
        else if matches!(*service,"s3"|"iam"|"route53"|"cloudfront") { "native detail or tag API; availability is recorded explicitly" }
        else { "tags returned by the selected list/describe operations" },
        "level":"selected configuration fields and documented relationships; not full upstream schema parity"
    })).collect();
    json!({"schema_version":6,"sqlite_schema_version":inventory_engine::store::SCHEMA_VERSION,
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
            {"selector":"cloudfront","aliases":[],"collector_groups":["cloudfront"]},
            {"selector":"sqs","aliases":[],"collector_groups":["sqs"]},
            {"selector":"sns","aliases":[],"collector_groups":["sns"]},
            {"selector":"kms","aliases":[],"collector_groups":["kms"]},
            {"selector":"step-functions","aliases":["stepfunctions","sfn"],"collector_groups":["step-functions"]}],
        "default_selection":"all sixteen native collector groups; Cloud Control requires --cloud-control",
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
            "Target-health memberships use a synthetic tuple identity including target group, target ID, port, Availability Zone and QUIC server ID; arbitrary IP ownership remains unknown.",
            "SQS DELIVERS_TO edges require a parseable RedrivePolicy.deadLetterTargetArn; no other queue relationship is inferred.",
            "SNS SUBSCRIBES_TO edges require a well-formed TopicArn in the scan account, region and partition; the target topic may be unobserved in the snapshot and remains a dangling exact-ARN reference.",
            "Step Functions ASSUMES_ROLE edges require an exact IAM role ARN in the scan partition and account; ENCRYPTED_BY edges require an exact KMS key ARN in the scan partition, region and account. Aliases, bare key IDs and foreign keys stay properties, and logging destinations produce no CloudWatch edge."],
        "optional_sources":[{"source":"aws_config","operation":"SelectAggregateResourceConfig","requires":"existing explicit or unambiguously discovered aggregator"},
        {"source":"resource_explorer","operation":"ListResources","requires":"existing explicit or validated discovered view"}],
        "bounds":{"max_pages_per_scope":10000,"max_items_per_scope":100000,"max_in_flight_requests":4096,
            "max_unpaginated_target_health_memberships":10000},
        "not_claimed":["all AWS resource types","full Steampipe schemas or PostgreSQL syntax","full Cartography schema compatibility",
            "Kubernetes objects inside EKS","Classic ELB or complete ELBv2 configuration",
            "native tag API enrichment for DynamoDB, Lambda or ELBv2","ELBv2 authentication details or complete listener certificate lists",
            "KMS key policy bodies, key material, or an inferred alias-to-key edge",
            "Step Functions definition bodies, variable references, execution data, or CloudWatch log-group edges",
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
        "aws_step_functions_state_machine" | "aws_step_functions_activity" => "step-functions",
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
        "aws_sqs_queue" => {
            "account + region; queue URL used as the read handle, queue ARN as identity, per-queue attribute/tag child scopes"
        }
        "aws_sns_topic" => {
            "account + region; topic ARN as identity, per-topic attribute/tag child scopes"
        }
        "aws_sns_subscription" => {
            "account + region; account-wide subscription listing filtered by the subscription ARN region; pending confirmations use a deterministic synthetic digest identity"
        }
        "aws_kms_key" => {
            "account + region; exact key ARN as identity, per-key describe/rotation/tag child scopes for customer-managed keys"
        }
        "aws_kms_alias" => {
            "account + region; exact alias ARN as identity, target key id kept as a property"
        }
        "aws_step_functions_state_machine" => {
            "account + region; exact state machine ARN as identity, per-machine describe/tag child scopes, definition and variable references omitted"
        }
        "aws_step_functions_activity" => {
            "account + region; exact activity ARN as identity, per-activity describe/tag child scopes"
        }
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

#[cfg(test)]
mod tests;

mod entries;
