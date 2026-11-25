// Pinned model: .local/aws-models/models/schemas/service/2019-12-02/schemas-2019-12-02.json
use super::super::construction::{Input, Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListDiscoverers",
        items: "Discoverers",
        identity: ResourceIdentity::Arn("DiscovererArn"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[],
        tags: Some("Tags"),
        queryable: &[],
    },
    Inventory {
        operation: "ListRegistries",
        items: "Registries",
        identity: ResourceIdentity::Arn("RegistryArn"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[Input::String("Scope", "Local")],
        tags: Some("Tags"),
        queryable: &["RegistryName"],
    },
    Inventory {
        operation: "ListSchemas",
        items: "Schemas",
        identity: ResourceIdentity::Arn("SchemaArn"),
        token: Some(("NextToken", "NextToken")),
        scope: Scope::Region,
        input: &[Input::Parent(
            "RegistryName",
            "ListRegistries",
            "RegistryName",
        )],
        tags: Some("Tags"),
        queryable: &[],
    },
];
