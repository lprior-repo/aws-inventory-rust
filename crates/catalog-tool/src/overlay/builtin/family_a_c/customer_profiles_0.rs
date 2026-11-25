use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "AddProfileKey",
            deferred(
                "Model requires ProfileId, KeyName, Values, DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates a new key value with a specific profile, such as a Contact Record ContactId. A profile object can have a single unique key and any number of additional keys that can be used to identify the profile that it belongs to.",
            ),
        ),
        (
            "AssociateStreamForSegments",
            deferred(
                "Model requires DomainName, DestinationArn, DestinationRoleArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates an Amazon Kinesis data stream to receive segment membership events for a given domain. This is a domain-level configuration that applies to all segment subscriptions within the domain. A domain can have only one…",
            ),
        ),
        (
            "BatchGetCalculatedAttributeForProfile",
            deferred(
                "Model requires CalculatedAttributeName, DomainName, ProfileIds. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Fetch the possible attribute values given the attribute name.",
            ),
        ),
        (
            "BatchGetProfile",
            deferred(
                "Model requires DomainName, ProfileIds. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Get a batch of profiles.",
            ),
        ),
        (
            "BatchPutProfileObject",
            deferred(
                "Model requires DomainName, ObjectTypeName, Items. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds multiple profile objects to a domain of a given ObjectType in a single API call. When adding a specific profile object, like a Contact Record, an inferred profile can get created if it is not mapped to an existing profile.…",
            ),
        ),
        (
            "CreateCalculatedAttributeDefinition",
            deferred(
                "Model requires DomainName, CalculatedAttributeName, AttributeDetails, Statistic. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new calculated attribute definition. After creation, new object data ingested into Customer Profiles will be included in the calculated attribute, which can be retrieved for a profile using the…",
            ),
        ),
        (
            "CreateDomain",
            deferred(
                "Model requires DomainName, DefaultExpirationDays. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a domain, which is a container for all customer data, such as customer profile attributes, object types, profile keys, and encryption keys. You can create multiple domains, and each domain can have multiple third-party…",
            ),
        ),
        (
            "CreateDomainLayout",
            deferred(
                "Model requires DomainName, LayoutDefinitionName, Description, DisplayName, LayoutType, Layout. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates the layout to view data for a specific domain. This API can only be invoked from the Amazon Connect admin website.",
            ),
        ),
        (
            "CreateEventStream",
            deferred(
                "Model requires DomainName, Uri, EventStreamName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an event stream, which is a subscription to real-time events, such as when profiles are created and updated through Connect Customer Customer Profiles. Each event stream can be associated with only one Kinesis Data Stream…",
            ),
        ),
        (
            "CreateEventTrigger",
            deferred(
                "Model requires DomainName, EventTriggerName, ObjectTypeName, EventTriggerConditions. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an event trigger, which specifies the rules when to perform action based on customer's ingested data. Each event stream can be associated with only one integration in the same region and AWS account as the event stream.",
            ),
        ),
        (
            "CreateIntegrationWorkflow",
            deferred(
                "Model requires DomainName, WorkflowType, IntegrationConfig, ObjectTypeName, RoleArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an integration workflow. An integration workflow is an async process which ingests historic data and sets up an integration for ongoing updates. The supported Amazon AppFlow sources are Salesforce, ServiceNow, and Marketo.",
            ),
        ),
        (
            "CreateProfile",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a standard profile. A standard profile represents the following attributes for a customer profile in a domain.",
            ),
        ),
        (
            "CreateRecommender",
            deferred(
                "Model requires DomainName, RecommenderName, RecommenderRecipeName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a recommender",
            ),
        ),
        (
            "CreateRecommenderFilter",
            deferred(
                "Model requires DomainName, RecommenderFilterName, RecommenderFilterExpression. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a recommender filter. A recommender filter specifies which items to include or exclude from recommendations.",
            ),
        ),
        (
            "CreateRecommenderSchema",
            deferred(
                "Model requires DomainName, RecommenderSchemaName, Fields. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a recommender schema. A recommender schema defines the set of data columns available for training recommenders and filters under a domain.",
            ),
        ),
        (
            "CreateSegmentDefinition",
            deferred(
                "Model requires DomainName, SegmentDefinitionName, DisplayName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a segment definition associated to the given domain.",
            ),
        ),
        (
            "CreateSegmentEstimate",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a segment estimate query.",
            ),
        ),
        (
            "CreateSegmentSnapshot",
            deferred(
                "Model requires DomainName, SegmentDefinitionName, DataFormat. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Triggers a job to export a segment to a specified destination.",
            ),
        ),
        (
            "CreateUploadJob",
            deferred(
                "Model requires DomainName, DisplayName, Fields, UniqueKey. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an Upload job to ingest data for segment imports. The metadata is created for the job with the provided field mapping and unique key.",
            ),
        ),
        (
            "DeleteCalculatedAttributeDefinition",
            deferred(
                "Model requires DomainName, CalculatedAttributeName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an existing calculated attribute definition. Note that deleting a default calculated attribute is possible, however once deleted, you will be unable to undo that action and will need to recreate it on your own using the…",
            ),
        ),
        (
            "DeleteDomain",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a specific domain and all of its customer data, such as customer profile attributes and their related objects.",
            ),
        ),
        (
            "DeleteDomainLayout",
            deferred(
                "Model requires DomainName, LayoutDefinitionName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the layout used to view data for a specific domain. This API can only be invoked from the Amazon Connect admin website.",
            ),
        ),
        (
            "DeleteDomainObjectType",
            deferred(
                "Model requires DomainName, ObjectTypeName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Delete a DomainObjectType for the given Domain and ObjectType name.",
            ),
        ),
        (
            "DeleteEventStream",
            deferred(
                "Model requires DomainName, EventStreamName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Disables and deletes the specified event stream.",
            ),
        ),
        (
            "DeleteEventTrigger",
            deferred(
                "Model requires DomainName, EventTriggerName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Disable and deletes the Event Trigger. You cannot delete an Event Trigger with an active Integration associated.",
            ),
        ),
        (
            "DeleteIntegration",
            deferred(
                "Model requires DomainName, Uri. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes an integration from a specific domain.",
            ),
        ),
        (
            "DeleteProfile",
            deferred(
                "Model requires ProfileId, DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the standard customer profile and all data pertaining to the profile.",
            ),
        ),
        (
            "DeleteProfileKey",
            deferred(
                "Model requires ProfileId, KeyName, Values, DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes a searchable key from a customer profile.",
            ),
        ),
        (
            "DeleteProfileObject",
            deferred(
                "Model requires ProfileId, ProfileObjectUniqueKey, ObjectTypeName, DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes an object associated with a profile of a given ProfileObjectType.",
            ),
        ),
        (
            "DeleteProfileObjectType",
            deferred(
                "Model requires DomainName, ObjectTypeName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes a ProfileObjectType from a specific domain as well as removes all the ProfileObjects of that type. It also disables integrations from this specific ProfileObjectType. In addition, it scrubs all of the fields of the…",
            ),
        ),
        (
            "DeleteRecommender",
            deferred(
                "Model requires DomainName, RecommenderName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a recommender.",
            ),
        ),
        (
            "DeleteRecommenderFilter",
            deferred(
                "Model requires DomainName, RecommenderFilterName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a recommender filter from a domain.",
            ),
        ),
        (
            "DeleteRecommenderSchema",
            deferred(
                "Model requires DomainName, RecommenderSchemaName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a recommender schema from a domain.",
            ),
        ),
        (
            "DeleteSegmentDefinition",
            deferred(
                "Model requires DomainName, SegmentDefinitionName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a segment definition from the domain.",
            ),
        ),
        (
            "DeleteSegmentSubscription",
            deferred(
                "Model requires DomainName, SegmentDefinitionName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a segment subscription for membership events. All active event notifications for this segment are stopped.",
            ),
        ),
        (
            "DeleteWorkflow",
            deferred(
                "Model requires DomainName, WorkflowId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified workflow and all its corresponding resources. This is an async process.",
            ),
        ),
        (
            "DetectProfileObjectType",
            deferred(
                "Model requires Objects, DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: The process of detecting profile object type mapping by using given objects.",
            ),
        ),
        (
            "DisassociateStreamForSegments",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Disassociates the Amazon Kinesis data stream configured for segment membership events. All active segment subscriptions delivering events to this stream are eventually stopped.",
            ),
        ),
        (
            "GetAutoMergingPreview",
            deferred(
                "Model requires DomainName, Consolidation, ConflictResolution. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Tests the auto-merging settings of your Identity Resolution Job without merging your data. It randomly selects a sample of matching groups from the existing matching results, and applies the automerging settings that you…",
            ),
        ),
        (
            "GetCalculatedAttributeDefinition",
            deferred(
                "Model requires DomainName, CalculatedAttributeName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Provides more information on a calculated attribute definition for Customer Profiles.",
            ),
        ),
        (
            "GetCalculatedAttributeForProfile",
            deferred(
                "Model requires DomainName, ProfileId, CalculatedAttributeName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieve a calculated attribute for a customer profile.",
            ),
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "GetDomain",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("DomainName")?,
                    Binding::From {
                        operation: name("ListDomains")?,
                        path: path("DomainName")?,
                    },
                )]
                .into_iter()
                .collect(),
                tags: Some(path("Tags")?),
                queryable: vec![path("DomainName")?],
                ..read(
                    "$",
                    Identity::Key(vec![path("DomainName")?]),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "GetDomainLayout",
            deferred(
                "Model requires DomainName, LayoutDefinitionName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the layout to view data for a specific domain. This API can only be invoked from the Amazon Connect admin website.",
            ),
        ),
        (
            "GetDomainObjectType",
            deferred(
                "Model requires DomainName, ObjectTypeName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Return a DomainObjectType for the input Domain and ObjectType names.",
            ),
        ),
        (
            "GetEventStream",
            deferred(
                "Model requires DomainName, EventStreamName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about the specified event stream in a specific domain.",
            ),
        ),
        (
            "GetEventTrigger",
            deferred(
                "Model requires DomainName, EventTriggerName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Get a specific Event Trigger from the domain.",
            ),
        ),
        (
            "GetIdentityResolutionJob",
            deferred(
                "Model requires DomainName, JobId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about an Identity Resolution Job in a specific domain. Identity Resolution Jobs are set up using the Amazon Connect admin console. For more information, see Use Identity Resolution to consolidate similar…",
            ),
        ),
        (
            "GetIntegration",
            deferred(
                "Model requires DomainName, Uri. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns an integration for a domain.",
            ),
        ),
        (
            "GetMatches",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Before calling this API, use CreateDomain or UpdateDomain to enable identity resolution: set Matching to true. GetMatches returns potentially matching profiles, based on the results of the latest run of a machine learning…",
            ),
        ),
        (
            "GetObjectTypeAttributeStatistics",
            deferred(
                "Model requires DomainName, ObjectTypeName, AttributeName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: The GetObjectTypeAttributeValues API delivers statistical insights about attributes within a specific object type, but is exclusively available for domains with data store enabled. This API performs daily calculations to provide…",
            ),
        ),
        (
            "GetProfileHistoryRecord",
            deferred(
                "Model requires DomainName, ProfileId, Id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a history record for a specific profile, for a specific domain.",
            ),
        ),
        (
            "GetProfileObjectType",
            deferred(
                "Model requires DomainName, ObjectTypeName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the object types for a specific domain.",
            ),
        ),
        (
            "GetProfileObjectTypeTemplate",
            deferred(
                "Model requires TemplateId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the template information for a specific object type. A template is a predefined ProfileObjectType, such as “Salesforce-Account” or “Salesforce-Contact.” When a user sends a ProfileObject, using the PutProfileObject API,…",
            ),
        ),
        (
            "GetProfileRecommendations",
            deferred(
                "Model requires DomainName, ProfileId, RecommenderName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Fetches the recommendations for a profile in the input Customer Profiles domain. Fetches all the profile recommendations",
            ),
        ),
        (
            "GetRecommender",
            deferred(
                "Model requires DomainName, RecommenderName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a recommender.",
            ),
        ),
        (
            "GetRecommenderFilter",
            deferred(
                "Model requires DomainName, RecommenderFilterName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about a specific recommender filter in a domain.",
            ),
        ),
        (
            "GetRecommenderSchema",
            deferred(
                "Model requires DomainName, RecommenderSchemaName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about a specific recommender schema in a domain.",
            ),
        ),
        (
            "GetSegmentDefinition",
            deferred(
                "Model requires DomainName, SegmentDefinitionName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets a segment definition from the domain.",
            ),
        ),
        (
            "GetSegmentEstimate",
            deferred(
                "Model requires DomainName, EstimateId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the result of a segment estimate query.",
            ),
        ),
        (
            "GetSegmentMembership",
            deferred(
                "Model requires DomainName, SegmentDefinitionName, ProfileIds. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Determines if the given profiles are within a segment.",
            ),
        ),
        (
            "GetSegmentSnapshot",
            deferred(
                "Model requires DomainName, SegmentDefinitionName, SnapshotId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieve the latest status of a segment snapshot.",
            ),
        ),
        (
            "GetSegmentSubscription",
            deferred(
                "Model requires DomainName, SegmentDefinitionName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the current subscription configuration, execution schedule, and status for segment membership events.",
            ),
        ),
        (
            "GetSimilarProfiles",
            deferred(
                "Model requires DomainName, MatchType, SearchKey, SearchValue. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a set of profiles that belong to the same matching group using the matchId or profileId . You can also specify the type of matching that you want for finding similar profiles using either RULE_BASED_MATCHING or…",
            ),
        ),
        (
            "GetStreamForSegments",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about the segment membership event stream configured for a specific domain, including the stream state and associated segments.",
            ),
        ),
        (
            "GetUploadJob",
            deferred(
                "Model requires DomainName, JobId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API retrieves the details of a specific upload job.",
            ),
        ),
        (
            "GetUploadJobPath",
            deferred(
                "Model requires DomainName, JobId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API retrieves the pre-signed URL and client token for uploading the file associated with the upload job.",
            ),
        ),
        (
            "GetWorkflow",
            deferred(
                "Model requires DomainName, WorkflowId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Get details of specified workflow.",
            ),
        ),
        (
            "GetWorkflowSteps",
            deferred(
                "Model requires DomainName, WorkflowId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Get granular list of steps in workflow.",
            ),
        ),
        (
            "ListAccountIntegrations",
            deferred(
                "Model requires Uri. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all of the integrations associated to a specific URI in the AWS account.",
            ),
        ),
        (
            "ListCalculatedAttributeDefinitions",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists calculated attribute definitions for Customer Profiles",
            ),
        ),
        (
            "ListCalculatedAttributesForProfile",
            deferred(
                "Model requires DomainName, ProfileId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieve a list of calculated attributes for a customer profile.",
            ),
        ),
        (
            "ListDomainLayouts",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the existing layouts that can be used to view data for a specific domain. This API can only be invoked from the Amazon Connect admin website.",
            ),
        ),
        (
            "ListDomainObjectTypes",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: List all DomainObjectType(s) in a Customer Profiles domain.",
            ),
        ),
    ])
}

