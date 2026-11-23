use super::*;
mod endpoints;

fn table_arn(id: &str) -> String {
    format!("arn:aws:cleanrooms:us-east-1:123456789012:configuredtable/{id}")
}
fn membership_arn(id: &str) -> String {
    format!("arn:aws:cleanrooms:us-east-1:123456789012:membership/{id}")
}

