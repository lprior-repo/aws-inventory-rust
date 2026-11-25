use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "AssociateGatewayToServer",
            deferred(
                "Model requires GatewayArn, ServerArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates a backup gateway with your server. After you complete the association process, you can back up and restore your VMs through the gateway.",
            ),
        ),
        (
            "CreateGateway",
            deferred(
                "Model requires ActivationKey, GatewayDisplayName, GatewayType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a backup gateway. After you create a gateway, you can associate it with a server using the AssociateGatewayToServer operation.",
            ),
        ),
        (
            "DeleteGateway",
            deferred(
                "Model requires GatewayArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a backup gateway.",
            ),
        ),
        (
            "DeleteHypervisor",
            deferred(
                "Model requires HypervisorArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a hypervisor.",
            ),
        ),
        (
            "DisassociateGatewayFromServer",
            deferred(
                "Model requires GatewayArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Disassociates a backup gateway from the specified server. After the disassociation process finishes, the gateway can no longer access the virtual machines on the server.",
            ),
        ),
        (
            "GetBandwidthRateLimitSchedule",
            deferred(
                "Model requires GatewayArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the bandwidth rate limit schedule for a specified gateway. By default, gateways do not have bandwidth rate limit schedules, which means no bandwidth rate limiting is in effect. Use this to get a gateway's bandwidth rate…",
            ),
        ),
        (
            "GetGateway",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("GatewayArn")?,
                    Binding::From {
                        operation: name("ListGateways")?,
                        path: path("GatewayArn")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("GatewayArn")?],
                ..read(
                    "Gateway",
                    Identity::Arn(path("GatewayArn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "GetHypervisor",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("HypervisorArn")?,
                    Binding::From {
                        operation: name("ListHypervisors")?,
                        path: path("HypervisorArn")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("HypervisorArn")?],
                ..read(
                    "Hypervisor",
                    Identity::Arn(path("HypervisorArn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "GetHypervisorPropertyMappings",
            deferred(
                "Model requires HypervisorArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This action retrieves the property mappings for the specified hypervisor. A hypervisor property mapping displays the relationship of entity properties available from the hypervisor to the properties available in Amazon Web…",
            ),
        ),
        (
            "GetVirtualMachine",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("ResourceArn")?,
                    Binding::From {
                        operation: name("ListVirtualMachines")?,
                        path: path("ResourceArn")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("ResourceArn")?],
                ..read(
                    "VirtualMachine",
                    Identity::Arn(path("ResourceArn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "ImportHypervisorConfiguration",
            deferred(
                "Model requires Name, Host. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Connect to a hypervisor by importing its configuration.",
            ),
        ),
        (
            "ListGateways",
            OperationOverlay {
                queryable: vec![path("GatewayArn")?],
                ..read(
                    "Gateways",
                    Identity::Arn(path("GatewayArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListHypervisors",
            OperationOverlay {
                queryable: vec![path("HypervisorArn")?],
                ..read(
                    "Hypervisors",
                    Identity::Arn(path("HypervisorArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires ResourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the tags applied to the resource identified by its Amazon Resource Name (ARN).",
            ),
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "ListVirtualMachines",
            OperationOverlay {
                queryable: vec![path("ResourceArn")?],
                ..read(
                    "VirtualMachines",
                    Identity::Arn(path("ResourceArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "PutBandwidthRateLimitSchedule",
            deferred(
                "Model requires GatewayArn, BandwidthRateLimitIntervals. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This action sets the bandwidth rate limit schedule for a specified gateway. By default, gateways do not have a bandwidth rate limit schedule, which means no bandwidth rate limiting is in effect. Use this to initiate a gateway's…",
            ),
        ),
        (
            "PutHypervisorPropertyMappings",
            deferred(
                "Model requires HypervisorArn, VmwareToAwsTagMappings, IamRoleArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This action sets the property mappings for the specified hypervisor. A hypervisor property mapping displays the relationship of entity properties available from the hypervisor to the properties available in Amazon Web Services.",
            ),
        ),
        (
            "PutMaintenanceStartTime",
            deferred(
                "Model requires GatewayArn, HourOfDay, MinuteOfHour. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Set the maintenance start time for a gateway.",
            ),
        ),
        (
            "StartVirtualMachinesMetadataSync",
            deferred(
                "Model requires HypervisorArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This action sends a request to sync metadata across the specified virtual machines.",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires ResourceARN, Tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Tag the resource.",
            ),
        ),
        (
            "TestHypervisorConfiguration",
            deferred(
                "Model requires GatewayArn, Host. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Tests your hypervisor configuration to validate that backup gateway can connect with the hypervisor and its resources.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires ResourceARN, TagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes tags from the resource.",
            ),
        ),
        (
            "UpdateGatewayInformation",
            deferred(
                "Model requires GatewayArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a gateway's name. Specify which gateway to update using the Amazon Resource Name (ARN) of the gateway in your request.",
            ),
        ),
        (
            "UpdateGatewaySoftwareNow",
            deferred(
                "Model requires GatewayArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the gateway virtual machine (VM) software. The request immediately triggers the software update. When you make this request, you get a 200 OK success response immediately. However, it might take some time for the update…",
            ),
        ),
        (
            "UpdateHypervisor",
            deferred(
                "Model requires HypervisorArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a hypervisor metadata, including its host, username, and password. Specify which hypervisor to update using the Amazon Resource Name (ARN) of the hypervisor in your request.",
            ),
        ),
    ])
}
