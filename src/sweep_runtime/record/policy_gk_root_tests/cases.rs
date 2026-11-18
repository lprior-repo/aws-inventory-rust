use super::*;

pub(super) struct Case {
    pub service: &'static str,
    pub operation: &'static str,
    pub kind: &'static str,
    pub items: &'static str,
    pub token: Option<&'static str>,
    pub ids: [String; 2],
    pub rows: [Json; 2],
    pub fixed: Json,
}

fn case(
    service: &'static str,
    operation: &'static str,
    kind: &'static str,
    items: &'static str,
    identity: &str,
    ids: [String; 2],
    token: Option<&'static str>,
    fixed: Json,
) -> Case {
    let rows = ids.each_ref().map(|id| {
        if identity == "$" {
            json!(id)
        } else {
            json!({identity:id})
        }
    });
    Case {
        service,
        operation,
        kind,
        items,
        token,
        ids,
        rows,
        fixed,
    }
}
fn names() -> [String; 2] {
    ["unattached-first".to_owned(), "unattached-later".to_owned()]
}
fn arns(service: &str, resource: &str) -> [String; 2] {
    [
        "00000000-0000-0000-0000-000000000001",
        "00000000-0000-0000-0000-000000000002",
    ]
    .map(|id| format!("arn:aws:{service}:us-east-1:123456789012:{resource}/{id}"))
}

