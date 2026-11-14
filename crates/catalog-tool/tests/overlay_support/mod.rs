use catalog_tool::overlay::{MemberPath, OperationName, OverlayError, Rule, ServiceOverlay};

pub const HEADER: &str = "format = 1\nreviewed = '2026-11-13'\nreviewer = 'name'\n";
pub const EXAMPLE: &str = include_str!("../fixtures/overlay/complete.toml");
pub type TestResult = Result<(), Box<dyn std::error::Error>>;

pub fn parse(body: &str) -> Result<ServiceOverlay, OverlayError> {
    ServiceOverlay::parse("ec2", &format!("{HEADER}{body}"))
}
pub fn path(s: &str) -> Result<MemberPath, catalog_tool::overlay::NameError> {
    MemberPath::new(s)
}
pub fn name(s: &str) -> Result<OperationName, catalog_tool::overlay::NameError> {
    OperationName::new(s)
}
pub fn error(text: &str, operation: Option<&str>, key: &str, value: &str, rule: Rule) {
    let result = ServiceOverlay::parse("ec2", text).err();
    let actual = result.map(|e| match e {
        OverlayError::Rule {
            service,
            operation,
            key,
            value,
            rule,
        } => (service, operation, key, value.into_string(), rule),
        other => (
            other.to_string(),
            None,
            String::new(),
            String::new(),
            Rule::Resource,
        ),
    });
    assert_eq!(
        actual,
        Some((
            "ec2".to_owned(),
            operation.map(str::to_owned),
            key.to_owned(),
            value.to_owned(),
            rule
        ))
    );
}
pub fn reject(body: &str, key: &str, value: &str, rule: Rule) {
    error(
        &format!("{HEADER}[operation.ListThings]\n{body}"),
        Some("ListThings"),
        key,
        value,
        rule,
    );
}
pub fn write(root: &std::path::Path, relative: &str, text: &str) -> std::io::Result<()> {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, text)
}

pub mod example_root;
