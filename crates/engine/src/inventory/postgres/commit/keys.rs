use inventory_core::inventory::{AccountId, Kind, Region, ResourceId, ResourceKey};
use sqlx::{Postgres, QueryBuilder, Row, postgres::PgRow};

pub(in crate::inventory::postgres) struct Keys<'a> {
    pub accounts: Vec<&'a str>,
    pub regions: Vec<&'a str>,
    pub kinds: Vec<&'a str>,
    pub ids: Vec<&'a str>,
}

impl<'a> Keys<'a> {
    pub(in crate::inventory::postgres) fn new(keys: impl Iterator<Item = &'a ResourceKey>) -> Self {
        let (lower, upper) = keys.size_hint();
        let capacity = match upper {
            Some(upper) => upper,
            None => lower,
        };
        let mut arrays = Self {
            accounts: Vec::with_capacity(capacity),
            regions: Vec::with_capacity(capacity),
            kinds: Vec::with_capacity(capacity),
            ids: Vec::with_capacity(capacity),
        };
        for key in keys {
            arrays.accounts.push(key.account.as_str());
            arrays.regions.push(key.region.as_str());
            arrays.kinds.push(key.kind.as_str());
            arrays.ids.push(key.id.as_str());
        }
        arrays
    }

    pub(in crate::inventory::postgres) fn bind(&self, query: &mut QueryBuilder<Postgres>) {
        query.push("UNNEST(");
        self.bind_arrays(query);
        query.push(")");
    }

    pub(in crate::inventory::postgres) fn bind_arrays(&self, query: &mut QueryBuilder<Postgres>) {
        query
            .push_bind(&self.accounts)
            .push("::text[],")
            .push_bind(&self.regions)
            .push("::text[],")
            .push_bind(&self.kinds)
            .push("::text[],")
            .push_bind(&self.ids)
            .push("::text[]");
    }
}

pub(in crate::inventory::postgres) fn decode(
    row: &PgRow,
) -> Result<ResourceKey, crate::inventory::StoreError> {
    Ok(ResourceKey {
        account: AccountId::parse(row.try_get("account")?)?,
        region: Region::parse(row.try_get("region")?)?,
        kind: Kind::parse(row.try_get("kind")?)?,
        id: ResourceId::parse(row.try_get("id")?)?,
    })
}
