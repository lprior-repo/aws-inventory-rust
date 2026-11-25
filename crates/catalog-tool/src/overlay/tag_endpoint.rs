use super::{
    ItemSelector, MemberPath, NameError, OperationName, OperationOverlay, OverlayClass, Paginator,
    Scope,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagEndpoint {
    pub parents: Vec<TagEndpointParent>,
    pub selector: TagEndpointSelector,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagEndpointParent {
    pub operation: OperationName,
    pub request: TagEndpointRequest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagEndpointRequest {
    None,
    Member {
        input: MemberPath,
        parent: MemberPath,
    },
    Context {
        input: MemberPath,
        parent: MemberPath,
    },
    Owner {
        input: MemberPath,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagEndpointSelector {
    Map {
        path: MemberPath,
    },
    List {
        path: MemberPath,
        key: MemberPath,
        value: MemberPath,
    },
}

impl TagEndpoint {
    pub fn operation(
        self,
        scope: Scope,
        paginator: Paginator,
    ) -> Result<OperationOverlay, NameError> {
        Ok(OperationOverlay {
            class: Some(OverlayClass::Expand),
            scope: Some(scope),
            items: Some(ItemSelector::Root),
            paginator: Some(paginator),
            tag_endpoint: Some(self),
            ..OperationOverlay::default()
        })
    }
}
