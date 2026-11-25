use super::contact_exclusion_support::{ADDRESS, Case, PHONE, TestResult, assert_contact_dropped};
use aws_wire::request::Protocol;
use serde_json::{Value, json};

#[test]
fn given_odb_autonomous_databases_when_stored_then_customer_emails_are_dropped() -> TestResult {
    assert_contact_dropped(&Case {
        service: "odb",
        date: "2024-08-20",
        protocol: Protocol::AwsJson10,
        operation: "ListAutonomousDatabases",
        companions: &[],
        body: json!({"autonomousDatabases": [{
            "autonomousDatabaseId": "adb-1",
            "autonomousDatabaseArn": "arn:aws:odb:us-east-1:123456789012:autonomous-database/adb-1",
            "displayName": "orders",
            "status": "AVAILABLE",
            "customerContacts": [{"email": ADDRESS}]
        }]}),
        rows: 1,
        removed: &[&["customerContacts", "email"]],
        kept: &[
            &["autonomousDatabaseId"],
            &["autonomousDatabaseArn"],
            &["displayName"],
            &["status"],
        ],
    })
}

#[test]
fn given_odb_exadata_infrastructures_when_stored_then_customer_emails_are_dropped() -> TestResult {
    assert_contact_dropped(&Case {
        service: "odb",
        date: "2024-08-20",
        protocol: Protocol::AwsJson10,
        operation: "ListCloudExadataInfrastructures",
        companions: &[],
        body: json!({"cloudExadataInfrastructures": [{
            "cloudExadataInfrastructureId": "exa-1",
            "cloudExadataInfrastructureArn": "arn:aws:odb:us-east-1:123456789012:cloud-exadata-infrastructure/exa-1",
            "displayName": "primary",
            "status": "AVAILABLE",
            "customerContactsToSendToOCI": [{"email": ADDRESS}]
        }]}),
        rows: 1,
        removed: &[&["customerContactsToSendToOCI", "email"]],
        kept: &[
            &["cloudExadataInfrastructureId"],
            &["cloudExadataInfrastructureArn"],
            &["displayName"],
            &["status"],
        ],
    })
}

#[test]
fn given_sms_phone_numbers_when_stored_then_number_is_dropped() -> TestResult {
    assert_contact_dropped(&Case {
        service: "pinpoint-sms-voice-v2",
        date: "2022-03-31",
        protocol: Protocol::AwsJson10,
        operation: "DescribePhoneNumbers",
        companions: &[],
        body: json!({"PhoneNumbers": [{
            "PhoneNumberArn": "arn:aws:sms-voice:us-east-1:123456789012:phone-number/pn-1",
            "PhoneNumberId": "pn-1",
            "PhoneNumber": PHONE,
            "Status": "ACTIVE",
            "IsoCountryCode": "US",
            "NumberType": "LONG_CODE"
        }]}),
        rows: 1,
        removed: &[&["PhoneNumber"]],
        kept: &[
            &["PhoneNumberArn"],
            &["PhoneNumberId"],
            &["Status"],
            &["IsoCountryCode"],
            &["NumberType"],
        ],
    })
}

#[test]
fn given_sms_verified_destinations_when_stored_then_number_is_dropped() -> TestResult {
    assert_contact_dropped(&Case {
        service: "pinpoint-sms-voice-v2",
        date: "2022-03-31",
        protocol: Protocol::AwsJson10,
        operation: "DescribeVerifiedDestinationNumbers",
        companions: &[],
        body: json!({"VerifiedDestinationNumbers": [{
            "VerifiedDestinationNumberArn": "arn:aws:sms-voice:us-east-1:123456789012:verified-destination-number/vdn-1",
            "VerifiedDestinationNumberId": "vdn-1",
            "DestinationPhoneNumber": PHONE,
            "Status": "VERIFIED"
        }]}),
        rows: 1,
        removed: &[&["DestinationPhoneNumber"]],
        kept: &[
            &["VerifiedDestinationNumberArn"],
            &["VerifiedDestinationNumberId"],
            &["Status"],
        ],
    })
}

#[test]
fn given_security_hub_members_when_stored_then_email_is_dropped() -> TestResult {
    assert_contact_dropped(&Case {
        service: "securityhub",
        date: "2018-10-26",
        protocol: Protocol::RestJson1,
        operation: "ListMembers",
        companions: &[],
        body: json!({"Members": [{
            "AccountId": "111122223333",
            "Email": ADDRESS,
            "AdministratorId": "444455556666",
            "MemberStatus": "ENABLED"
        }]}),
        rows: 1,
        removed: &[&["Email"]],
        kept: &[&["AccountId"], &["AdministratorId"], &["MemberStatus"]],
    })
}

