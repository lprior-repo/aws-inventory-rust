use super::{TestResult, paginated, replace, super_write, write_model};
use serde_json::{Value, json};

pub const EXPECTED: &str = concat!(
    "# derivations from the pinned models; nothing here makes an operation callable (VISION 5.1).\n",
    "operations 17\nenumerate 8\nexpand 4\ndenied 3\nunclassified 2\n",
    "expand_modeled 2\nexpand_modeled_with_list_source 1\nexpand_needs_overlay 2\n",
    "expand_needs_overlay_not_in_resource 1\nexpand_needs_overlay_required_member_not_identifier 1\n",
    "ambiguous_resource 1\nlist_describe 13\nlist_describe_denied 1\nlist_describe_paginated 3\n",
    "list_describe_without_paginator 10\nunpaginated 6\npagination_needs_overlay 4\n",
    "pagination_needs_overlay_token_both_sides 1\npagination_needs_overlay_token_input_only 1\n",
    "pagination_needs_overlay_token_output_only 1\npagination_needs_overlay_truncation_flag 1\n",
    "paginated 3\nservice_tokens_completed 1\npage_size_known 1\npage_size_unbounded 1\n",
    "paginated_without_page_size 1\nresources 4\nresources_with_list 1\nresources_with_read 4\n",
    "resources_with_list_and_read 1\nresources_with_arn 1\nbatch_reads 2\nbatch_reads_with_single_counterpart 1\n",
    "token_pairs 1\nservice alpha 8 4 2 2 3 6 4\n"
);

pub fn rich_model() -> Result<Value, Box<dyn std::error::Error>> {
    let mut value = paginated();
    replace(
        &mut value,
        "/shapes/n#Service/traits/smithy.api#paginated",
        json!({
        "inputToken":"Cursor","outputToken":"Envelope.Token","items":"Envelope.Items"}),
    )?;
    replace(
        &mut value,
        "/shapes/n#ListRoots/traits/smithy.api#paginated",
        json!({"inputToken":"Next","pageSize":"Limit"}),
    )?;
    let additions = additions();
    let shapes = value
        .get_mut("shapes")
        .and_then(Value::as_object_mut)
        .ok_or("shapes missing")?;
    shapes.extend(additions);
    Ok(value)
}

fn additions() -> serde_json::Map<String, Value> {
    let mut shapes = page_and_input_shapes();
    shapes.extend(resource_and_other_shapes());
    shapes
}

fn page_and_input_shapes() -> serde_json::Map<String, Value> {
    let shapes = json!({
        "n#ListUnbounded":{"type":"operation","input":{"target":"n#Unbounded"},"output":{"target":"n#Output"},
            "traits":{"smithy.api#paginated":{"inputToken":"Next","outputToken":"Envelope.Token","pageSize":"Limit"}}},
        "n#ListNoSize":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"},
            "traits":{"smithy.api#paginated":{"inputToken":"Next","outputToken":"Envelope.Token"}}},
        "n#Unbounded":{"type":"structure","members":{"Next":{"target":"smithy.api#String"},"Limit":{"target":"smithy.api#Integer"}}},
        "n#DescribeChild":{"type":"operation","input":{"target":"n#Parent"}},
        "n#DescribeNoList":{"type":"operation","input":{"target":"n#Parent"}},
        "n#DescribeNoResource":{"type":"operation","input":{"target":"n#Parent"}},
        "n#DescribeUnbound":{"type":"operation","input":{"target":"n#RequiredFilter"}},
        "n#Parent":{"type":"structure","members":{"Parent":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}}}},
        "n#RequiredFilter":{"type":"structure","members":{"Filter":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}}}},
    });
    object(shapes)
}

fn resource_and_other_shapes() -> serde_json::Map<String, Value> {
    let shapes = json!({
        "n#Resource":{"type":"resource","identifiers":{"Parent":{"target":"smithy.api#String"}},
            "read":{"target":"n#DescribeChild"},"list":{"target":"n#ListRoots"},"traits":{"aws.api#arn":"arn:test:{Parent}"}},
        "n#ZResource":{"type":"resource","identifiers":{"Parent":{"target":"smithy.api#String"}},"read":{"target":"n#DescribeChild"}},
        "n#MissingList":{"type":"resource","identifiers":{"Parent":{"target":"smithy.api#String"}},"read":{"target":"n#DescribeNoList"}},
        "n#Unbound":{"type":"resource","identifiers":{"Parent":{"target":"smithy.api#String"}},"read":{"target":"n#DescribeUnbound"}},
        "n#ListPlain":{"type":"operation"},"n#ListBlocked":{"type":"operation"},
        "n#ListBoth":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#TokenOutput"}},
        "n#ListInput":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#FlagOutput"}},
        "n#ListOutput":{"type":"operation","output":{"target":"n#TokenOutput"}},
        "n#ListFlag":{"type":"operation","output":{"target":"n#FlagOutput"}},
        "n#TokenOutput":{"type":"structure","members":{"Token":{"target":"smithy.api#String"},"IsTruncated":{"target":"smithy.api#Boolean"}}},
        "n#FlagOutput":{"type":"structure","members":{"IsTruncated":{"target":"smithy.api#Boolean"}}},
        "n#PutThing":{"type":"operation"},"n#GetThing":{"type":"operation"},
        "n#BatchGetThings":{"type":"operation"},"n#BatchDescribeMissing":{"type":"operation"}
    });
    object(shapes)
}

fn object(shapes: Value) -> serde_json::Map<String, Value> {
    match shapes {
        Value::Object(shapes) => shapes,
        _ => serde_json::Map::new(),
    }
}

pub fn fixture() -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    write_model(dir.path(), "alpha", &rich_model()?)?;
    super_write(
        dir.path(),
        "catalog/overlay/must-deny.txt",
        "ListBlocked\tcontent\n",
    )?;
    Ok(dir)
}

pub fn assert_fixture_report() -> TestResult {
    let dir = fixture()?;
    assert_eq!(
        catalog_tool::plans::build_report(dir.path(), dir.path())?,
        EXPECTED
    );
    Ok(())
}
