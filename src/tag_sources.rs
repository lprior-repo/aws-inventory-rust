use anyhow::Result;
use inventory_core::catalog::runtime::{RuntimeCatalog, kinds_without_tag_source};
use serde_json::{Value, json};

const CATALOG: &[u8] = include_bytes!("../catalog/aws-api.catalog");

pub fn report() -> Result<Value> {
    let catalog = RuntimeCatalog::from_bytes(CATALOG)?;
    let missing = kinds_without_tag_source(&catalog);
    let kinds: Vec<&str> = missing.iter().map(|kind| kind.as_str()).collect();
    Ok(json!({"kinds_without_tag_source": kinds}))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unsourced() -> Result<Vec<String>> {
        let value = report()?;
        Ok(serde_json::from_value(
            value
                .get("kinds_without_tag_source")
                .cloned()
                .unwrap_or(serde_json::Value::Null),
        )?)
    }

    #[test]
    fn acm_certificate_is_sourced_by_its_list_tags_endpoint() -> Result<()> {
        let catalog = RuntimeCatalog::from_bytes(CATALOG)?;
        assert!(
            catalog
                .schemas()
                .iter()
                .any(|schema| schema.kind().as_str() == "AWS::CertificateManager::Certificate"),
            "precondition: the ACM certificate kind is in the compiled catalog"
        );
        let kinds = unsourced()?;
        assert!(
            !kinds
                .iter()
                .any(|kind| kind == "AWS::CertificateManager::Certificate"),
            "ACM certificate is sourced via ListTagsForCertificate"
        );
        Ok(())
    }

    #[test]
    fn kind_without_any_tag_call_is_reported() -> Result<()> {
        let kinds = unsourced()?;
        assert!(
            kinds
                .iter()
                .any(|kind| kind == "AWS::ACMPCA::CertificateAuthority"),
            "ACM PCA ListTags has no approved binding, so no tag source reaches it"
        );
        Ok(())
    }
}
