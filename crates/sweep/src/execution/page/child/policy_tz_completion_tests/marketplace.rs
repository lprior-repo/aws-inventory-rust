use super::*;
use minicbor::{Decoder, data::Type};

#[tokio::test]
async fn given_agreement_search_when_fake_transport_pages_then_required_filter_survives_resume()
-> TestResult {
    // Given: the actual pinned model and reviewed search overlay, without an emulator.
    let compiled = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&compiled)?;
    let scenario = seeded(&catalog).await?;
    // When: a continuation page is decoded, committed, reopened and requested.
    let (scenario, seen) = scenario
        .drain(&catalog, "marketplace-agreement", |_, _, wire, resumed| {
            respond(wire, resumed)
        })
        .await?;
    // Then: both requests retain the filter and both canonical account-global IDs persist.
    assert_eq!(seen.get("SearchAgreements"), Some(&2));
    assert_agreements(&scenario).await?;
    assert_complete(&scenario).await
}

#[tokio::test]
async fn given_empty_agreement_search_when_fake_transport_returns_no_matches_then_scope_completes()
-> TestResult {
    let compiled = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&compiled)?;
    let scenario = seeded(&catalog).await?;
    let (scenario, seen) = scenario
        .drain(&catalog, "marketplace-agreement", |_, _, wire, resumed| {
            assert!(!resumed);
            assert_input(wire, resumed)?;
            Ok(json!({"agreementViewSummaries":[]}))
        })
        .await?;
    assert_eq!(seen.get("SearchAgreements"), Some(&1));
    scenario
        .assert_ids_at("AWS::ApiOnly::MarketplaceAgreement", "global", vec![])
        .await?;
    assert_complete(&scenario).await
}

fn respond(wire: &http::Request<bytes::Bytes>, resumed: bool) -> TestResult<Json> {
    assert_input(wire, resumed)?;
    let id = if resumed {
        "agreement-two"
    } else {
        "agreement-one"
    };
    Ok(page(
        "agreementViewSummaries",
        json!({"agreementId":id}),
        !resumed,
        "nextToken",
    ))
}

async fn assert_agreements(scenario: &Scenario<'_, '_>) -> TestResult {
    scenario
        .assert_ids_at(
            "AWS::ApiOnly::MarketplaceAgreement",
            "global",
            vec!["agreement-one".into(), "agreement-two".into()],
        )
        .await
}

fn compiled() -> TestResult<Vec<u8>> {
    Ok(crate::consumer_fixture::compile(
        "marketplace-agreement",
        "2020-03-01",
        &["SearchAgreements"],
    )?
    .bytes)
}

async fn seeded<'a, 'catalog>(catalog: &'a RuntimeCatalog<'catalog>) -> TestResult<Scenario<'a, 'catalog>> {
    let scenario = Scenario::new(catalog, "marketplace-required-filter").await?;
    scenario.seed(catalog, "marketplace-agreement", &["SearchAgreements"]).await?;
    Ok(scenario)
}

async fn assert_complete(scenario: &Scenario<'_, '_>) -> TestResult {
    let summary = scenario.store.sweep_summary(&scenario.sweep).await?;
    assert_eq!((summary.complete_scopes, summary.failed_scopes), (1, 0));
    Ok(())
}

fn assert_input(wire: &http::Request<bytes::Bytes>, resumed: bool) -> TestResult {
    assert_eq!(wire.method(), http::Method::POST);
    assert_eq!(
        wire.uri().path(),
        "/service/AWSMPCommerceService_v20200301/operation/SearchAgreements"
    );
    assert_eq!(
        wire.headers().get("smithy-protocol"),
        Some(&http::HeaderValue::from_static("rpc-v2-cbor"))
    );
    let mut expected = json!({"maxResults":50,"filters":[{"name":"AgreementType","values":["PurchaseAgreement"]}]});
    if resumed {
        expected
            .as_object_mut()
            .ok_or("search input expectation is not an object")?
            .insert("nextToken".to_owned(), json!("page-two"));
    }
    assert_eq!(decode_input(wire.body())?, expected);
    Ok(())
}

fn decode_input(body: &[u8]) -> TestResult<Json> {
    let mut decoder = Decoder::new(body);
    assert_eq!(decoder.map()?, None);
    let mut fields = serde_json::Map::new();
    while decoder.datatype()? != Type::Break {
        let name = decoder.str()?.to_owned();
        let value = match name.as_str() {
            "filters" => decode_filter(&mut decoder)?,
            "maxResults" => json!(decoder.u64()?),
            "nextToken" => json!(decoder.str()?),
            _ => return Err("unexpected search input member".into()),
        };
        assert_eq!(fields.insert(name, value), None);
    }
    decoder.skip()?;
    assert_eq!(decoder.position(), body.len());
    Ok(Json::Object(fields))
}

fn decode_filter(decoder: &mut Decoder<'_>) -> TestResult<Json> {
    assert_eq!(decoder.array()?, None);
    assert_eq!(decoder.map()?, None);
    let mut fields = serde_json::Map::new();
    while decoder.datatype()? != Type::Break {
        let name = decoder.str()?.to_owned();
        let value = match name.as_str() {
            "name" => json!(decoder.str()?),
            "values" => {
                assert_eq!(decoder.array()?, None);
                let value = json!([decoder.str()?]);
                assert_eq!(decoder.datatype()?, Type::Break);
                decoder.skip()?;
                value
            }
            _ => return Err("unexpected filter member".into()),
        };
        assert_eq!(fields.insert(name, value), None);
    }
    decoder.skip()?;
    assert_eq!(decoder.datatype()?, Type::Break);
    decoder.skip()?;
    Ok(json!([fields]))
}
