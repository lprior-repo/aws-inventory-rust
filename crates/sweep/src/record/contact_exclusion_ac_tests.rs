use super::contact_exclusion_support::{ADDRESS, Case, PHONE, TestResult, assert_contact_dropped};
use aws_wire::request::Protocol;
use serde_json::json;

#[test]
fn given_certificate_when_stored_then_validation_emails_are_dropped() -> TestResult {
    let options = json!([{
        "DomainName": "example.test",
        "ValidationEmails": [ADDRESS],
        "ValidationDomain": "example.test",
        "ValidationStatus": "SUCCESS"
    }]);
    assert_contact_dropped(&Case {
        service: "acm",
        date: "2015-12-08",
        protocol: Protocol::AwsJson11,
        operation: "DescribeCertificate",
        companions: &["ListCertificates"],
        body: json!({"Certificate": {
            "CertificateArn": "arn:aws:acm:us-east-1:123456789012:certificate/abc",
            "DomainName": "example.test",
            "Status": "ISSUED",
            "DomainValidationOptions": options,
            "RenewalSummary": {
                "RenewalStatus": "PENDING_VALIDATION",
                "DomainValidationOptions": options,
                "RenewalStatusReason": "OTHER"
            }
        }}),
        rows: 1,
        removed: &[
            &["DomainValidationOptions", "ValidationEmails"],
            &[
                "RenewalSummary",
                "DomainValidationOptions",
                "ValidationEmails",
            ],
        ],
        kept: &[
            &["CertificateArn"],
            &["DomainName"],
            &["Status"],
            &["DomainValidationOptions", "ValidationDomain"],
            &["RenewalSummary", "RenewalStatus"],
            &["RenewalSummary", "DomainValidationOptions", "DomainName"],
        ],
    })
}

#[test]
fn given_certificate_authorities_when_stored_then_rfc822_name_is_dropped() -> TestResult {
    assert_contact_dropped(&Case {
        service: "acm-pca",
        date: "2017-08-22",
        protocol: Protocol::AwsJson11,
        operation: "ListCertificateAuthorities",
        companions: &[],
        body: json!({"CertificateAuthorities": [{
            "Arn": "arn:aws:acm-pca:us-east-1:123456789012:certificate-authority/ca-1",
            "Type": "ROOT",
            "Status": "ACTIVE",
            "CertificateAuthorityConfiguration": {
                "KeyAlgorithm": "RSA_2048",
                "SigningAlgorithm": "SHA256WITHRSA",
                "Subject": {"CommonName": "example.test"},
                "CsrExtensions": {"SubjectInformationAccess": [{
                    "AccessMethod": {"CustomObjectIdentifier": "1.2.3.4"},
                    "AccessLocation": {"Rfc822Name": ADDRESS, "DnsName": "example.test"}
                }]}
            }
        }]}),
        rows: 1,
        removed: &[&[
            "CertificateAuthorityConfiguration",
            "CsrExtensions",
            "SubjectInformationAccess",
            "AccessLocation",
            "Rfc822Name",
        ]],
        kept: &[
            &["Arn"],
            &["Type"],
            &["Status"],
            &["CertificateAuthorityConfiguration", "KeyAlgorithm"],
            &[
                "CertificateAuthorityConfiguration",
                "CsrExtensions",
                "SubjectInformationAccess",
                "AccessLocation",
                "DnsName",
            ],
        ],
    })
}

#[test]
fn given_b2bi_profile_when_stored_then_email_and_phone_are_dropped() -> TestResult {
    assert_contact_dropped(&Case {
        service: "b2bi",
        date: "2022-06-23",
        protocol: Protocol::AwsJson10,
        operation: "GetProfile",
        companions: &["ListProfiles"],
        body: json!({
            "profileId": "p-1",
            "profileArn": "arn:aws:b2bi:us-east-1:123456789012:profile/p-1",
            "name": "billing",
            "email": ADDRESS,
            "phone": PHONE,
            "businessName": "Example",
            "logging": "ENABLED"
        }),
        rows: 1,
        removed: &[&["email"], &["phone"]],
        kept: &[
            &["profileId"],
            &["profileArn"],
            &["name"],
            &["businessName"],
            &["logging"],
        ],
    })
}

#[test]
fn given_b2bi_partnership_when_stored_then_email_and_phone_are_dropped() -> TestResult {
    assert_contact_dropped(&Case {
        service: "b2bi",
        date: "2022-06-23",
        protocol: Protocol::AwsJson10,
        operation: "GetPartnership",
        companions: &["ListPartnerships"],
        body: json!({
            "partnershipId": "ps-1",
            "partnershipArn": "arn:aws:b2bi:us-east-1:123456789012:partnership/ps-1",
            "profileId": "p-1",
            "name": "trading-partner",
            "email": ADDRESS,
            "phone": PHONE
        }),
        rows: 1,
        removed: &[&["email"], &["phone"]],
        kept: &[
            &["partnershipId"],
            &["partnershipArn"],
            &["profileId"],
            &["name"],
        ],
    })
}

