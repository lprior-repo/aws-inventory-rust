use super::*;
mod definitions;
pub(super) use definitions::all;

pub(super) struct Case {
    pub service: &'static str,
    pub operation: &'static str,
    pub kind: &'static str,
    pub items: &'static str,
    pub token: Option<&'static str>,
    pub ids: [String; 2],
    pub rows: [Json; 2],
    pub fixed: Json,
}

fn case(
    meta: (&'static str, &'static str, &'static str, &'static str),
    identity: &str,
    ids: [String; 2],
    token: Option<&'static str>,
    fixed: Json,
) -> Case {
    let (service, operation, kind, items) = meta;
    let rows = ids.each_ref().map(|id| {
        if identity == "$" {
            json!(id)
        } else {
            json!({identity:id})
        }
    });
    Case {
        service,
        operation,
        kind,
        items,
        token,
        ids,
        rows,
        fixed,
    }
}
fn names() -> [String; 2] {
    ["unattached-first".to_owned(), "unattached-later".to_owned()]
}
fn arns(service: &str, resource: &str) -> [String; 2] {
    [
        "00000000-0000-0000-0000-000000000001",
        "00000000-0000-0000-0000-000000000002",
    ]
    .map(|id| format!("arn:aws:{service}:us-east-1:123456789012:{resource}/{id}"))
}
