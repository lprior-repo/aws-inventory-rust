use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "AssociateChannelFlow",
            deferred(
                "Model requires ChannelArn, ChannelFlowArn, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates a channel flow with a channel. Once associated, all messages to that channel go through channel flow processors. To stop processing, use the DisassociateChannelFlow API. Only administrators or channel moderators can…",
            ),
        ),
        (
            "BatchCreateChannelMembership",
            deferred(
                "Model requires ChannelArn, MemberArns, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds a specified number of users and bots to a channel.",
            ),
        ),
        (
            "ChannelFlowCallback",
            deferred(
                "Model requires CallbackId, ChannelArn, ChannelMessage. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Calls back Amazon Chime SDK messaging with a processing response message. This should be invoked from the processor Lambda. This is a developer API. You can return one of the following processing responses: Update message content…",
            ),
        ),
        (
            "CreateChannel",
            deferred(
                "Model requires AppInstanceArn, Name, ClientRequestToken, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a channel to which you can add users and send messages. Restriction : You can't change a channel's privacy. The x-amz-chime-bearer request header is mandatory. Use the ARN of the AppInstanceUser or AppInstanceBot that…",
            ),
        ),
        (
            "CreateChannelBan",
            deferred(
                "Model requires ChannelArn, MemberArn, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Permanently bans a member from a channel. Moderators can't add banned members to a channel. To undo a ban, you first have to DeleteChannelBan , and then CreateChannelMembership . Bans are cleaned up when you delete users or…",
            ),
        ),
        (
            "CreateChannelFlow",
            deferred(
                "Model requires AppInstanceArn, Processors, Name, ClientRequestToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a channel flow, a container for processors. Processors are AWS Lambda functions that perform actions on chat messages, such as stripping out profanity. You can associate channel flows with channels, and the processors in…",
            ),
        ),
        (
            "CreateChannelMembership",
            deferred(
                "Model requires ChannelArn, MemberArn, Type, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds a member to a channel. The InvitedBy field in ChannelMembership is derived from the request header. A channel member can: List messages Send messages Receive messages Edit their own messages Leave the channel Privacy…",
            ),
        ),
        (
            "CreateChannelModerator",
            deferred(
                "Model requires ChannelArn, ChannelModeratorArn, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new ChannelModerator . A channel moderator can: Add and remove other members of the channel. Add and remove other moderators of the channel. Add and remove user bans for the channel. Redact messages in the channel. List…",
            ),
        ),
        (
            "DeleteChannel",
            deferred(
                "Model requires ChannelArn, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Immediately makes a channel and its memberships inaccessible and marks them for deletion. This is an irreversible process. The x-amz-chime-bearer request header is mandatory. Use the ARN of the AppInstanceUserArn or…",
            ),
        ),
        (
            "DeleteChannelBan",
            deferred(
                "Model requires ChannelArn, MemberArn, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes a member from a channel's ban list. The x-amz-chime-bearer request header is mandatory. Use the ARN of the AppInstanceUser or AppInstanceBot that makes the API call as the value in the header.",
            ),
        ),
        (
            "DeleteChannelFlow",
            deferred(
                "Model requires ChannelFlowArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a channel flow, an irreversible process. This is a developer API. This API works only when the channel flow is not associated with any channel. To get a list of all channels that a channel flow is associated with, use the…",
            ),
        ),
        (
            "DeleteChannelMembership",
            deferred(
                "Model requires ChannelArn, MemberArn, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes a member from a channel. The x-amz-chime-bearer request header is mandatory. Use the AppInstanceUserArn of the user that makes the API call as the value in the header.",
            ),
        ),
        (
            "DeleteChannelMessage",
            deferred(
                "Model requires ChannelArn, MessageId, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a channel message. Only admins can perform this action. Deletion makes messages inaccessible immediately. A background process deletes any revisions created by UpdateChannelMessage . The x-amz-chime-bearer request header…",
            ),
        ),
        (
            "DeleteChannelModerator",
            deferred(
                "Model requires ChannelArn, ChannelModeratorArn, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a channel moderator. The x-amz-chime-bearer request header is mandatory. Use the ARN of the AppInstanceUser or AppInstanceBot that makes the API call as the value in the header.",
            ),
        ),
        (
            "DeleteMessagingStreamingConfigurations",
            deferred(
                "Model requires AppInstanceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the streaming configurations for an AppInstance . For more information, see Streaming messaging data in the Amazon Chime SDK Developer Guide .",
            ),
        ),
        (
            "DescribeChannel",
            deferred(
                "Model requires ChannelArn, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the full details of a channel in an Amazon Chime AppInstance . The x-amz-chime-bearer request header is mandatory. Use the ARN of the AppInstanceUser or AppInstanceBot that makes the API call as the value in the header.",
            ),
        ),
        (
            "DescribeChannelBan",
            deferred(
                "Model requires ChannelArn, MemberArn, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the full details of a channel ban. The x-amz-chime-bearer request header is mandatory. Use the ARN of the AppInstanceUser or AppInstanceBot that makes the API call as the value in the header.",
            ),
        ),
        (
            "DescribeChannelFlow",
            deferred(
                "Model requires ChannelFlowArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the full details of a channel flow in an Amazon Chime AppInstance . This is a developer API.",
            ),
        ),
        (
            "DescribeChannelMembership",
            deferred(
                "Model requires ChannelArn, MemberArn, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the full details of a user's channel membership. The x-amz-chime-bearer request header is mandatory. Use the ARN of the AppInstanceUser or AppInstanceBot that makes the API call as the value in the header.",
            ),
        ),
        (
            "DescribeChannelMembershipForAppInstanceUser",
            deferred(
                "Model requires ChannelArn, AppInstanceUserArn, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the details of a channel based on the membership of the specified AppInstanceUser or AppInstanceBot . The x-amz-chime-bearer request header is mandatory. Use the ARN of the AppInstanceUser or AppInstanceBot that makes the…",
            ),
        ),
        (
            "DescribeChannelModeratedByAppInstanceUser",
            deferred(
                "Model requires ChannelArn, AppInstanceUserArn, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the full details of a channel moderated by the specified AppInstanceUser or AppInstanceBot . The x-amz-chime-bearer request header is mandatory. Use the ARN of the AppInstanceUser or AppInstanceBot that makes the API call…",
            ),
        ),
        (
            "DescribeChannelModerator",
            deferred(
                "Model requires ChannelArn, ChannelModeratorArn, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the full details of a single ChannelModerator. The x-amz-chime-bearer request header is mandatory. Use the AppInstanceUserArn of the user that makes the API call as the value in the header.",
            ),
        ),
        (
            "DisassociateChannelFlow",
            deferred(
                "Model requires ChannelArn, ChannelFlowArn, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Disassociates a channel flow from all its channels. Once disassociated, all messages to that channel stop going through the channel flow processor. Only administrators or channel moderators can disassociate a channel flow. The…",
            ),
        ),
        (
            "GetChannelMembershipPreferences",
            deferred(
                "Model requires ChannelArn, MemberArn, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the membership preferences of an AppInstanceUser or AppInstanceBot for the specified channel. A user or a bot must be a member of the channel and own the membership in order to retrieve membership preferences. Users or bots…",
            ),
        ),
        (
            "GetChannelMessage",
            deferred(
                "Model requires ChannelArn, MessageId, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the full details of a channel message. The x-amz-chime-bearer request header is mandatory. Use the ARN of the AppInstanceUser or AppInstanceBot that makes the API call as the value in the header.",
            ),
        ),
        (
            "GetChannelMessageStatus",
            deferred(
                "Model requires ChannelArn, MessageId, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets message status for a specified messageId . Use this API to determine the intermediate status of messages going through channel flow processing. The API provides an alternative to retrieving message status if the event was…",
            ),
        ),
        (
            "GetMessagingSessionEndpoint",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is Endpoint; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: The details of the endpoint for the messaging session.",
            ),
        ),
        (
            "GetMessagingStreamingConfigurations",
            deferred(
                "Model requires AppInstanceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the data streaming configuration for an AppInstance . For more information, see Streaming messaging data in the Amazon Chime SDK Developer Guide .",
            ),
        ),
        (
            "ListChannelBans",
            deferred(
                "Model requires ChannelArn, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all the users and bots banned from a particular channel. The x-amz-chime-bearer request header is mandatory. Use the ARN of the AppInstanceUser or AppInstanceBot that makes the API call as the value in the header.",
            ),
        ),
        (
            "ListChannelFlows",
            deferred(
                "Model requires AppInstanceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a paginated lists of all the channel flows created under a single Chime. This is a developer API.",
            ),
        ),
        (
            "ListChannelMemberships",
            deferred(
                "Model requires ChannelArn, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all channel memberships in a channel. The x-amz-chime-bearer request header is mandatory. Use the ARN of the AppInstanceUser or AppInstanceBot that makes the API call as the value in the header. If you want to list the…",
            ),
        ),
        (
            "ListChannelMembershipsForAppInstanceUser",
            deferred(
                "Model requires ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all channels that an AppInstanceUser or AppInstanceBot is a part of. Only an AppInstanceAdmin can call the API with a user ARN that is not their own. The x-amz-chime-bearer request header is mandatory. Use the ARN of the…",
            ),
        ),
        ("ListChannelMessages", denied(DenyReason::Content)),
        (
            "ListChannelModerators",
            deferred(
                "Model requires ChannelArn, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all the moderators for a channel. The x-amz-chime-bearer request header is mandatory. Use the ARN of the AppInstanceUser or AppInstanceBot that makes the API call as the value in the header.",
            ),
        ),
        (
            "ListChannels",
            deferred(
                "Model requires AppInstanceArn, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all Channels created under a single Chime App as a paginated list. You can specify filters to narrow results. Functionality & restrictions Use privacy = PUBLIC to retrieve all public channels in the account. Only an…",
            ),
        ),
        (
            "ListChannelsAssociatedWithChannelFlow",
            deferred(
                "Model requires ChannelFlowArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all channels associated with a specified channel flow. You can associate a channel flow with multiple channels, but you can only associate a channel with one channel flow. This is a developer API.",
            ),
        ),
        (
            "ListChannelsModeratedByAppInstanceUser",
            deferred(
                "Model requires ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: A list of the channels moderated by an AppInstanceUser . The x-amz-chime-bearer request header is mandatory. Use the ARN of the AppInstanceUser or AppInstanceBot that makes the API call as the value in the header.",
            ),
        ),
        (
            "ListSubChannels",
            deferred(
                "Model requires ChannelArn, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all the SubChannels in an elastic channel when given a channel ID. Available only to the app instance admins and channel moderators of elastic channels.",
            ),
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires ResourceARN. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the tags applied to an Amazon Chime SDK messaging resource.",
            ),
        ),
        (
            "PutChannelExpirationSettings",
            deferred(
                "Model requires ChannelArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Sets the number of days before the channel is automatically deleted. A background process deletes expired channels within 6 hours of expiration. Actual deletion times may vary. Expired channels that have not yet been deleted…",
            ),
        ),
        (
            "PutChannelMembershipPreferences",
            deferred(
                "Model requires ChannelArn, MemberArn, ChimeBearer, Preferences. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Sets the membership preferences of an AppInstanceUser or AppInstanceBot for the specified channel. The user or bot must be a member of the channel. Only the user or bot who owns the membership can set preferences. Users or bots…",
            ),
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "PutMessagingStreamingConfigurations",
            deferred(
                "Model requires AppInstanceArn, StreamingConfigurations. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Sets the data streaming configuration for an AppInstance . For more information, see Streaming messaging data in the Amazon Chime SDK Developer Guide .",
            ),
        ),
        (
            "RedactChannelMessage",
            deferred(
                "Model requires ChannelArn, MessageId, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Redacts message content and metadata. The message exists in the back end, but the action returns null content, and the state shows as redacted. The x-amz-chime-bearer request header is mandatory. Use the ARN of the…",
            ),
        ),
        (
            "SearchChannels",
            deferred(
                "Model requires Fields. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Allows the ChimeBearer to search channels by channel members. Users or bots can search across the channels that they belong to. Users in the AppInstanceAdmin role can search across all channels. The x-amz-chime-bearer request…",
            ),
        ),
        (
            "SendChannelMessage",
            deferred(
                "Model requires ChannelArn, Content, Type, Persistence, ClientRequestToken, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Sends a message to a particular channel that the member is a part of. The x-amz-chime-bearer request header is mandatory. Use the ARN of the AppInstanceUser or AppInstanceBot that makes the API call as the value in the header.…",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires ResourceARN, Tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Applies the specified tags to the specified Amazon Chime SDK messaging resource.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires ResourceARN, TagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes the specified tags from the specified Amazon Chime SDK messaging resource.",
            ),
        ),
        (
            "UpdateChannel",
            deferred(
                "Model requires ChannelArn, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Update a channel's attributes. Restriction : You can't change a channel's privacy. The x-amz-chime-bearer request header is mandatory. Use the ARN of the AppInstanceUser or AppInstanceBot that makes the API call as the value in…",
            ),
        ),
        (
            "UpdateChannelFlow",
            deferred(
                "Model requires ChannelFlowArn, Processors, Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates channel flow attributes. This is a developer API.",
            ),
        ),
        (
            "UpdateChannelMessage",
            deferred(
                "Model requires ChannelArn, MessageId, Content, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the content of a message. The x-amz-chime-bearer request header is mandatory. Use the ARN of the AppInstanceUser or AppInstanceBot that makes the API call as the value in the header.",
            ),
        ),
        (
            "UpdateChannelReadMarker",
            deferred(
                "Model requires ChannelArn, ChimeBearer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: The details of the time when a user last read messages in a channel. The x-amz-chime-bearer request header is mandatory. Use the ARN of the AppInstanceUser or AppInstanceBot that makes the API call as the value in the header.",
            ),
        ),
    ])
}
