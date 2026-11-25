use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "AssociatePhoneNumbersWithVoiceConnector",
            deferred(
                "Model requires VoiceConnectorId, E164PhoneNumbers. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates phone numbers with the specified Amazon Chime SDK Voice Connector.",
            ),
        ),
        (
            "AssociatePhoneNumbersWithVoiceConnectorGroup",
            deferred(
                "Model requires VoiceConnectorGroupId, E164PhoneNumbers. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates phone numbers with the specified Amazon Chime SDK Voice Connector group.",
            ),
        ),
        (
            "BatchDeletePhoneNumber",
            deferred(
                "Model requires PhoneNumberIds. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Moves phone numbers into the Deletion queue . Phone numbers must be disassociated from any users or Amazon Chime SDK Voice Connectors before they can be deleted. Phone numbers remain in the Deletion queue for 7 days before they…",
            ),
        ),
        (
            "BatchUpdatePhoneNumber",
            deferred(
                "Model requires UpdatePhoneNumberRequestItems. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates phone number product types, calling names, or phone number names. You can update one attribute at a time for each UpdatePhoneNumberRequestItem . For example, you can update the product type, the calling name, or phone…",
            ),
        ),
        (
            "CreatePhoneNumberOrder",
            deferred(
                "Model requires ProductType, E164PhoneNumbers. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an order for phone numbers to be provisioned. For numbers outside the U.S., you must use the Amazon Chime SDK SIP media application dial-in product type.",
            ),
        ),
        (
            "CreateProxySession",
            deferred(
                "Model requires VoiceConnectorId, ParticipantPhoneNumbers, Capabilities. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a proxy session for the specified Amazon Chime SDK Voice Connector for the specified participant phone numbers. End of support notice: On April 7, 2026, AWS will end support for Amazon Chime SDK proxy sessions.",
            ),
        ),
        (
            "CreateSipMediaApplication",
            deferred(
                "Model requires AwsRegion, Name, Endpoints. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a SIP media application. For more information about SIP media applications, see Managing SIP media applications and rules in the Amazon Chime SDK Administrator Guide .",
            ),
        ),
        (
            "CreateSipMediaApplicationCall",
            deferred(
                "Model requires FromPhoneNumber, ToPhoneNumber, SipMediaApplicationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an outbound call to a phone number from the phone number specified in the request, and it invokes the endpoint of the specified sipMediaApplicationId .",
            ),
        ),
        (
            "CreateSipRule",
            deferred(
                "Model requires Name, TriggerType, TriggerValue, TargetApplications. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a SIP rule, which can be used to run a SIP media application as a target for a specific trigger type. For more information about SIP rules, see Managing SIP media applications and rules in the Amazon Chime SDK…",
            ),
        ),
        (
            "CreateVoiceConnector",
            deferred(
                "Model requires Name, RequireEncryption. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an Amazon Chime SDK Voice Connector. For more information about Voice Connectors, see Managing Amazon Chime SDK Voice Connector groups in the Amazon Chime SDK Administrator Guide .",
            ),
        ),
        (
            "CreateVoiceConnectorGroup",
            deferred(
                "Model requires Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an Amazon Chime SDK Voice Connector group under the administrator's AWS account. You can associate Amazon Chime SDK Voice Connectors with the Voice Connector group by including VoiceConnectorItems in the request. You can…",
            ),
        ),
        (
            "CreateVoiceProfile",
            deferred(
                "Model requires SpeakerSearchTaskId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a voice profile, which consists of an enrolled user and their latest voice print. Before creating any voice profiles, you must provide all notices and obtain all consents from the speaker as required under applicable…",
            ),
        ),
        (
            "CreateVoiceProfileDomain",
            deferred(
                "Model requires Name, ServerSideEncryptionConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a voice profile domain, a collection of voice profiles, their voice prints, and encrypted enrollment audio. Before creating any voice profiles, you must provide all notices and obtain all consents from the speaker as…",
            ),
        ),
        (
            "DeletePhoneNumber",
            deferred(
                "Model requires PhoneNumberId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Moves the specified phone number into the Deletion queue . A phone number must be disassociated from any users or Amazon Chime SDK Voice Connectors before it can be deleted. Deleted phone numbers remain in the Deletion queue…",
            ),
        ),
        (
            "DeleteProxySession",
            deferred(
                "Model requires VoiceConnectorId, ProxySessionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified proxy session from the specified Amazon Chime SDK Voice Connector. End of support notice: On April 7, 2026, AWS will end support for Amazon Chime SDK proxy sessions.",
            ),
        ),
        (
            "DeleteSipMediaApplication",
            deferred(
                "Model requires SipMediaApplicationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a SIP media application.",
            ),
        ),
        (
            "DeleteSipRule",
            deferred(
                "Model requires SipRuleId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a SIP rule.",
            ),
        ),
        (
            "DeleteVoiceConnector",
            deferred(
                "Model requires VoiceConnectorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an Amazon Chime SDK Voice Connector. Any phone numbers associated with the Amazon Chime SDK Voice Connector must be disassociated from it before it can be deleted.",
            ),
        ),
        (
            "DeleteVoiceConnectorEmergencyCallingConfiguration",
            deferred(
                "Model requires VoiceConnectorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the emergency calling details from the specified Amazon Chime SDK Voice Connector.",
            ),
        ),
        (
            "DeleteVoiceConnectorExternalSystemsConfiguration",
            deferred(
                "Model requires VoiceConnectorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the external systems configuration for a Voice Connector.",
            ),
        ),
        (
            "DeleteVoiceConnectorGroup",
            deferred(
                "Model requires VoiceConnectorGroupId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an Amazon Chime SDK Voice Connector group. Any VoiceConnectorItems and phone numbers associated with the group must be removed before it can be deleted.",
            ),
        ),
        (
            "DeleteVoiceConnectorOrigination",
            deferred(
                "Model requires VoiceConnectorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the origination settings for the specified Amazon Chime SDK Voice Connector. If emergency calling is configured for the Voice Connector, it must be deleted prior to deleting the origination settings.",
            ),
        ),
        (
            "DeleteVoiceConnectorProxy",
            deferred(
                "Model requires VoiceConnectorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the proxy configuration from the specified Amazon Chime SDK Voice Connector. End of support notice: On April 7, 2026, AWS will end support for Amazon Chime SDK proxy sessions.",
            ),
        ),
        (
            "DeleteVoiceConnectorStreamingConfiguration",
            deferred(
                "Model requires VoiceConnectorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a Voice Connector's streaming configuration.",
            ),
        ),
        (
            "DeleteVoiceConnectorTermination",
            deferred(
                "Model requires VoiceConnectorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the termination settings for the specified Amazon Chime SDK Voice Connector. If emergency calling is configured for the Voice Connector, it must be deleted prior to deleting the termination settings.",
            ),
        ),
        (
            "DeleteVoiceConnectorTerminationCredentials",
            deferred(
                "Model requires VoiceConnectorId, Usernames. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified SIP credentials used by your equipment to authenticate during call termination.",
            ),
        ),
        (
            "DeleteVoiceProfile",
            deferred(
                "Model requires VoiceProfileId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a voice profile, including its voice print and enrollment data. WARNING: This action is not reversible.",
            ),
        ),
        (
            "DeleteVoiceProfileDomain",
            deferred(
                "Model requires VoiceProfileDomainId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes all voice profiles in the domain. WARNING: This action is not reversible.",
            ),
        ),
        (
            "DisassociatePhoneNumbersFromVoiceConnector",
            deferred(
                "Model requires VoiceConnectorId, E164PhoneNumbers. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Disassociates the specified phone numbers from the specified Amazon Chime SDK Voice Connector.",
            ),
        ),
        (
            "DisassociatePhoneNumbersFromVoiceConnectorGroup",
            deferred(
                "Model requires VoiceConnectorGroupId, E164PhoneNumbers. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Disassociates the specified phone numbers from the specified Amazon Chime SDK Voice Connector group.",
            ),
        ),
        (
            "GetGlobalSettings",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is VoiceConnector; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Retrieves the global settings for the Amazon Chime SDK Voice Connectors in an AWS account.",
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
            "GetProxySession",
            deferred(
                "Model requires VoiceConnectorId, ProxySessionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the specified proxy session details for the specified Amazon Chime SDK Voice Connector. End of support notice: On April 7, 2026, AWS will end support for Amazon Chime SDK proxy sessions.",
            ),
        ),
        (
            "GetSipMediaApplication",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("SipMediaApplicationId")?,
                    Binding::From {
                        operation: name("ListSipMediaApplications")?,
                        path: path("SipMediaApplicationId")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("SipMediaApplicationArn")?],
                ..read(
                    "SipMediaApplication",
                    Identity::Arn(path("SipMediaApplicationArn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "GetSipMediaApplicationAlexaSkillConfiguration",
            deferred(
                "Model requires SipMediaApplicationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the Alexa Skill configuration for the SIP media application. Due to changes made by the Amazon Alexa service, this API is no longer available for use. For more information, refer to the Alexa Smart Properties page.",
            ),
        ),
        (
            "GetSipMediaApplicationLoggingConfiguration",
            deferred(
                "Model requires SipMediaApplicationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the logging configuration for the specified SIP media application.",
            ),
        ),
        (
            "GetSipRule",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("SipRuleId")?,
                    Binding::From {
                        operation: name("ListSipRules")?,
                        path: path("SipRuleId")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("SipRuleId")?],
                ..read(
                    "SipRule",
                    Identity::Key(vec![path("SipRuleId")?]),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "GetSpeakerSearchTask",
            deferred(
                "Model requires VoiceConnectorId, SpeakerSearchTaskId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the details of the specified speaker search task.",
            ),
        ),
        (
            "GetVoiceConnector",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("VoiceConnectorId")?,
                    Binding::From {
                        operation: name("ListVoiceConnectors")?,
                        path: path("VoiceConnectorId")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("VoiceConnectorArn")?],
                ..read(
                    "VoiceConnector",
                    Identity::Arn(path("VoiceConnectorArn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "GetVoiceConnectorEmergencyCallingConfiguration",
            deferred(
                "Model requires VoiceConnectorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the emergency calling configuration details for the specified Voice Connector.",
            ),
        ),
        (
            "GetVoiceConnectorExternalSystemsConfiguration",
            deferred(
                "Model requires VoiceConnectorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about an external systems configuration for a Voice Connector.",
            ),
        ),
        (
            "GetVoiceConnectorGroup",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("VoiceConnectorGroupId")?,
                    Binding::From {
                        operation: name("ListVoiceConnectorGroups")?,
                        path: path("VoiceConnectorGroupId")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("VoiceConnectorGroupArn")?],
                ..read(
                    "VoiceConnectorGroup",
                    Identity::Arn(path("VoiceConnectorGroupArn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "GetVoiceConnectorLoggingConfiguration",
            deferred(
                "Model requires VoiceConnectorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the logging configuration settings for the specified Voice Connector. Shows whether SIP message logs are enabled for sending to Amazon CloudWatch Logs.",
            ),
        ),
        (
            "GetVoiceConnectorOrigination",
            deferred(
                "Model requires VoiceConnectorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the origination settings for the specified Voice Connector.",
            ),
        ),
        (
            "GetVoiceConnectorProxy",
            deferred(
                "Model requires VoiceConnectorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the proxy configuration details for the specified Amazon Chime SDK Voice Connector. End of support notice: On April 7, 2026, AWS will end support for Amazon Chime SDK proxy sessions.",
            ),
        ),
        (
            "GetVoiceConnectorStreamingConfiguration",
            deferred(
                "Model requires VoiceConnectorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the streaming configuration details for the specified Amazon Chime SDK Voice Connector. Shows whether media streaming is enabled for sending to Amazon Kinesis. It also shows the retention period, in hours, for the…",
            ),
        ),
        (
            "GetVoiceConnectorTermination",
            deferred(
                "Model requires VoiceConnectorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the termination setting details for the specified Voice Connector.",
            ),
        ),
        (
            "GetVoiceConnectorTerminationHealth",
            deferred(
                "Model requires VoiceConnectorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about the last time a SIP OPTIONS ping was received from your SIP infrastructure for the specified Amazon Chime SDK Voice Connector.",
            ),
        ),
        (
            "GetVoiceProfile",
            deferred(
                "Model requires VoiceProfileId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the details of the specified voice profile.",
            ),
        ),
        (
            "GetVoiceProfileDomain",
            deferred(
                "Model requires VoiceProfileDomainId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the details of the specified voice profile domain.",
            ),
        ),
        (
            "GetVoiceToneAnalysisTask",
            deferred(
                "Model requires VoiceConnectorId, VoiceToneAnalysisTaskId, IsCaller. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the details of a voice tone analysis task.",
            ),
        ),
        (
            "ListAvailableVoiceConnectorRegions",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is VoiceConnectorRegions; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists the available AWS Regions in which you can create an Amazon Chime SDK Voice Connector.",
            ),
        ),
        (
            "ListPhoneNumberOrders",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is PhoneNumberOrders; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists the phone numbers for an administrator's Amazon Chime SDK account.",
            ),
        ),
        (
            "ListPhoneNumbers",
            OperationOverlay {
                page_size: Some(99),
                queryable: vec![path("PhoneNumberArn")?],
                ..read(
                    "PhoneNumbers",
                    Identity::Arn(path("PhoneNumberArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListProxySessions",
            deferred(
                "Model requires VoiceConnectorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the proxy sessions for the specified Amazon Chime SDK Voice Connector. End of support notice: On April 7, 2026, AWS will end support for Amazon Chime SDK proxy sessions.",
            ),
        ),
    ])
}

pub(super) fn part_2() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "ListSipMediaApplications",
            OperationOverlay {
                queryable: vec![path("SipMediaApplicationArn")?],
                ..read(
                    "SipMediaApplications",
                    Identity::Arn(path("SipMediaApplicationArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListSipRules",
            OperationOverlay {
                queryable: vec![path("SipRuleId")?],
                ..read(
                    "SipRules",
                    Identity::Key(vec![path("SipRuleId")?]),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListSupportedPhoneNumberCountries",
            deferred(
                "Model requires ProductType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the countries that you can order phone numbers from.",
            ),
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires ResourceARN. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of the tags in a given resource.",
            ),
        ),
        (
            "ListVoiceConnectorGroups",
            OperationOverlay {
                queryable: vec![path("VoiceConnectorGroupArn")?],
                ..read(
                    "VoiceConnectorGroups",
                    Identity::Arn(path("VoiceConnectorGroupArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListVoiceConnectorTerminationCredentials",
            deferred(
                "Model requires VoiceConnectorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the SIP credentials for the specified Amazon Chime SDK Voice Connector.",
            ),
        ),
        (
            "ListVoiceConnectors",
            OperationOverlay {
                queryable: vec![path("VoiceConnectorArn")?],
                ..read(
                    "VoiceConnectors",
                    Identity::Arn(path("VoiceConnectorArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListVoiceProfileDomains",
            OperationOverlay {
                queryable: vec![path("VoiceProfileDomainArn")?],
                ..read(
                    "VoiceProfileDomains",
                    Identity::Arn(path("VoiceProfileDomainArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListVoiceProfiles",
            deferred(
                "Model requires VoiceProfileDomainId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the voice profiles in a voice profile domain.",
            ),
        ),
        (
            "PutSipMediaApplicationAlexaSkillConfiguration",
            deferred(
                "Model requires SipMediaApplicationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the Alexa Skill configuration for the SIP media application. Due to changes made by the Amazon Alexa service, this API is no longer available for use. For more information, refer to the Alexa Smart Properties page.",
            ),
        ),
        (
            "PutSipMediaApplicationLoggingConfiguration",
            deferred(
                "Model requires SipMediaApplicationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the logging configuration for the specified SIP media application.",
            ),
        ),
        (
            "PutVoiceConnectorEmergencyCallingConfiguration",
            deferred(
                "Model requires VoiceConnectorId, EmergencyCallingConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a Voice Connector's emergency calling configuration.",
            ),
        ),
        (
            "PutVoiceConnectorExternalSystemsConfiguration",
            deferred(
                "Model requires VoiceConnectorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds an external systems configuration to a Voice Connector.",
            ),
        ),
        (
            "PutVoiceConnectorLoggingConfiguration",
            deferred(
                "Model requires VoiceConnectorId, LoggingConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a Voice Connector's logging configuration.",
            ),
        ),
        (
            "PutVoiceConnectorOrigination",
            deferred(
                "Model requires VoiceConnectorId, Origination. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a Voice Connector's origination settings.",
            ),
        ),
        (
            "PutVoiceConnectorProxy",
            deferred(
                "Model requires VoiceConnectorId, DefaultSessionExpiryMinutes, PhoneNumberPoolCountries. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Puts the specified proxy configuration to the specified Amazon Chime SDK Voice Connector. End of support notice: On April 7, 2026, AWS will end support for Amazon Chime SDK proxy sessions.",
            ),
        ),
        (
            "PutVoiceConnectorStreamingConfiguration",
            deferred(
                "Model requires VoiceConnectorId, StreamingConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a Voice Connector's streaming configuration settings.",
            ),
        ),
        (
            "PutVoiceConnectorTermination",
            deferred(
                "Model requires VoiceConnectorId, Termination. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a Voice Connector's termination settings.",
            ),
        ),
        (
            "PutVoiceConnectorTerminationCredentials",
            deferred(
                "Model requires VoiceConnectorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a Voice Connector's termination credentials.",
            ),
        ),
        (
            "RestorePhoneNumber",
            deferred(
                "Model requires PhoneNumberId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Restores a deleted phone number.",
            ),
        ),
        (
            "SearchAvailablePhoneNumbers",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is E164PhoneNumbers; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Searches the provisioned phone numbers in an organization.",
            ),
        ),
        (
            "StartSpeakerSearchTask",
            deferred(
                "Model requires VoiceConnectorId, TransactionId, VoiceProfileDomainId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts a speaker search task. Before starting any speaker search tasks, you must provide all notices and obtain all consents from the speaker as required under applicable privacy and biometrics laws, and as required under the AWS…",
            ),
        ),
        (
            "StartVoiceToneAnalysisTask",
            deferred(
                "Model requires VoiceConnectorId, TransactionId, LanguageCode. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts a voice tone analysis task. For more information about voice tone analysis, see Using Amazon Chime SDK voice analytics in the Amazon Chime SDK Developer Guide . Before starting any voice tone analysis tasks, you must…",
            ),
        ),
        (
            "StopSpeakerSearchTask",
            deferred(
                "Model requires VoiceConnectorId, SpeakerSearchTaskId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops a speaker search task.",
            ),
        ),
        (
            "StopVoiceToneAnalysisTask",
            deferred(
                "Model requires VoiceConnectorId, VoiceToneAnalysisTaskId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops a voice tone analysis task.",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires ResourceARN, Tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds a tag to the specified resource.",
            ),
        ),
    ])
}

pub(super) fn part_3() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "UntagResource",
            deferred(
                "Model requires ResourceARN, TagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes tags from a resource.",
            ),
        ),
        (
            "UpdateGlobalSettings",
            deferred(
                "Model requires VoiceConnector. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates global settings for the Amazon Chime SDK Voice Connectors in an AWS account.",
            ),
        ),
        (
            "UpdatePhoneNumber",
            deferred(
                "Model requires PhoneNumberId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates phone number details, such as product type, calling name, or phone number name for the specified phone number ID. You can update one phone number detail at a time. For example, you can update either the product type,…",
            ),
        ),
        (
            "UpdatePhoneNumberSettings",
            deferred(
                "Model requires CallingName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the phone number settings for the administrator's AWS account, such as the default outbound calling name. You can update the default outbound calling name once every seven days. Outbound calling names can take up to 72…",
            ),
        ),
        (
            "UpdateProxySession",
            deferred(
                "Model requires VoiceConnectorId, ProxySessionId, Capabilities. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the specified proxy session details, such as voice or SMS capabilities. End of support notice: On April 7, 2026, AWS will end support for Amazon Chime SDK proxy sessions.",
            ),
        ),
        (
            "UpdateSipMediaApplication",
            deferred(
                "Model requires SipMediaApplicationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the details of the specified SIP media application.",
            ),
        ),
        (
            "UpdateSipMediaApplicationCall",
            deferred(
                "Model requires SipMediaApplicationId, TransactionId, Arguments. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Invokes the AWS Lambda function associated with the SIP media application and transaction ID in an update request. The Lambda function can then return a new set of actions.",
            ),
        ),
        (
            "UpdateSipRule",
            deferred(
                "Model requires SipRuleId, Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the details of the specified SIP rule.",
            ),
        ),
        (
            "UpdateVoiceConnector",
            deferred(
                "Model requires VoiceConnectorId, Name, RequireEncryption. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the details for the specified Amazon Chime SDK Voice Connector.",
            ),
        ),
        (
            "UpdateVoiceConnectorGroup",
            deferred(
                "Model requires VoiceConnectorGroupId, Name, VoiceConnectorItems. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the settings for the specified Amazon Chime SDK Voice Connector group.",
            ),
        ),
        (
            "UpdateVoiceProfile",
            deferred(
                "Model requires VoiceProfileId, SpeakerSearchTaskId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the specified voice profile’s voice print and refreshes its expiration timestamp. As a condition of using this feature, you acknowledge that the collection, use, storage, and retention of your caller’s biometric…",
            ),
        ),
        (
            "UpdateVoiceProfileDomain",
            deferred(
                "Model requires VoiceProfileDomainId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the settings for the specified voice profile domain.",
            ),
        ),
        (
            "ValidateE911Address",
            deferred(
                "Model requires AwsAccountId, StreetNumber, StreetInfo, City, State, Country, PostalCode. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Validates an address to be used for 911 calls made with Amazon Chime SDK Voice Connectors. You can use validated addresses in a Presence Information Data Format Location Object file that you include in SIP requests. That helps…",
            ),
        ),
    ])
}
