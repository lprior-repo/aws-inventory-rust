use std::{io, path::Path};

pub const ALPHA: &str = r#"{"smithy":"2.0","shapes":{
 "n#Service":{"type":"service"},
 "n#ListRoots":{"type":"operation","traits":{"smithy.api#readonly":{}}},
 "n#DescribeChild":{"type":"operation","input":{"target":"n#Required"}},
 "n#Required":{"type":"structure","members":{"Parent":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}}}},
 "n#GetSecret":{"type":"operation","traits":{"smithy.api#readonly":{}}},
 "n#PutThing":{"type":"operation","traits":{"smithy.api#readonly":{}}},
 "n#GetOther":{"type":"operation","traits":{"smithy.api#readonly":{}}},
 "n#Unrelated":{"type":"union","members":{"Ignored":{"traits":{"smithy.api#required":{}}}}}
}}"#;

pub const BETA: &str = r#"{"smithy":"2.0","shapes":{
 "z#Service":{"type":"service"},
 "z#ListBlocked":{"type":"operation","traits":{"smithy.api#readonly":{}}},
 "z#DescribeEmpty":{"type":"operation","input":{"target":"z#Optional"}},
 "z#Optional":{"type":"structure","members":{"Filter":{"target":"smithy.api#String"}}},
 "z#GetPaid":{"type":"operation"},
 "z#GetId":{"type":"operation"},
 "z#DescribeReference":{"type":"operation"}
}}"#;

pub const DENY: &str = concat!(
    "GetId\tmutating\nGetPaid\tpaid\nGetSecret\tsecret\nListBlocked\tcontent\n",
    "beta/DescribeReference\treference-data\n"
);

pub fn policy()
-> Result<catalog_tool::classes::DenyList<'static>, catalog_tool::classes::ParseError> {
    catalog_tool::classes::DenyList::parse(DENY)
}

pub const REPORT: &str = concat!(
    "# enumerate and expand are candidates; ",
    "callable only with a reviewed service overlay (VISION 5.1).\n",
    "operations 10\nenumerate 2\nexpand 1\ndenied 6\nunclassified 1\n",
    "denied content 1\ndenied secret 1\ndenied paid 1\ndenied mutating 2\n",
    "denied reference-data 1\nno_read_verb 1\nreadonly_denied 3\n",
    "readonly_unclassified 1\nmust_deny_entries 5\n",
    "service alpha 1 1 2 1\nservice beta 1 0 4 0\n"
);

pub fn write(root: &Path, relative: &str, text: &str) -> io::Result<()> {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, text)
}

pub fn fixture() -> io::Result<tempfile::TempDir> {
    let dir = tempfile::tempdir()?;
    write(dir.path(), "models/beta/service/v1/beta.json", BETA)?;
    write(dir.path(), "models/alpha/service/v1/alpha.json", ALPHA)?;
    Ok(dir)
}
