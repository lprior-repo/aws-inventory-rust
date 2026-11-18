use crate::kind_map::ApiOnlyMapping;

pub(super) const ENTRIES: &[ApiOnlyMapping<'static>] = &[
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::GameLiftStreamsApplication",
        reason: "Pinned CFN registry has no GameLift Streams application type; API summary identifies the application ARN.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::GameLiftStreamsStreamGroup",
        reason: "Pinned CFN registry has no GameLift Streams stream group type; API summary identifies the stream group ARN.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::GlacierVault",
        reason: "Pinned CFN registry has no Glacier vault resource type; vault metadata has its own ARN.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::GlobalAcceleratorCustomRoutingAccelerator",
        reason: "Custom routing accelerator is not the standard-routing CFN Accelerator; pinned registry has no corresponding custom-routing type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::GlobalAcceleratorCustomRoutingListener",
        reason: "Custom routing listener is a child of a custom routing accelerator and is distinct from the standard-routing CFN Listener; no corresponding pinned CFN custom-routing type exists.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::GlobalAcceleratorCustomRoutingEndpointGroup",
        reason: "Custom routing endpoint group is a child of a custom routing listener and is distinct from the standard-routing CFN EndpointGroup; no corresponding pinned CFN custom-routing type exists.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::GreengrassComponent",
        reason: "Component ARN is unversioned; CFN ComponentVersion represents a versioned ARN, collected separately via ListComponentVersions.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::GreengrassCoreDevice",
        reason: "Greengrass core-device registration is not an IoT Thing or CFN Deployment; pinned registry has no core-device type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::GroundStationSatellite",
        reason: "Registered satellite is not a CFN GroundStation Config; pinned registry has no satellite resource type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::IdentityStore",
        reason: "Identity store container has no pinned CFN type; CFN IdentityStore User/Group types represent children, not the store.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::IoTManagedIntegrationsCloudConnector",
        reason: "Managed integrations cloud connector is distinct from Greengrass ConnectorDefinition; no corresponding pinned CFN type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::IoTManagedIntegrationsConnectorDestination",
        reason: "Managed integrations connector destination is distinct from IoTWireless Destination; no corresponding pinned CFN type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::IoTManagedIntegrationsCredentialLocker",
        reason: "Lists locker identifiers and metadata, not credential contents; no corresponding type in pinned CFN registry.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::IoTManagedIntegrationsManagedThing",
        reason: "Managed integrations device is distinct from an IoT Thing; no corresponding type in pinned CFN registry.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::IoTManagedIntegrationsProvisioningProfile",
        reason: "Managed integrations provisioning profile is distinct from IoT ProvisioningTemplate; no corresponding pinned CFN type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::IoTThingsGraphSystemInstance",
        reason: "Deployed system instance is distinct from a FlowTemplate; pinned CFN registry has no system-instance resource type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::IoTThingsGraphSystemTemplate",
        reason: "System template is distinct from a FlowTemplate; pinned CFN registry has no system-template resource type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::KeyspacesStream",
        reason: "CDC stream is a table data-capture attachment, not a standalone pinned CFN Cassandra Table or Kinesis Stream type.",
    },
];
