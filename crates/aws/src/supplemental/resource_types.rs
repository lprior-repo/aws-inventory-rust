use super::*;

pub(super) fn resource_kind(resource_type: &str) -> String {
    let normalized = normalize_resource_type(resource_type);
    let known = match normalized.as_str() {
        "ec2:instance" => Some("aws_ec2_instance"),
        "ec2:vpc" => Some("aws_vpc"),
        "ec2:subnet" => Some("aws_vpc_subnet"),
        "ec2:securitygroup" | "ec2:security-group" => Some("aws_vpc_security_group"),
        "ec2:networkinterface" | "ec2:network-interface" => Some("aws_ec2_network_interface"),
        "ec2:routetable" | "ec2:route-table" => Some("aws_vpc_route_table"),
        "ec2:internetgateway" | "ec2:internet-gateway" => Some("aws_vpc_internet_gateway"),
        "ec2:natgateway" | "ec2:nat-gateway" => Some("aws_vpc_nat_gateway"),
        "ec2:vpcendpoint" | "ec2:vpc-endpoint" => Some("aws_vpc_endpoint"),
        "ec2:networkacl" | "ec2:network-acl" => Some("aws_vpc_network_acl"),
        "ec2:vpcpeeringconnection" | "ec2:vpc-peering-connection" => {
            Some("aws_vpc_peering_connection")
        }
        "elasticloadbalancingv2:loadbalancer" => Some("aws_elbv2_load_balancer"),
        "elasticloadbalancingv2:listener" => Some("aws_elbv2_listener"),
        "elasticloadbalancingv2:targetgroup" => Some("aws_elbv2_target_group"),
        "elasticloadbalancingv2:listenerrule" => Some("aws_elbv2_rule"),
        "ec2:volume" => Some("aws_ebs_volume"),
        "dynamodb:table" => Some("aws_dynamodb_table"),
        "lambda:function" => Some("aws_lambda_function"),
        "ecs:cluster" => Some("aws_ecs_cluster"),
        "ecs:service" => Some("aws_ecs_service"),
        "ecs:task" => Some("aws_ecs_task"),
        "eks:cluster" => Some("aws_eks_cluster"),
        "eks:nodegroup" => Some("aws_eks_node_group"),
        "s3:bucket" => Some("aws_s3_bucket"),
        "iam:role" => Some("aws_iam_role"),
        "iam:instanceprofile" | "iam:instance-profile" => Some("aws_iam_instance_profile"),
        "iam:policy" | "iam:managedpolicy" => Some("aws_iam_policy"),
        "route53:hostedzone" | "route53:hosted-zone" => Some("aws_route53_zone"),
        "route53:recordset" | "route53:record-set" => Some("aws_route53_record"),
        "cloudfront:distribution" => Some("aws_cloudfront_distribution"),
        _ => None,
    };
    known
        .map(str::to_owned)
        .unwrap_or_else(|| fallback_kind(&normalized))
}

pub(super) fn normalize_resource_type(resource_type: &str) -> String {
    resource_type
        .to_ascii_lowercase()
        .replace("aws::", "")
        .replace("::", ":")
}

pub(super) fn fallback_kind(normalized: &str) -> String {
    format!(
        "aws_{}",
        normalized
            .chars()
            .take(256)
            .map(|character| if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            })
            .collect::<String>()
    )
}

pub(super) fn explorer_resource_kind(resource_type: &str, arn: &str) -> String {
    let normalized = normalize_resource_type(resource_type);
    let elb_type = normalized
        .strip_prefix("elasticloadbalancing:")
        .or_else(|| normalized.strip_prefix("elasticloadbalancingv2:"));
    match elb_type {
        Some(resource_type) => explorer_elbv2_kind(resource_type, arn)
            .map(str::to_owned)
            .unwrap_or_else(|| fallback_kind(&normalized)),
        None => resource_kind(resource_type),
    }
}

pub(super) fn explorer_elbv2_kind(resource_type: &str, arn: &str) -> Option<&'static str> {
    let mut fields = arn.splitn(6, ':');
    if fields.next()? != "arn"
        || fields.next()?.is_empty()
        || fields.next()? != "elasticloadbalancing"
        || fields.next()?.is_empty()
        || fields.next()?.is_empty()
    {
        return None;
    }
    let mut path = fields.next()?.split('/');
    let family = path.next()?;
    let variant = path.next()?;
    let (kind, remaining) = match (family, variant) {
        ("loadbalancer", "app" | "net" | "gwy") => ("aws_elbv2_load_balancer", 2),
        ("listener", "app" | "net" | "gwy") => ("aws_elbv2_listener", 3),
        ("listener-rule", "app" | "net") => ("aws_elbv2_rule", 4),
        ("targetgroup", name) if !name.is_empty() => ("aws_elbv2_target_group", 1),
        _ => return None,
    };

    if !(0..remaining).all(|_| path.next().is_some_and(|part| !part.is_empty()))
        || path.next().is_some()
    {
        return None;
    }
    let type_matches = if let Some((type_family, type_variant)) = resource_type.split_once('/') {
        type_family == family && type_variant == variant && family != "targetgroup"
    } else {
        resource_type == family || (resource_type == "listenerrule" && family == "listener-rule")
    };
    type_matches.then_some(kind)
}

pub(super) fn string(value: &Value, key: &str) -> Result<String, AwsScanError> {
    required(value.get(key).and_then(Value::as_str), key)
}

pub(super) fn json_tags(value: Option<&Value>) -> BTreeMap<String, String> {
    match value {
        Some(Value::Object(object)) => object
            .iter()
            .filter_map(|(key, value)| Some((key.clone(), value.as_str()?.to_owned())))
            .collect(),
        Some(Value::Array(array)) => array
            .iter()
            .filter_map(|entry| {
                let key = entry.get("key").or_else(|| entry.get("Key"))?.as_str()?;
                let value = entry
                    .get("value")
                    .or_else(|| entry.get("Value"))?
                    .as_str()?;
                Some((key.to_owned(), value.to_owned()))
            })
            .collect(),
        _ => BTreeMap::new(),
    }
}
