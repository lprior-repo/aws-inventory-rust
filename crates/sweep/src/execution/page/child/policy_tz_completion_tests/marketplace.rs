use super::*;
use minicbor::{Decoder, data::Type};

#[tokio::test]
async fn given_agreement_search_when_fake_transport_pages_then_required_filter_survives_resume()
-> TestResult {
    let compiled = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&compiled)?;
    let scenario = seeded(&catalog).await?;
    let (scenario, seen) = scenario
        .drain(&catalog, "marketplace-agreement", |_, _, wire, resumed| {
            respond(wire, resumed)
        })
        .await?;
    assert_eq!(seen.get("SearchAgreements"), Some(&4));
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
    assert_eq!(seen.get("SearchAgreements"), Some(&2));
    scenario
        .assert_ids_at("AWS::ApiOnly::MarketplaceAgreement", "global", vec![])
        .await?;
    assert_complete(&scenario).await
}

fn respond(wire: &http::Request<bytes::Bytes>, resumed: bool) -> TestResult<Json> {
    let party = assert_input(wire, resumed)?;
    let id = format!("{party}-{}", if resumed { "two" } else { "one" });
    let mut response = json!({"agreementViewSummaries":[{"agreementId":id}]});
    if !resumed {
        response["nextToken"] = json!(format!("{party}-next"));
    }
    Ok(response)
}

async fn assert_agreements(scenario: &Scenario<'_, '_>) -> TestResult {
    scenario
        .assert_ids_at(
            "AWS::ApiOnly::MarketplaceAgreement",
            "global",
            ["Acceptor-one", "Acceptor-two", "Proposer-one", "Proposer-two"].map(str::to_owned).into(),
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

async fn seeded<'a, 'catalog>(
    catalog: &'a RuntimeCatalog<'catalog>,
) -> TestResult<Scenario<'a, 'catalog>> {
    let scenario = Scenario::new(catalog, "marketplace-required-filter").await?;
    let plan = catalog.lookup("marketplace-agreement", "SearchAgreements").ok_or("missing SearchAgreements")?;
    let mut work = Vec::new();
    crate::execution::draining::append_plan_work(
        &scenario.account, &scenario.home, &scenario.sweep, plan, &[Region::Global], &mut work,
    )?;
    scenario.store.seed_works(&work).await?;
    Ok(scenario)
}

async fn assert_complete(scenario: &Scenario<'_, '_>) -> TestResult {
    let summary = scenario.store.sweep_summary(&scenario.sweep).await?;
    assert_eq!((summary.complete_scopes, summary.failed_scopes), (2, 0));
    Ok(())
}

fn assert_input(wire: &http::Request<bytes::Bytes>, resumed: bool) -> TestResult<&'static str> {
    assert_eq!(wire.method(), http::Method::POST);
    assert_eq!(
        wire.uri().path(),
        "/service/AWSMPCommerceService_v20200301/operation/SearchAgreements"
    );
    assert_eq!(
        wire.headers().get("smithy-protocol"),
        Some(&http::HeaderValue::from_static("rpc-v2-cbor"))
    );
    let actual = decode_input(wire.body())?;
    let party = match actual.pointer("/filters/1/values/0").and_then(Json::as_str) {
        Some("Acceptor") => "Acceptor",
        Some("Proposer") => "Proposer",
        _ => return Err("invalid or missing agreement party filter".into()),
    };
    let mut expected = json!({"maxResults":50,"filters":[
        {"name":"AgreementType","values":["PurchaseAgreement"]},
        {"name":"PartyType","values":[party]}
    ]});
    if resumed {
        expected["nextToken"] = json!(format!("{party}-next"));
    }
    assert_eq!(actual, expected);
    Ok(party)
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
    let mut filters = Vec::new();
    while decoder.datatype()? != Type::Break {
        filters.push(decode_filter_item(decoder)?);
    }
    decoder.skip()?;
    Ok(Json::Array(filters))
}

fn decode_filter_item(decoder: &mut Decoder<'_>) -> TestResult<Json> {
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
    Ok(Json::Object(fields))
}
