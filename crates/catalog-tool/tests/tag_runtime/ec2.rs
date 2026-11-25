use super::*;

const EC2_MODEL: &str = ".local/aws-models/models/ec2/service/2016-11-15/ec2-2016-11-15.json";
const NAMESPACE: &str = "com.amazonaws.ec2#";
const NOT_RESOURCE_TAGS: [&str; 4] = [
    "DescribeClassicLinkInstances",
    "DescribeInstanceSqlHaStates",
    "DescribeSnapshotTierStatus",
    "DescribeVpcClassicLink",
];

fn shape<'a>(model: &'a serde_json::Value, id: &str) -> Option<&'a serde_json::Value> {
    model.get("shapes")?.get(id)
}

fn member_target<'a>(
    model: &'a serde_json::Value,
    container: &str,
    member: &str,
) -> Option<&'a str> {
    shape(model, container)?
        .get("members")?
        .get(member)?
        .get("target")?
        .as_str()
}

fn output_shape<'a>(model: &'a serde_json::Value, operation: &str) -> Option<&'a str> {
    shape(model, &format!("{NAMESPACE}{operation}"))?
        .get("output")?
        .get("target")?
        .as_str()
}

fn item_shape<'a>(
    model: &'a serde_json::Value,
    operation: &str,
    items: &overlay::ItemSelector,
) -> Option<&'a str> {
    let output = output_shape(model, operation)?;
    let selected = match items {
        overlay::ItemSelector::Root => output,
        overlay::ItemSelector::Member(path) | overlay::ItemSelector::Explicit { path, .. } => path
            .as_str()
            .split('.')
            .try_fold(output, |container, member| {
                member_target(model, container, member)
            })?,
    };
    let selected_shape = shape(model, selected)?;
    match selected_shape.get("type")?.as_str()? {
        "list" => selected_shape.get("member")?.get("target")?.as_str(),
        _ => Some(selected),
    }
}

fn is_tag_list(model: &serde_json::Value, target: &str) -> bool {
    let Some(list) = shape(model, target) else {
        return false;
    };
    list.get("type").and_then(serde_json::Value::as_str) == Some("list")
        && list
            .get("member")
            .and_then(|member| member.get("target"))
            .and_then(serde_json::Value::as_str)
            == Some("com.amazonaws.ec2#Tag")
}

fn tag_members<'a>(model: &'a serde_json::Value, item: &str) -> Vec<&'a str> {
    let Some(members) = shape(model, item)
        .and_then(|item| item.get("members"))
        .and_then(serde_json::Value::as_object)
    else {
        return Vec::new();
    };
    members
        .iter()
        .filter(|(_, member)| {
            member
                .get("target")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|target| is_tag_list(model, target))
        })
        .map(|(name, _)| name.as_str())
        .collect()
}

fn expected_tag_member<'a>(
    model: &'a serde_json::Value,
    operation: &str,
    items: &overlay::ItemSelector,
) -> Result<Option<&'a str>, String> {
    let Some(item) = item_shape(model, operation, items) else {
        return Err(format!("unresolved item shape for {operation}"));
    };
    let members = tag_members(model, item);
    match members.as_slice() {
        [] if NOT_RESOURCE_TAGS.contains(&operation) => {
            Err(format!("excluded {operation} has no tag member"))
        }
        [] => Ok(None),
        [_] if NOT_RESOURCE_TAGS.contains(&operation) => Ok(None),
        [member] => Ok(Some(member)),
        _ => Err(format!(
            "ambiguous tag members for {operation}: {members:?}"
        )),
    }
}

fn tag_plan(fields: &smithy::Registry, member: &str) -> Result<TagPlan, String> {
    Ok(TagPlan::List {
        path: Box::new([fields
            .id(member)
            .ok_or_else(|| format!("missing {member} field"))?]),
        key: fields.id("Key").ok_or("missing Key field")?,
        value: fields.id("Value").ok_or("missing Value field")?,
    })
}

#[test]
fn given_pinned_ec2_model_when_items_carry_tag_lists_then_overlay_and_catalog_agree() -> TestResult
{
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let model: serde_json::Value = serde_json::from_slice(&fs::read(root.join(EC2_MODEL))?)?;
    let runtime =
        RuntimeCatalog::from_bytes(include_bytes!("../../../../catalog/aws-api.catalog"))?;
    let fields = smithy::Registry::parse(include_str!("../../../../catalog/field-ids.txt"))?;
    let overlays = overlay::embedded()?;
    let ec2 = overlays
        .iter()
        .find(|service| service.service() == "ec2")
        .ok_or("missing EC2 overlay")?;
    let mut tagged_operations = 0_usize;
    let mut mismatches = Vec::new();
    let mut excluded = 0_usize;
    for (name, policy) in ec2.operations() {
        let (Some(overlay::OverlayClass::Enumerate | overlay::OverlayClass::Expand), Some(items)) =
            (&policy.class, &policy.items)
        else {
            continue;
        };
        let operation = name.as_str();
        let expected = expected_tag_member(&model, operation, items)?;
        let declared = policy.tags.as_ref().map(overlay::MemberPath::as_str);
        if declared != expected {
            mismatches.push(format!(
                "{operation}: declared {declared:?}, model {expected:?}"
            ));
        }
        let plan = runtime
            .lookup("ec2", operation)
            .ok_or_else(|| format!("missing runtime plan {operation}"))?;
        let compiled = plan.resource_emissions().primary().tags();
        let expected_plan = expected
            .map(|member| tag_plan(&fields, member))
            .transpose()?;
        if compiled != expected_plan.as_ref() {
            mismatches.push(format!(
                "{operation}: compiled tag plan differs from {expected:?}"
            ));
        }
        tagged_operations += usize::from(expected.is_some());
        excluded += usize::from(NOT_RESOURCE_TAGS.contains(&operation));
    }
    assert!(mismatches.is_empty(), "{mismatches:#?}");
    assert_eq!(excluded, NOT_RESOURCE_TAGS.len());
    assert_eq!(tagged_operations, 109);
    Ok(())
}
