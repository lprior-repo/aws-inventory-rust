use super::Fault;
use serde_json::{Value, json};

const AWS: Commercial = Commercial {
    prefixes: &["us", "eu", "ap", "sa", "ca", "me", "af", "il", "mx"],
    regions: &[
        "af-south-1",
        "ap-east-1",
        "ap-east-2",
        "ap-northeast-1",
        "ap-northeast-2",
        "ap-northeast-3",
        "ap-south-1",
        "ap-south-2",
        "ap-southeast-1",
        "ap-southeast-2",
        "ap-southeast-3",
        "ap-southeast-4",
        "ap-southeast-5",
        "ap-southeast-6",
        "ap-southeast-7",
        "aws-global",
        "ca-central-1",
        "ca-west-1",
        "eu-central-1",
        "eu-central-2",
        "eu-north-1",
        "eu-south-1",
        "eu-south-2",
        "eu-west-1",
        "eu-west-2",
        "eu-west-3",
        "il-central-1",
        "me-central-1",
        "me-south-1",
        "me-west-1",
        "mx-central-1",
        "sa-east-1",
        "sa-west-1",
        "us-east-1",
        "us-east-2",
        "us-west-1",
        "us-west-2",
    ],
    dns: "amazonaws.com",
    dual_dns: "api.aws",
    global: "us-east-1",
};

struct Commercial {
    prefixes: &'static [&'static str],
    regions: &'static [&'static str],
    dns: &'static str,
    dual_dns: &'static str,
    global: &'static str,
}

pub(super) fn contains(region: &str) -> bool {
    if AWS.regions.contains(&region) {
        return true;
    }
    let mut parts = region.split('-');
    let (Some(prefix), Some(area), Some(number), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return false;
    };
    AWS.prefixes.contains(&prefix)
        && !area.is_empty()
        && !number.is_empty()
        && area.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        && number.bytes().all(|b| b.is_ascii_digit())
}

pub(super) fn resolve(region: &str) -> Result<Value, Fault> {
    if !contains(region) {
        return Err(Fault::Partition(region.into()));
    }
    Ok(
        json!({"name":"aws", "dnsSuffix":AWS.dns, "dualStackDnsSuffix":AWS.dual_dns,
        "supportsFIPS":true, "supportsDualStack":true, "implicitGlobalRegion":AWS.global}),
    )
}
