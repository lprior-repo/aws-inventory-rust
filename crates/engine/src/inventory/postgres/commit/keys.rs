use inventory_core::inventory::ResourceKey;
use sqlx::{Postgres, QueryBuilder};

pub(super) struct Keys<'a> {
    pub accounts: Vec<&'a str>,
    pub regions: Vec<&'a str>,
    pub kinds: Vec<&'a str>,
    pub ids: Vec<&'a str>,
}

impl<'a> Keys<'a> {
    pub(super) fn new(keys: impl Iterator<Item = &'a ResourceKey>) -> Self {
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

    pub(super) fn bind(&self, query: &mut QueryBuilder<Postgres>) {
        query.push("UNNEST(");
        self.bind_arrays(query);
        query.push(")");
    }

    pub(super) fn bind_arrays(&self, query: &mut QueryBuilder<Postgres>) {
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
