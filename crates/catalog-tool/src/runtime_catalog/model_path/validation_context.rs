use crate::kind_map::KindMap;
use crate::overlay::CheckedOverlays;
use crate::runtime_catalog::ServiceIndex;

pub(in crate::runtime_catalog) struct ValidationContext<'p, 'a> {
    pub kinds: &'p KindMap,
    pub checked: &'p CheckedOverlays,
    pub services: &'p ServiceIndex<'a>,
}