fn template_case(
    service: &'static str,
    date: &'static str,
    protocol: Protocol,
    body: Value,
) -> Case {
    Case {
        service,
        date,
        protocol,
        operation: "ListCustomVerificationEmailTemplates",
        companions: &[],
        body,
        rows: 1,
        removed: &[&["FromEmailAddress"]],
        kept: &[
            &["TemplateName"],
            &["TemplateSubject"],
            &["SuccessRedirectionURL"],
        ],
    }
}

#[test]
fn given_ses_templates_when_stored_then_from_address_is_dropped() -> TestResult {
    let xml = format!(
        "<ListCustomVerificationEmailTemplatesResponse><ListCustomVerificationEmailTemplatesResult><CustomVerificationEmailTemplates><member><TemplateName>welcome</TemplateName><FromEmailAddress>{ADDRESS}</FromEmailAddress><TemplateSubject>Verify your address</TemplateSubject><SuccessRedirectionURL>https://example.test/ok</SuccessRedirectionURL><FailureRedirectionURL>https://example.test/fail</FailureRedirectionURL></member></CustomVerificationEmailTemplates></ListCustomVerificationEmailTemplatesResult></ListCustomVerificationEmailTemplatesResponse>"
    );
    assert_contact_dropped(&template_case(
        "ses",
        "2010-12-01",
        Protocol::AwsQuery,
        Value::String(xml),
    ))
}

#[test]
fn given_sesv2_templates_when_stored_then_from_address_is_dropped() -> TestResult {
    let body = json!({"CustomVerificationEmailTemplates": [{
        "TemplateName": "welcome",
        "FromEmailAddress": ADDRESS,
        "TemplateSubject": "Verify your address",
        "SuccessRedirectionURL": "https://example.test/ok",
        "FailureRedirectionURL": "https://example.test/fail"
    }]});
    assert_contact_dropped(&template_case(
        "sesv2",
        "2019-09-27",
        Protocol::RestJson1,
        body,
    ))
}
#[test]
fn given_sesv2_deliverability_reports_when_stored_then_from_address_is_dropped() -> TestResult {
    assert_contact_dropped(&Case {
        service: "sesv2",
        date: "2019-09-27",
        protocol: Protocol::RestJson1,
        operation: "ListDeliverabilityTestReports",
        companions: &[],
        body: json!({"DeliverabilityTestReports": [{
            "ReportId": "r-1",
            "ReportName": "launch",
            "Subject": "Test",
            "FromEmailAddress": ADDRESS,
            "DeliverabilityTestStatus": "COMPLETED"
        }]}),
        rows: 1,
        removed: &[&["FromEmailAddress"]],
        kept: &[
            &["ReportId"],
            &["ReportName"],
            &["DeliverabilityTestStatus"],
        ],
    })
}

#[test]
fn given_ssm_contact_channels_when_stored_then_simple_address_is_dropped() -> TestResult {
    assert_contact_dropped(&Case {
        service: "ssm-contacts",
        date: "2021-05-03",
        protocol: Protocol::AwsJson11,
        operation: "ListContactChannels",
        companions: &["ListContacts"],
        body: json!({"ContactChannels": [{
            "ContactChannelArn": "arn:aws:ssm-contacts:us-east-1:123456789012:contact-channel/c/ch-1",
            "ContactArn": "arn:aws:ssm-contacts:us-east-1:123456789012:contact/c",
            "Name": "email",
            "Type": "EMAIL",
            "DeliveryAddress": {"SimpleAddress": ADDRESS},
            "ActivationStatus": "ACTIVATED"
        }]}),
        rows: 1,
        removed: &[&["DeliveryAddress", "SimpleAddress"]],
        kept: &[
            &["ContactChannelArn"],
            &["ContactArn"],
            &["Type"],
            &["ActivationStatus"],
        ],
    })
}

fn workmail_case(operation: &'static str, items: &'static str, kept: &'static [&[&str]]) -> Case {
    Case {
        service: "workmail",
        date: "2017-10-01",
        protocol: Protocol::AwsJson11,
        operation,
        companions: &["ListOrganizations"],
        body: json!({items: [{
            "Id": "m-1",
            "Email": ADDRESS,
            "Name": "entity",
            "State": "ENABLED"
        }]}),
        rows: 1,
        removed: &[&["Email"]],
        kept,
    }
}

#[test]
fn given_workmail_entities_when_stored_then_email_is_dropped() -> TestResult {
    let kept: &[&[&str]] = &[&["Id"], &["Name"], &["State"]];
    for (operation, items) in [
        ("ListGroups", "Groups"),
        ("ListUsers", "Users"),
        ("ListResources", "Resources"),
    ] {
        assert_contact_dropped(&workmail_case(operation, items, kept))?;
    }
    Ok(())
}
