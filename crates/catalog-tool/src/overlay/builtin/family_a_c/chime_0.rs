// Pinned source: .local/aws-models/models/chime/service/2018-05-01/chime-2018-05-01.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "AssociatePhoneNumberWithUser",
            deferred(
                "Model requires AccountId, UserId, E164PhoneNumber. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates a phone number with the specified Amazon Chime user.",
            ),
        ),
        (
            "AssociateSigninDelegateGroupsWithAccount",
            deferred(
                "Model requires AccountId, SigninDelegateGroups. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates the specified sign-in delegate groups with the specified Amazon Chime account.",
            ),
        ),
        (
            "BatchCreateRoomMembership",
            deferred(
                "Model requires AccountId, RoomId, MembershipItemList. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds up to 50 members to a chat room in an Amazon Chime Enterprise account. Members can be users or bots. The member role designates whether the member is a chat room administrator or a general chat room member.",
            ),
        ),
        (
            "BatchDeletePhoneNumber",
            deferred(
                "Model requires PhoneNumberIds. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Moves phone numbers into the Deletion queue . Phone numbers must be disassociated from any users or Amazon Chime Voice Connectors before they can be deleted. Phone numbers remain in the Deletion queue for 7 days before they are…",
            ),
        ),
        (
            "BatchSuspendUser",
            deferred(
                "Model requires AccountId, UserIdList. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Suspends up to 50 users from a Team or EnterpriseLWA Amazon Chime account. For more information about different account types, see Managing Your Amazon Chime Accounts in the Amazon Chime Administration Guide . Users suspended…",
            ),
        ),
        (
            "BatchUnsuspendUser",
            deferred(
                "Model requires AccountId, UserIdList. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes the suspension from up to 50 previously suspended users for the specified Amazon Chime EnterpriseLWA account. Only users on EnterpriseLWA accounts can be unsuspended using this action. For more information about different…",
            ),
        ),
        (
            "BatchUpdatePhoneNumber",
            deferred(
                "Model requires UpdatePhoneNumberRequestItems. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates phone number product types or calling names. You can update one attribute at a time for each UpdatePhoneNumberRequestItem . For example, you can update the product type or the calling name. For toll-free numbers, you…",
            ),
        ),
        (
            "BatchUpdateUser",
            deferred(
                "Model requires AccountId, UpdateUserRequestItems. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates user details within the UpdateUserRequestItem object for up to 20 users for the specified Amazon Chime account. Currently, only LicenseType updates are supported for this action.",
            ),
        ),
        (
            "CreateAccount",
            deferred(
                "Model requires Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an Amazon Chime account under the administrator's AWS account. Only Team account types are currently supported for this action. For more information about different account types, see Managing Your Amazon Chime Accounts…",
            ),
        ),
        (
            "CreateBot",
            deferred(
                "Model requires AccountId, DisplayName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a bot for an Amazon Chime Enterprise account.",
            ),
        ),
        (
            "CreateMeetingDialOut",
            deferred(
                "Model requires MeetingId, FromPhoneNumber, ToPhoneNumber, JoinToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Uses the join token and call metadata in a meeting request (From number, To number, and so forth) to initiate an outbound call to a public switched telephone network (PSTN) and join them into a Chime meeting. Also ensures that…",
            ),
        ),
        (
            "CreatePhoneNumberOrder",
            deferred(
                "Model requires ProductType, E164PhoneNumbers. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an order for phone numbers to be provisioned. For toll-free numbers, you cannot use the Amazon Chime Business Calling product type. For numbers outside the U.S., you must use the Amazon Chime SIP Media Application Dial-In…",
            ),
        ),
        (
            "CreateRoom",
            deferred(
                "Model requires AccountId, Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a chat room for the specified Amazon Chime Enterprise account.",
            ),
        ),
        (
            "CreateRoomMembership",
            deferred(
                "Model requires AccountId, RoomId, MemberId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds a member to a chat room in an Amazon Chime Enterprise account. A member can be either a user or a bot. The member role designates whether the member is a chat room administrator or a general chat room member.",
            ),
        ),
        (
            "CreateUser",
            deferred(
                "Model requires AccountId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a user under the specified Amazon Chime account.",
            ),
        ),
        (
            "DeleteAccount",
            deferred(
                "Model requires AccountId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified Amazon Chime account. You must suspend all users before deleting Team account. You can use the BatchSuspendUser action to dodo. For EnterpriseLWA and EnterpriseAD accounts, you must release the claimed…",
            ),
        ),
        (
            "DeleteEventsConfiguration",
            deferred(
                "Model requires AccountId, BotId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the events configuration that allows a bot to receive outgoing events.",
            ),
        ),
        (
            "DeletePhoneNumber",
            deferred(
                "Model requires PhoneNumberId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Moves the specified phone number into the Deletion queue . A phone number must be disassociated from any users or Amazon Chime Voice Connectors before it can be deleted. Deleted phone numbers remain in the Deletion queue for 7…",
            ),
        ),
        (
            "DeleteRoom",
            deferred(
                "Model requires AccountId, RoomId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a chat room in an Amazon Chime Enterprise account.",
            ),
        ),
        (
            "DeleteRoomMembership",
            deferred(
                "Model requires AccountId, RoomId, MemberId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes a member from a chat room in an Amazon Chime Enterprise account.",
            ),
        ),
        (
            "DisassociatePhoneNumberFromUser",
            deferred(
                "Model requires AccountId, UserId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Disassociates the primary provisioned phone number from the specified Amazon Chime user.",
            ),
        ),
        (
            "DisassociateSigninDelegateGroupsFromAccount",
            deferred(
                "Model requires AccountId, GroupNames. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Disassociates the specified sign-in delegate groups from the specified Amazon Chime account.",
            ),
        ),
        (
            "GetAccount",
            deferred(
                "Model requires AccountId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves details for the specified Amazon Chime account, such as account type and supported licenses.",
            ),
        ),
        (
            "GetAccountSettings",
            deferred(
                "Model requires AccountId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves account settings for the specified Amazon Chime account ID, such as remote control and dialout settings. For more information about these settings, see Use the Policies Page in the Amazon Chime Administration Guide .",
            ),
        ),
        (
            "GetBot",
            deferred(
                "Model requires AccountId, BotId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves details for the specified bot, such as bot email address, bot type, status, and display name.",
            ),
        ),
        (
            "GetEventsConfiguration",
            deferred(
                "Model requires AccountId, BotId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets details for an events configuration that allows a bot to receive outgoing events, such as an HTTPS endpoint or Lambda function ARN.",
            ),
        ),
        (
            "GetGlobalSettings",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is BusinessCalling, VoiceConnector; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Retrieves global settings for the administrator's AWS account, such as Amazon Chime Business Calling and Amazon Chime Voice Connector settings.",
            ),
        ),
        (
            "GetPhoneNumber",
            deferred(
                "Model requires PhoneNumberId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves details for the specified phone number ID, such as associations, capabilities, and product type.",
            ),
        ),
        (
            "GetPhoneNumberOrder",
            deferred(
                "Model requires PhoneNumberOrderId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves details for the specified phone number order, such as the order creation timestamp, phone numbers in E.164 format, product type, and order status.",
            ),
        ),
        (
            "GetPhoneNumberSettings",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is CallingName, CallingNameUpdatedTimestamp; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Retrieves the phone number settings for the administrator's AWS account, such as the default outbound calling name.",
            ),
        ),
        (
            "GetRetentionSettings",
            deferred(
                "Model requires AccountId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the retention settings for the specified Amazon Chime Enterprise account. For more information about retention settings, see Managing Chat Retention Policies in the Amazon Chime Administration Guide .",
            ),
        ),
        (
            "GetRoom",
            deferred(
                "Model requires AccountId, RoomId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves room details, such as the room name, for a room in an Amazon Chime Enterprise account.",
            ),
        ),
        (
            "GetUser",
            deferred(
                "Model requires AccountId, UserId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves details for the specified user ID, such as primary email address, license type,and personal meeting PIN. To retrieve user details with an email address instead of a user ID, use the ListUsers action, and then filter by…",
            ),
        ),
        (
            "GetUserSettings",
            deferred(
                "Model requires AccountId, UserId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves settings for the specified user ID, such as any associated phone number settings.",
            ),
        ),
        (
            "InviteUsers",
            deferred(
                "Model requires AccountId, UserEmailList. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Sends email to a maximum of 50 users, inviting them to the specified Amazon Chime Team account. Only Team account types are currently supported for this action.",
            ),
        ),
        // Model: Lists the Amazon Chime accounts under the administrator's AWS account. You can filter accounts by account name prefix. To find out which Amazon Chime account a user belongs to, you can filter by the…
        (
            "ListAccounts",
            OperationOverlay {
                queryable: vec![path("AccountId")?],
                ..read(
                    "Accounts",
                    Identity::Key(vec![path("AccountId")?]),
                    Scope::Account,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListBots",
            deferred(
                "Model requires AccountId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the bots associated with the administrator's Amazon Chime Enterprise account ID.",
            ),
        ),
        (
            "ListPhoneNumberOrders",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is PhoneNumberOrders; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists the phone number orders for the administrator's Amazon Chime account.",
            ),
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "ListPhoneNumbers",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is PhoneNumbers; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists the phone numbers for the specified Amazon Chime account, Amazon Chime user, Amazon Chime Voice Connector, or Amazon Chime Voice Connector group.",
            ),
        ),
        (
            "ListRoomMemberships",
            deferred(
                "Model requires AccountId, RoomId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the membership details for the specified room in an Amazon Chime Enterprise account, such as the members' IDs, email addresses, and names.",
            ),
        ),
        (
            "ListRooms",
            deferred(
                "Model requires AccountId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the room details for the specified Amazon Chime Enterprise account. Optionally, filter the results by a member ID (user ID or bot ID) to see a list of rooms that the member belongs to.",
            ),
        ),
        (
            "ListSupportedPhoneNumberCountries",
            deferred(
                "Model requires ProductType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists supported phone number countries.",
            ),
        ),
        (
            "ListUsers",
            deferred(
                "Model requires AccountId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the users that belong to the specified Amazon Chime account. You can specify an email address to list only the user that the email address belongs to.",
            ),
        ),
        (
            "LogoutUser",
            deferred(
                "Model requires AccountId, UserId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Logs out the specified user from all of the devices they are currently logged into.",
            ),
        ),
        (
            "PutEventsConfiguration",
            deferred(
                "Model requires AccountId, BotId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an events configuration that allows a bot to receive outgoing events sent by Amazon Chime. Choose either an HTTPS endpoint or a Lambda function ARN. For more information, see Bot .",
            ),
        ),
        (
            "PutRetentionSettings",
            deferred(
                "Model requires AccountId, RetentionSettings. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Puts retention settings for the specified Amazon Chime Enterprise account. We recommend using AWS CloudTrail to monitor usage of this API for your account. For more information, see Logging Amazon Chime API Calls with AWS…",
            ),
        ),
        (
            "RedactConversationMessage",
            deferred(
                "Model requires AccountId, ConversationId, MessageId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Redacts the specified message from the specified Amazon Chime conversation.",
            ),
        ),
        (
            "RedactRoomMessage",
            deferred(
                "Model requires AccountId, RoomId, MessageId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Redacts the specified message from the specified Amazon Chime channel.",
            ),
        ),
        (
            "RegenerateSecurityToken",
            deferred(
                "Model requires AccountId, BotId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Regenerates the security token for a bot.",
            ),
        ),
        (
            "ResetPersonalPIN",
            deferred(
                "Model requires AccountId, UserId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Resets the personal meeting PIN for the specified user on an Amazon Chime account. Returns the User object with the updated personal meeting PIN.",
            ),
        ),
        (
            "RestorePhoneNumber",
            deferred(
                "Model requires PhoneNumberId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Moves a phone number from the Deletion queue back into the phone number Inventory .",
            ),
        ),
        (
            "SearchAvailablePhoneNumbers",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is E164PhoneNumbers; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Searches for phone numbers that can be ordered. For US numbers, provide at least one of the following search filters: AreaCode , City , State , or TollFreePrefix . If you provide City , you must also provide State . Numbers…",
            ),
        ),
        (
            "UpdateAccount",
            deferred(
                "Model requires AccountId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates account details for the specified Amazon Chime account. Currently, only account name and default license updates are supported for this action.",
            ),
        ),
        (
            "UpdateAccountSettings",
            deferred(
                "Model requires AccountId, AccountSettings. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the settings for the specified Amazon Chime account. You can update settings for remote control of shared screens, or for the dial-out option. For more information about these settings, see Use the Policies Page in the…",
            ),
        ),
        (
            "UpdateBot",
            deferred(
                "Model requires AccountId, BotId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the status of the specified bot, such as starting or stopping the bot from running in your Amazon Chime Enterprise account.",
            ),
        ),
        (
            "UpdateGlobalSettings",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is not an inventory record; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Updates global settings for the administrator's AWS account, such as Amazon Chime Business Calling and Amazon Chime Voice Connector settings.",
            ),
        ),
        (
            "UpdatePhoneNumber",
            deferred(
                "Model requires PhoneNumberId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates phone number details, such as product type or calling name, for the specified phone number ID. You can update one phone number detail at a time. For example, you can update either the product type or the calling name in…",
            ),
        ),
        (
            "UpdatePhoneNumberSettings",
            deferred(
                "Model requires CallingName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the phone number settings for the administrator's AWS account, such as the default outbound calling name. You can update the default outbound calling name once every seven days. Outbound calling names can take up to 72…",
            ),
        ),
        (
            "UpdateRoom",
            deferred(
                "Model requires AccountId, RoomId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates room details, such as the room name, for a room in an Amazon Chime Enterprise account.",
            ),
        ),
        (
            "UpdateRoomMembership",
            deferred(
                "Model requires AccountId, RoomId, MemberId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates room membership details, such as the member role, for a room in an Amazon Chime Enterprise account. The member role designates whether the member is a chat room administrator or a general chat room member. The member role…",
            ),
        ),
        (
            "UpdateUser",
            deferred(
                "Model requires AccountId, UserId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates user details for a specified user ID. Currently, only LicenseType updates are supported for this action.",
            ),
        ),
        (
            "UpdateUserSettings",
            deferred(
                "Model requires AccountId, UserId, UserSettings. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the settings for the specified user, such as phone number settings.",
            ),
        ),
    ])
}