pub(super) fn all() -> Vec<Case> {
    let mut scripts = case(
        "gamelift",
        "ListScripts",
        "AWS::GameLift::Script",
        "Scripts",
        "ScriptArn",
        arns("gamelift", "script"),
        Some("NextToken"),
        json!({}),
    );
    for row in &mut scripts.rows {
        row["Name"] = json!("same-display-name");
        row["Version"] = json!("1.0");
        row["SizeOnDisk"] = json!(123);
        row["CreationTime"] = json!(1750000000);
        row["NodeJsVersion"] = json!("24.x");
        row["StorageLocation"] = json!({"Bucket":"owned-source", "Key":"script.zip", "RoleArn":"arn:aws:iam::123456789012:role/script-read", "ObjectVersion":"v1"});
    }
    let mut locations = case(
        "gamelift",
        "ListLocations",
        "AWS::GameLift::Location",
        "Locations",
        "LocationArn",
        [
            "arn:aws:gamelift:us-east-1::location/custom-first".to_owned(),
            "arn:aws:gamelift:us-east-1::location/custom-later".to_owned(),
        ],
        Some("NextToken"),
        json!({"Filters":["CUSTOM"]}),
    );
    for (row, name) in locations
        .rows
        .iter_mut()
        .zip(["custom-first", "custom-later"])
    {
        row["LocationName"] = json!(name);
        row["PingBeacon"] = json!({"UDPEndpoint":{"Domain":"beacon.example.test","Port":7770}});
    }
    let mut result = vec![scripts, locations];
    for (operation, kind, items, resource, build) in [
        (
            "ListComponentBuildVersions",
            "AWS::ImageBuilder::Component",
            "componentSummaryList",
            "component",
            true,
        ),
        (
            "ListWorkflowBuildVersions",
            "AWS::ImageBuilder::Workflow",
            "workflowSummaryList",
            "workflow/build",
            true,
        ),
        (
            "ListComponents",
            "AWS::ApiOnly::ImageBuilderComponentSemanticVersion",
            "componentVersionList",
            "component",
            false,
        ),
        (
            "ListImages",
            "AWS::ApiOnly::ImageBuilderImageSemanticVersion",
            "imageVersionList",
            "image",
            false,
        ),
        (
            "ListWorkflows",
            "AWS::ApiOnly::ImageBuilderWorkflowSemanticVersion",
            "workflowVersionList",
            "workflow/build",
            false,
        ),
    ] {
        let ids = ["1.0.0", "2.0.0"].map(|version| {
            format!(
                "arn:aws:imagebuilder:us-east-1:123456789012:{resource}/same/{version}{}",
                if build { "/1" } else { "" }
            )
        });
        let mut entry = case(
            "imagebuilder",
            operation,
            kind,
            items,
            "arn",
            ids,
            Some("nextToken"),
            if build {
                json!({})
            } else {
                json!({"owner":"Self"})
            },
        );
        for row in &mut entry.rows {
            row["name"] = json!("same");
            row["version"] = json!("1.0.0");
            row["owner"] = json!("123456789012");
            row["description"] = json!("owned metadata");
            row["type"] = json!("BUILD");
            if resource == "component" {
                row["platform"] = json!("Linux");
            }
            if build {
                row["tags"] = json!({"environment":"fixture"});
                row["state"] = json!({"status":"DISABLED","reason":"retained metadata"});
            }
        }
        result.push(entry);
    }
    for (operation, kind, items) in [
        ("ListBlueprints", "AWS::Glue::Blueprint", "Blueprints"),
        (
            "ListDevEndpoints",
            "AWS::Glue::DevEndpoint",
            "DevEndpointNames",
        ),
        ("ListMLTransforms", "AWS::Glue::MLTransform", "TransformIds"),
        ("ListTriggers", "AWS::Glue::Trigger", "TriggerNames"),
        (
            "ListDataQualityRulesets",
            "AWS::Glue::DataQualityRuleset",
            "Rulesets",
        ),
        ("ListUsageProfiles", "AWS::Glue::UsageProfile", "Profiles"),
        (
            "GetSecurityConfigurations",
            "AWS::Glue::SecurityConfiguration",
            "SecurityConfigurations",
        ),
    ] {
        let scalar = matches!(
            operation,
            "ListBlueprints" | "ListDevEndpoints" | "ListMLTransforms" | "ListTriggers"
        );
        let mut entry = case(
            "glue",
            operation,
            kind,
            items,
            if scalar { "$" } else { "Name" },
            names(),
            Some("NextToken"),
            json!({}),
        );
        if operation == "ListDataQualityRulesets" {
            for row in &mut entry.rows {
                row["TargetTable"] = json!({"TableName":"target","DatabaseName":"database","CatalogId":"123456789012"});
                row["RuleCount"] = json!(2);
            }
        }
        if operation == "GetSecurityConfigurations" {
            for row in &mut entry.rows {
                row["EncryptionConfiguration"] = json!({"S3Encryption":[{"S3EncryptionMode":"SSE-KMS","KmsKeyArn":"arn:aws:kms:us-east-1:123456789012:key/key-id"}],"CloudWatchEncryption":{"CloudWatchEncryptionMode":"SSE-KMS"},"JobBookmarksEncryption":{"JobBookmarksEncryptionMode":"CSE-KMS"},"DataQualityEncryption":{"DataQualityEncryptionMode":"SSEKMS"}});
            }
        }
        result.push(entry);
    }
    let mut capacity = case(
        "glacier",
        "ListProvisionedCapacity",
        "AWS::ApiOnly::GlacierProvisionedCapacity",
        "ProvisionedCapacityList",
        "CapacityId",
        names(),
        None,
        json!({}),
    );
    for row in &mut capacity.rows {
        row["StartDate"] = json!("2026-11-01T00:00:00Z");
        row["ExpirationDate"] = json!("2026-12-01T00:00:00Z");
    }
    result.push(capacity);
    let connector_ids = arns("inspector2", "connector");
    let config_ids = ["first", "later"]
        .map(|name| format!("arn:aws:config:us-east-1:123456789012:connector/azure/tenant/{name}"));
    let mut connectors = case(
        "inspector2",
        "ListConnectors",
        "AWS::InspectorV2::Connector",
        "items",
        "connectorArn",
        connector_ids.clone(),
        Some("nextToken"),
        json!({}),
    );
    for (row, config) in connectors.rows.iter_mut().zip(&config_ids) {
        row["awsConfigConnectorArn"] = json!(config);
        row["provider"] = json!("AZURE");
        row["enablementStatus"] = json!("FAILED_TO_ENABLE");
        row["createdAt"] = json!("2026-11-01T00:00:00Z");
        row["updatedAt"] = json!("2026-11-02T00:00:00Z");
        row["health"] = json!({"connectorStatus":"UNHEALTHY","lastCheckedAt":"2026-11-02T00:00:00Z","message":"configuration disabled"});
        row["scopeConfiguration"] = json!({"vmScanning":{"scopeType":"SUBSCRIPTION","scopeValues":["subscription-id"],"state":"DISABLED","stateReason":"operator disabled"},"containerImageScanning":{"scopeType":"TENANT"},"serverlessScanning":{"scopeType":"TENANT"}});
        row["tags"] = json!({"environment":"fixture"});
    }
    let mut scans = case(
        "inspector2",
        "ListConnectorScanConfigurations",
        "AWS::ApiOnly::Inspector2ConnectorScanConfiguration",
        "scanConfigurations",
        "awsConfigConnectorArn",
        config_ids,
        Some("nextToken"),
        json!({}),
    );
    for (row, connector) in scans.rows.iter_mut().zip(connector_ids) {
        row["connectorArns"] = json!([connector]);
        row["scanConfiguration"] =
            json!({"containerImageScanning":{"pushDuration":"DAYS_30","pullDuration":"DAYS_7"}});
    }
    result.extend([connectors, scans]);
    let ids = ["first", "later"].map(|name| {
        format!("arn:aws:invoicing::123456789012:procurement-portal-preference/{name}")
    });
    let mut preferences = case(
        "invoicing",
        "ListProcurementPortalPreferences",
        "AWS::Invoicing::ProcurementPortalPreference",
        "ProcurementPortalPreferences",
        "ProcurementPortalPreferenceArn",
        ids,
        Some("NextToken"),
        json!({}),
    );
    for row in &mut preferences.rows {
        row["AwsAccountId"] = json!("123456789012");
        row["ProcurementPortalName"] = json!("COUPA");
        row["BuyerDomain"] = json!("NetworkID");
        row["BuyerIdentifier"] = json!("buyer");
        row["SupplierDomain"] = json!("NetworkID");
        row["SupplierIdentifier"] = json!("supplier");
        row["Selector"] = json!({"InvoiceUnitArns":["arn:aws:invoicing::123456789012:invoice-unit/not-discovered"],"SellerOfRecords":["AWS_INC"]});
        row["EinvoiceDeliveryEnabled"] = json!(false);
        row["PurchaseOrderRetrievalEnabled"] = json!(false);
        row["Version"] = json!(1);
        row["CreateDate"] = json!(1750000000);
        row["LastUpdateDate"] = json!(1750000000);
    }
    result.push(preferences);
    result
}