#[test]
fn given_chime_voice_numbers_when_stored_then_e164_is_dropped() -> TestResult {
    assert_contact_dropped(&Case {
        service: "chime-sdk-voice",
        date: "2022-08-03",
        protocol: Protocol::RestJson1,
        operation: "ListPhoneNumbers",
        companions: &[],
        body: json!({"PhoneNumbers": [{
            "PhoneNumberId": "pn-1",
            "PhoneNumberArn": "arn:aws:chime:us-east-1:123456789012:phone-number/pn-1",
            "E164PhoneNumber": PHONE,
            "Country": "US",
            "Type": "Local",
            "Status": "Assigned"
        }]}),
        rows: 1,
        removed: &[&["E164PhoneNumber"]],
        kept: &[&["PhoneNumberArn"], &["Country"], &["Type"], &["Status"]],
    })
}

#[test]
fn given_connect_phone_numbers_when_stored_then_number_is_dropped() -> TestResult {
    assert_contact_dropped(&Case {
        service: "connect",
        date: "2017-08-08",
        protocol: Protocol::RestJson1,
        operation: "ListPhoneNumbersV2",
        companions: &[],
        body: json!({"ListPhoneNumbersSummaryList": [{
            "PhoneNumberId": "pn-1",
            "PhoneNumberArn": "arn:aws:connect:us-east-1:123456789012:phone-number/pn-1",
            "PhoneNumber": PHONE,
            "PhoneNumberCountryCode": "US",
            "PhoneNumberType": "DID",
            "TargetArn": "arn:aws:connect:us-east-1:123456789012:instance/i-1"
        }]}),
        rows: 1,
        removed: &[&["PhoneNumber"]],
        kept: &[
            &["PhoneNumberId"],
            &["PhoneNumberArn"],
            &["PhoneNumberCountryCode"],
            &["PhoneNumberType"],
            &["TargetArn"],
        ],
    })
}

#[test]
fn given_connect_campaign_when_stored_then_source_phone_is_dropped() -> TestResult {
    assert_contact_dropped(&Case {
        service: "connectcampaigns",
        date: "2021-01-30",
        protocol: Protocol::RestJson1,
        operation: "DescribeCampaign",
        companions: &["ListCampaigns"],
        body: json!({"campaign": {
            "id": "c-1",
            "arn": "arn:aws:connect-campaigns:us-east-1:123456789012:campaign/c-1",
            "name": "outbound",
            "connectInstanceId": "i-1",
            "dialerConfig": {"agentlessDialerConfig": {}},
            "outboundCallConfig": {
                "connectContactFlowId": "f-1",
                "connectSourcePhoneNumber": PHONE,
                "connectQueueId": "q-1"
            }
        }}),
        rows: 1,
        removed: &[&["outboundCallConfig", "connectSourcePhoneNumber"]],
        kept: &[
            &["id"],
            &["arn"],
            &["name"],
            &["outboundCallConfig", "connectContactFlowId"],
            &["outboundCallConfig", "connectQueueId"],
        ],
    })
}

#[test]
fn given_connect_campaign_v2_when_stored_then_source_email_and_phone_are_dropped() -> TestResult {
    assert_contact_dropped(&Case {
        service: "connectcampaignsv2",
        date: "2024-04-23",
        protocol: Protocol::RestJson1,
        operation: "DescribeCampaign",
        companions: &["ListCampaigns"],
        body: json!({"campaign": {
            "id": "c-1",
            "arn": "arn:aws:connect-campaigns:us-east-1:123456789012:campaign/c-1",
            "name": "outbound",
            "connectInstanceId": "i-1",
            "channelSubtypeConfig": {
                "telephony": {
                    "connectQueueId": "q-1",
                    "outboundMode": {"agentless": {}},
                    "defaultOutboundConfig": {
                        "connectContactFlowId": "f-1",
                        "connectSourcePhoneNumber": PHONE
                    }
                },
                "email": {
                    "outboundMode": {"agentless": {}},
                    "defaultOutboundConfig": {
                        "connectSourceEmailAddress": ADDRESS,
                        "wisdomTemplateArn": "arn:aws:wisdom:us-east-1:123456789012:template/t-1"
                    }
                }
            }
        }}),
        rows: 1,
        removed: &[
            &[
                "channelSubtypeConfig",
                "telephony",
                "defaultOutboundConfig",
                "connectSourcePhoneNumber",
            ],
            &[
                "channelSubtypeConfig",
                "email",
                "defaultOutboundConfig",
                "connectSourceEmailAddress",
            ],
        ],
        kept: &[
            &["id"],
            &["arn"],
            &["name"],
            &["channelSubtypeConfig", "telephony", "connectQueueId"],
            &[
                "channelSubtypeConfig",
                "telephony",
                "defaultOutboundConfig",
                "connectContactFlowId",
            ],
            &[
                "channelSubtypeConfig",
                "email",
                "defaultOutboundConfig",
                "wisdomTemplateArn",
            ],
        ],
    })
}
