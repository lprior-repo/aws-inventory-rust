use super::*;

pub(super) fn assert_seeded_rows(oracle: &TypedSeedOracle, rows: &InventorySnapshot) -> Result<()> {
    ensure!(
        oracle.pairs.len() == 6,
        "live seed must return all six typed VPC/subnet pairs"
    );
    ensure!(
        oracle
            .pairs
            .first()
            .is_some_and(|pair| pair.vpc_id == oracle.vpc_id
                && pair.subnet_id == oracle.subnet_id
                && pair.vpc_key == oracle.vpc_key
                && pair.subnet_key == oracle.subnet_key
                && pair.relationship == oracle.relationship),
        "typed seed primary output must match its first actual resource pair"
    );
    distinct(oracle)?;
    stored(oracle, rows)?;
    kinds(oracle)?;
    scopes(oracle)
}

fn distinct(oracle: &TypedSeedOracle) -> Result<()> {
    let vpcs = oracle
        .pairs
        .iter()
        .map(|pair| &pair.vpc_key)
        .collect::<std::collections::BTreeSet<_>>();
    let subnets = oracle
        .pairs
        .iter()
        .map(|pair| &pair.subnet_key)
        .collect::<std::collections::BTreeSet<_>>();
    ensure!(
        vpcs.len() == oracle.pairs.len() && subnets.len() == oracle.pairs.len(),
        "seeded resource identities must all be distinct"
    );
    Ok(())
}

fn stored(oracle: &TypedSeedOracle, snapshot: &InventorySnapshot) -> Result<()> {
    ensure!(
        oracle.pairs.iter().all(|pair| snapshot
            .resources
            .iter()
            .any(|(key, _)| *key == pair.vpc_key && pair.vpc_id == pair.vpc_key.id)),
        "seeded VPC IDs and scoped keys are absent from SQL"
    );
    ensure!(
        oracle.pairs.iter().all(|pair| snapshot
            .resources
            .iter()
            .any(|(key, _)| *key == pair.subnet_key && pair.subnet_id == pair.subnet_key.id)),
        "seeded subnet IDs and scoped keys are absent from SQL"
    );
    Ok(())
}

fn kinds(oracle: &TypedSeedOracle) -> Result<()> {
    ensure!(
        oracle
            .pairs
            .iter()
            .all(|pair| pair.vpc_key.kind.as_str() == "AWS::EC2::VPC"
                && pair.subnet_key.kind.as_str() == "AWS::EC2::Subnet"
                && pair.relationship.as_str() == "IN_VPC"),
        "seed keys must preserve modeled EC2 kinds and relationship"
    );
    Ok(())
}

fn scopes(oracle: &TypedSeedOracle) -> Result<()> {
    ensure!(
        oracle
            .pairs
            .iter()
            .all(|pair| pair.vpc_key.account == oracle.account_id
                && pair.subnet_key.account == oracle.account_id),
        "seed keys must preserve returned account identity"
    );
    ensure!(
        oracle
            .pairs
            .iter()
            .all(|pair| pair.vpc_key.region == oracle.region
                && pair.subnet_key.region == oracle.region),
        "seed keys must preserve returned region identity"
    );
    Ok(())
}
