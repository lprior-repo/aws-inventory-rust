use super::lease::trusted_url;
use super::*;
#[test]
fn service_discovery_never_accepts_url_authority_injection() {
    let suffix = ".timestream.us-east-1.amazonaws.com";
    assert!(trusted_url("ingest-cell1.timestream.us-east-1.amazonaws.com", suffix).is_ok());
    for address in [
        "http://ingest-cell1.timestream.us-east-1.amazonaws.com",
        "ingest-cell1.timestream.us-east-1.amazonaws.com.evil.test",
        "127.0.0.1",
        "user@ingest-cell1.timestream.us-east-1.amazonaws.com",
        "ingest-cell1.timestream.us-east-1.amazonaws.com:443",
        "ingest-cell1.timestream.us-west-2.amazonaws.com",
    ] {
        assert!(trusted_url(address, suffix).is_err(), "{address}");
    }
}
#[tokio::test]
async fn leases_expire_and_are_service_account_region_isolated()
-> Result<(), Box<dyn std::error::Error>> {
    let cache = EndpointCache::default();
    let now = Instant::now();
    let key = (
        "timestream-write".to_owned(),
        "123456789012".to_owned(),
        "us-east-1".to_owned(),
    );
    cache
        .insert(
            key.clone(),
            "https://ingest-cell1.timestream.us-east-1.amazonaws.com".to_owned(),
            Duration::from_secs(60),
            now,
        )
        .await?;
    assert!(cache.get(&key, now).await.is_some());
    assert!(
        cache
            .get(
                &key,
                now.checked_add(Duration::from_secs(60))
                    .ok_or("clock overflow")?
            )
            .await
            .is_none()
    );
    let query = ("timestream-query".to_owned(), key.1.clone(), key.2.clone());
    assert!(cache.get(&query, now).await.is_none());
    let account = (key.0.clone(), "999999999999".to_owned(), key.2.clone());
    assert!(cache.get(&account, now).await.is_none());
    let region = (key.0.clone(), key.1.clone(), "us-west-2".to_owned());
    assert!(cache.get(&region, now).await.is_none());
    Ok(())
}
#[test]
fn endpoint_leases_require_bounded_count_ttl_and_every_trusted_address()
-> Result<(), Box<dyn std::error::Error>> {
    use inventory_core::record::{FieldId, Key, Value};
    let endpoints = FieldId::new(1)?;
    let address = FieldId::new(2)?;
    let minutes = FieldId::new(3)?;
    let suffix = ".timestream.us-east-1.amazonaws.com";
    let item = |host: &str, ttl| {
        Value::Map(BTreeMap::from([
            (Key::Field(address), Value::Text(host.to_owned())),
            (Key::Field(minutes), Value::Int(ttl)),
        ]))
    };
    let response = |items| {
        Record::encode(&Value::Map(BTreeMap::from([(
            Key::Field(endpoints),
            Value::List(items),
        )])))
    };
    let valid = "ingest-cell1.timestream.us-east-1.amazonaws.com";
    let record = response(vec![item(valid, 60)])?;
    assert_eq!(
        lease(&record, &[endpoints], &[address], &[minutes], suffix)?,
        (format!("https://{valid}"), Duration::from_secs(3600))
    );
    for ttl in [-1, 0, 1441, i64::MAX] {
        let record = response(vec![item(valid, ttl)])?;
        assert!(lease(&record, &[endpoints], &[address], &[minutes], suffix).is_err());
    }
    for count in [0, 17] {
        let record = response((0..count).map(|_| item(valid, 60)).collect())?;
        assert!(lease(&record, &[endpoints], &[address], &[minutes], suffix).is_err());
    }
    let record = response(vec![item(valid, 60), item("evil.example", 60)])?;
    assert!(lease(&record, &[endpoints], &[address], &[minutes], suffix).is_err());
    Ok(())
}
