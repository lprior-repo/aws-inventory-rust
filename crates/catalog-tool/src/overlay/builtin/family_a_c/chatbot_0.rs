// Pinned source: .local/aws-models/models/chatbot/service/2017-10-11/chatbot-2017-10-11.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        ("AssociateToConfiguration", deferred("Model requires Resource, ChatConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Links a resource (for example, a custom action) to a channel configuration.")),
        ("CreateChimeWebhookConfiguration", deferred("Model requires WebhookDescription, WebhookUrl, SnsTopicArns, IamRoleArn, ConfigurationName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an AWS Chatbot configuration for Amazon Chime.")),
        ("CreateCustomAction", deferred("Model requires Definition, ActionName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a custom action that can be invoked as an alias or as a button on a notification.")),
        ("CreateMicrosoftTeamsChannelConfiguration", deferred("Model requires ChannelId, TeamId, TenantId, IamRoleArn, ConfigurationName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an AWS Chatbot configuration for Microsoft Teams.")),
        ("CreateSlackChannelConfiguration", deferred("Model requires SlackTeamId, SlackChannelId, IamRoleArn, ConfigurationName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an AWS Chatbot confugration for Slack.")),
        ("DeleteChimeWebhookConfiguration", deferred("Model requires ChatConfigurationArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a Amazon Chime webhook configuration for AWS Chatbot.")),
        ("DeleteCustomAction", deferred("Model requires CustomActionArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a custom action.")),
        ("DeleteMicrosoftTeamsChannelConfiguration", deferred("Model requires ChatConfigurationArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a Microsoft Teams channel configuration for AWS Chatbot")),
        ("DeleteMicrosoftTeamsConfiguredTeam", deferred("Model requires TeamId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the Microsoft Teams team authorization allowing for channels to be configured in that Microsoft Teams team. Note that the Microsoft Teams team must have no channels configured to remove it.")),
        ("DeleteMicrosoftTeamsUserIdentity", deferred("Model requires ChatConfigurationArn, UserId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Identifes a user level permission for a channel configuration.")),
        ("DeleteSlackChannelConfiguration", deferred("Model requires ChatConfigurationArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a Slack channel configuration for AWS Chatbot")),
        ("DeleteSlackUserIdentity", deferred("Model requires ChatConfigurationArn, SlackTeamId, SlackUserId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a user level permission for a Slack channel configuration.")),
        ("DeleteSlackWorkspaceAuthorization", deferred("Model requires SlackTeamId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the Slack workspace authorization that allows channels to be configured in that workspace. This requires all configured channels in the workspace to be deleted.")),
        // Model: Lists Amazon Chime webhook configurations optionally filtered by ChatConfigurationArn
        ("DescribeChimeWebhookConfigurations", OperationOverlay {
            tags: Some(path("Tags")?),
            queryable: vec![path("ChatConfigurationArn")?],
            ..read("WebhookConfigurations", Identity::Arn(path("ChatConfigurationArn")?), Scope::Account, token("NextToken", "NextToken")?)?
        }),
        // Model: Lists Slack channel configurations optionally filtered by ChatConfigurationArn
        ("DescribeSlackChannelConfigurations", OperationOverlay {
            tags: Some(path("Tags")?),
            queryable: vec![path("ChatConfigurationArn")?],
            ..read("SlackChannelConfigurations", Identity::Arn(path("ChatConfigurationArn")?), Scope::Account, token("NextToken", "NextToken")?)?
        }),
        ("DescribeSlackUserIdentities", deferred("Slack user identity references are external to AWS. No reviewed complete parent binding and selected-item resource identity is authorized for this operation.")),
        ("DescribeSlackWorkspaces", deferred("Slack workspace authorizations are external to AWS. No reviewed complete parent binding and selected-item resource identity is authorized for this operation.")),
        ("DisassociateFromConfiguration", deferred("Model requires Resource, ChatConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Unlink a resource, for example a custom action, from a channel configuration.")),
        ("GetAccountPreferences", deferred("No safe resource collection plan is approved for this modeled operation. Its output is AccountPreferences; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns AWS Chatbot account preferences.")),
        ("GetCustomAction", deferred("Model requires CustomActionArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a custom action.")),
        ("GetMicrosoftTeamsChannelConfiguration", deferred("Model requires ChatConfigurationArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a Microsoft Teams channel configuration in an AWS account.")),
        ("ListAssociations", OperationOverlay {
            queryable: vec![path("ChatConfiguration")?],
            ..read("Associations", Identity::Arn(path("Resource")?), Scope::Account, token("NextToken", "NextToken")?)?
        }),
        // Model: Lists custom actions defined in this account.
        ("ListCustomActions", OperationOverlay {
            queryable: vec![path("$")?],
            ..read("CustomActions", Identity::Arn(path("$")?), Scope::Account, token("NextToken", "NextToken")?)?
        }),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        // Model: Lists all AWS Chatbot Microsoft Teams channel configurations in an AWS account.
        ("ListMicrosoftTeamsChannelConfigurations", OperationOverlay {
            tags: Some(path("Tags")?),
            queryable: vec![path("ChatConfigurationArn")?],
            ..read("TeamChannelConfigurations", Identity::Arn(path("ChatConfigurationArn")?), Scope::Account, token("NextToken", "NextToken")?)?
        }),
        ("ListMicrosoftTeamsConfiguredTeams", deferred("Microsoft Teams team references are external to AWS. No reviewed complete parent binding and selected-item resource identity is authorized for this operation.")),
        ("ListMicrosoftTeamsUserIdentities", deferred("Microsoft Teams user identity references are external to AWS. No reviewed complete parent binding and selected-item resource identity is authorized for this operation.")),
        ("ListTagsForResource", deferred("Model requires ResourceARN. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all of the tags associated with the Amazon Resource Name (ARN) that you specify. The resource can be a user, server, or role.")),
        ("TagResource", deferred("Model requires ResourceARN, Tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Attaches a key-value pair to a resource, as identified by its Amazon Resource Name (ARN). Resources are users, servers, roles, and other entities.")),
        ("UntagResource", deferred("Model requires ResourceARN, TagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Detaches a key-value pair from a resource, as identified by its Amazon Resource Name (ARN). Resources are users, servers, roles, and other entities.")),
        ("UpdateAccountPreferences", deferred("No safe resource collection plan is approved for this modeled operation. Its output is AccountPreferences; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Updates AWS Chatbot account preferences.")),
        ("UpdateChimeWebhookConfiguration", deferred("Model requires ChatConfigurationArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a Amazon Chime webhook configuration.")),
        ("UpdateCustomAction", deferred("Model requires CustomActionArn, Definition. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a custom action.")),
        ("UpdateMicrosoftTeamsChannelConfiguration", deferred("Model requires ChatConfigurationArn, ChannelId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an Microsoft Teams channel configuration.")),
        ("UpdateSlackChannelConfiguration", deferred("Model requires ChatConfigurationArn, SlackChannelId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a Slack channel configuration.")),
    ])
}