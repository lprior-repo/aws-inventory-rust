use super::contact_exclusion_support::{ADDRESS, Case, TestResult, assert_contact_dropped};
use aws_wire::request::Protocol;
use serde_json::json;

const MEMBER_KEPT: &[&[&str]] = &[&["AccountId"], &["GraphArn"], &["Status"]];

#[test]
fn given_detective_invitations_when_stored_then_email_is_dropped() -> TestResult {
    assert_contact_dropped(&Case {
        service: "detective",
        date: "2018-10-26",
        protocol: Protocol::RestJson1,
        operation: "ListInvitations",
        companions: &[],
        body: json!({"Invitations": [{
            "AccountId": "111122223333",
            "EmailAddress": ADDRESS,
            "GraphArn": "arn:aws:detective:us-east-1:123456789012:graph:abcdef",
            "Status": "INVITED"
        }]}),
        rows: 1,
        removed: &[&["EmailAddress"]],
        kept: MEMBER_KEPT,
    })
}

#[test]
fn given_detective_members_when_stored_then_email_is_dropped() -> TestResult {
    assert_contact_dropped(&Case {
        service: "detective",
        date: "2018-10-26",
        protocol: Protocol::RestJson1,
        operation: "ListMembers",
        companions: &["ListGraphs"],
        body: json!({"MemberDetails": [{
            "AccountId": "111122223333",
            "EmailAddress": ADDRESS,
            "GraphArn": "arn:aws:detective:us-east-1:123456789012:graph:abcdef",
            "Status": "ENABLED"
        }]}),
        rows: 1,
        removed: &[&["EmailAddress"]],
        kept: MEMBER_KEPT,
    })
}

#[test]
fn given_macie_members_when_stored_then_email_is_dropped() -> TestResult {
    assert_contact_dropped(&Case {
        service: "macie2",
        date: "2020-01-01",
        protocol: Protocol::RestJson1,
        operation: "ListMembers",
        companions: &[],
        body: json!({"members": [{
            "accountId": "111122223333",
            "arn": "arn:aws:macie2:us-east-1:123456789012:member/111122223333",
            "email": ADDRESS,
            "relationshipStatus": "Enabled"
        }]}),
        rows: 1,
        removed: &[&["email"]],
        kept: &[&["accountId"], &["arn"], &["relationshipStatus"]],
    })
}

const CONTACT_KEPT: &[&[&str]] = &[&["arn"], &["name"], &["status"]];

#[test]
fn given_notification_email_contacts_when_listed_then_address_is_dropped() -> TestResult {
    assert_contact_dropped(&Case {
        service: "notificationscontacts",
        date: "2018-05-10",
        protocol: Protocol::RestJson1,
        operation: "ListEmailContacts",
        companions: &[],
        body: json!({"emailContacts": [{
            "arn": "arn:aws:notifications-contacts::123456789012:emailcontact/ec-1",
            "name": "oncall",
            "address": ADDRESS,
            "status": "active"
        }]}),
        rows: 1,
        removed: &[&["address"]],
        kept: CONTACT_KEPT,
    })
}

#[test]
fn given_notification_email_contact_when_described_then_address_is_dropped() -> TestResult {
    assert_contact_dropped(&Case {
        service: "notificationscontacts",
        date: "2018-05-10",
        protocol: Protocol::RestJson1,
        operation: "GetEmailContact",
        companions: &["ListEmailContacts"],
        body: json!({"emailContact": {
            "arn": "arn:aws:notifications-contacts::123456789012:emailcontact/ec-1",
            "name": "oncall",
            "address": ADDRESS,
            "status": "active"
        }}),
        rows: 1,
        removed: &[&["address"]],
        kept: CONTACT_KEPT,
    })
}
