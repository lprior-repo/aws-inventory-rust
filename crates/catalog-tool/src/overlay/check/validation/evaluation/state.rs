use super::Violation;
use crate::{
    classes::{Class, DenyList},
    overlay::{Binding, MemberPath, OperationOverlay},
    plans::{Model, Pagination, Vocabulary},
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct State<'a, 'm, 'd> {
    pub(super) service: &'a str,
    pub(super) name: &'a str,
    pub(super) model: &'a Model<'m>,
    pub(super) deny: &'a DenyList<'d>,
    pub(super) vocabulary: &'a Vocabulary,
    pub(super) overlays: &'a BTreeMap<String, OperationOverlay>,
    pub(super) item: OperationOverlay,
    pub(super) class: Class,
    pub(super) operation_id: &'m str,
    pub(super) violations: Vec<Violation>,
    pub(super) effective: &'static str,
    pub(super) candidate: bool,
    pub(super) pagination: Option<Pagination<'m>>,
    pub(super) selected: Option<crate::plans::model::ShapeRef<'m>>,
    pub(super) derived_bindings: BTreeMap<MemberPath, Binding>,
    pub(super) required: Vec<String>,
    pub(super) provided: BTreeSet<String>,
    pub(super) inferred_page_size: Option<i64>,
    pub(super) paginator_valid: bool,
    pub(super) identity_valid: bool,
}

pub(in crate::overlay::check) struct StateArgs<'a, 'm, 'd> {
    pub(in crate::overlay::check) service: &'a str,
    pub(in crate::overlay::check) name: &'a str,
    pub(in crate::overlay::check) item: Option<OperationOverlay>,
    pub(in crate::overlay::check) class: Class,
    pub(in crate::overlay::check) model: &'a Model<'m>,
    pub(in crate::overlay::check) deny: &'a DenyList<'d>,
    pub(in crate::overlay::check) vocabulary: &'a Vocabulary,
    pub(in crate::overlay::check) overlays: &'a BTreeMap<String, OperationOverlay>,
}

impl<'a, 'm, 'd> State<'a, 'm, 'd> {
    pub(super) fn new(args: StateArgs<'a, 'm, 'd>, operation_id: &'m str) -> Self {
        Self {
            service: args.service,
            name: args.name,
            model: args.model,
            deny: args.deny,
            vocabulary: args.vocabulary,
            overlays: args.overlays,
            item: args.item.unwrap_or_else(empty_overlay),
            class: args.class,
            operation_id,
            violations: Vec::new(),
            effective: "unclassified",
            candidate: false,
            pagination: None,
            selected: None,
            derived_bindings: BTreeMap::new(),
            required: Vec::new(),
            provided: BTreeSet::new(),
            inferred_page_size: None,
            paginator_valid: true,
            identity_valid: true,
        }
    }
}

fn empty_overlay() -> OperationOverlay {
    OperationOverlay {
        class: None,
        defer_reason: None,
        scope: None,
        items: None,
        page_size: None,
        exclude_from_hash: Vec::new(),
        skip: Vec::new(),
        quota_group: None,
        queryable: Vec::new(),
        tags: None,
        fixed: BTreeMap::new(),
        paginator: None,
        identity: None,
        bind: BTreeMap::new(),
        batch: None,
    }
}