pub(super) fn part_2() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "ListDomains",
            OperationOverlay {
                tags: Some(path("Tags")?),
                queryable: vec![path("DomainName")?],
                ..read(
                    "Items",
                    Identity::Key(vec![path("DomainName")?]),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListEventStreams",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of all the event streams in a specific domain.",
            ),
        ),
        (
            "ListEventTriggers",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: List all Event Triggers under a domain.",
            ),
        ),
        (
            "ListIdentityResolutionJobs",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all of the Identity Resolution Jobs in your domain. The response sorts the list by JobStartTime .",
            ),
        ),
        (
            "ListIntegrations",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all of the integrations in your domain.",
            ),
        ),
        (
            "ListObjectTypeAttributeValues",
            deferred(
                "Model requires DomainName, ObjectTypeName, AttributeName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: The ListObjectTypeAttributeValues API provides access to the most recent distinct values for any specified attribute, making it valuable for real-time data validation and consistency checks within your object types. This API…",
            ),
        ),
        (
            "ListObjectTypeAttributes",
            deferred(
                "Model requires DomainName, ObjectTypeName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Fetch the possible attribute values given the attribute name.",
            ),
        ),
        (
            "ListProfileAttributeValues",
            deferred(
                "Model requires DomainName, AttributeName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Fetch the possible attribute values given the attribute name.",
            ),
        ),
        (
            "ListProfileHistoryRecords",
            deferred(
                "Model requires DomainName, ProfileId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of history records for a specific profile, for a specific domain.",
            ),
        ),
        (
            "ListProfileObjectTypeTemplates",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is Items; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists all of the template information for object types.",
            ),
        ),
        (
            "ListProfileObjectTypes",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all of the templates available within the service.",
            ),
        ),
        (
            "ListProfileObjects",
            deferred(
                "Model requires DomainName, ObjectTypeName, ProfileId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of objects associated with a profile of a given ProfileObjectType.",
            ),
        ),
        (
            "ListRecommenderFilters",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of recommender filters in the specified domain.",
            ),
        ),
        (
            "ListRecommenderRecipes",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is RecommenderRecipes; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns a list of available recommender recipes that can be used to create recommenders.",
            ),
        ),
        (
            "ListRecommenderSchemas",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of recommender schemas in the specified domain.",
            ),
        ),
        (
            "ListRecommenders",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of recommenders in the specified domain.",
            ),
        ),
        (
            "ListRuleBasedMatches",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a set of MatchIds that belong to the given domain.",
            ),
        ),
        (
            "ListSegmentDefinitions",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all segment definitions under a domain.",
            ),
        ),
        (
            "ListSegmentSubscriptionEvents",
            deferred(
                "Model requires DomainName, SegmentDefinitionName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the most recent membership events for a segment. Each event represents a profile that entered or exited the segment. This operation is paginated.",
            ),
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Displays the tags associated with an Amazon Connect Customer Profiles resource. In Connect Customer Profiles, domains, profile object types, and integrations can be tagged.",
            ),
        ),
        (
            "ListUploadJobs",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API retrieves a list of upload jobs for the specified domain.",
            ),
        ),
        (
            "ListWorkflows",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Query to list all workflows.",
            ),
        ),
        (
            "MergeProfiles",
            deferred(
                "Model requires DomainName, MainProfileId, ProfileIdsToBeMerged. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Runs an AWS Lambda job that does the following: All the profileKeys in the ProfileToBeMerged will be moved to the main profile. All the objects in the ProfileToBeMerged will be moved to the main profile. All the ProfileToBeMerged…",
            ),
        ),
        (
            "PutDomainObjectType",
            deferred(
                "Model requires DomainName, ObjectTypeName, Fields. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Create/Update a DomainObjectType in a Customer Profiles domain. To create a new DomainObjectType, Data Store needs to be enabled on the Domain.",
            ),
        ),
        (
            "PutIntegration",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds an integration between the service and a third-party service, which includes Amazon AppFlow and Amazon Connect. An integration can belong to only one domain. To add or remove tags on an existing Integration, see TagResource…",
            ),
        ),
        (
            "PutProfileObject",
            deferred(
                "Model requires ObjectTypeName, Object, DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds additional objects to customer profiles of a given ObjectType. When adding a specific profile object, like a Contact Record, an inferred profile can get created if it is not mapped to an existing profile. The resulting…",
            ),
        ),
        (
            "PutProfileObjectType",
            deferred(
                "Model requires DomainName, ObjectTypeName, Description. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Defines a ProfileObjectType. To add or remove tags on an existing ObjectType, see TagResource / UntagResource .",
            ),
        ),
        (
            "PutSegmentSubscription",
            deferred(
                "Model requires DomainName, SegmentDefinitionName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates or updates a segment subscription for membership events. When a subscription is created, an initial snapshot is taken and the system begins monitoring for membership changes. You can optionally set a schedule…",
            ),
        ),
        (
            "SearchProfiles",
            deferred(
                "Model requires DomainName, KeyName, Values. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches for profiles within a specific domain using one or more predefined search keys (e.g., _fullName, _phone, _email, _account, etc.) and/or custom-defined search keys. A search key is a data type pair that consists of a…",
            ),
        ),
        (
            "SearchRecommendations",
            deferred(
                "Model requires DomainName, KeyName, KeyValues, Recommender. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves recommendations for a profile in a specific domain. The profile is identified using a search key, which consists of a KeyName and a KeyValues list. The KeyName can be a predefined key (for example, _profileId , _phone ,…",
            ),
        ),
        (
            "StartRecommender",
            deferred(
                "Model requires DomainName, RecommenderName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts a recommender that was previously stopped. Starting a recommender resumes its ability to generate recommendations.",
            ),
        ),
        (
            "StartUploadJob",
            deferred(
                "Model requires DomainName, JobId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API starts the processing of an upload job to ingest profile data.",
            ),
        ),
        (
            "StopRecommender",
            deferred(
                "Model requires DomainName, RecommenderName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops a recommender, suspending its ability to generate recommendations. The recommender can be restarted later using StartRecommender.",
            ),
        ),
        (
            "StopUploadJob",
            deferred(
                "Model requires DomainName, JobId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API stops the processing of an upload job.",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires resourceArn, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Assigns one or more tags (key-value pairs) to the specified Amazon Connect Customer Profiles resource. Tags can help you organize and categorize your resources. You can also use them to scope user permissions by granting a user…",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires resourceArn, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes one or more tags from the specified Amazon Connect Customer Profiles resource. In Connect Customer Profiles, domains, profile object types, and integrations can be tagged.",
            ),
        ),
        (
            "UpdateCalculatedAttributeDefinition",
            deferred(
                "Model requires DomainName, CalculatedAttributeName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing calculated attribute definition. When updating the Conditions, note that increasing the date range of a calculated attribute will not trigger inclusion of historical data greater than the current date range.",
            ),
        ),
    ])
}

pub(super) fn part_3() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "UpdateDomain",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the properties of a domain, including creating or selecting a dead letter queue or an encryption key. After a domain is created, the name can’t be changed. Use this API or CreateDomain to enable identity resolution : set…",
            ),
        ),
        (
            "UpdateDomainLayout",
            deferred(
                "Model requires DomainName, LayoutDefinitionName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the layout used to view data for a specific domain. This API can only be invoked from the Amazon Connect admin website.",
            ),
        ),
        (
            "UpdateEventTrigger",
            deferred(
                "Model requires DomainName, EventTriggerName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Update the properties of an Event Trigger.",
            ),
        ),
        (
            "UpdateProfile",
            deferred(
                "Model requires DomainName, ProfileId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the properties of a profile. The ProfileId is required for updating a customer profile. When calling the UpdateProfile API, specifying an empty string value means that any existing value will be removed. Not specifying a…",
            ),
        ),
        (
            "UpdateRecommender",
            deferred(
                "Model requires DomainName, RecommenderName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the properties of an existing recommender, allowing you to modify its configuration and description.",
            ),
        ),
    ])
}
