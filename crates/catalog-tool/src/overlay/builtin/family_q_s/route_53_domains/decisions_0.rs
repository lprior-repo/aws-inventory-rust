use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "AcceptDomainTransferFromAnotherAwsAccount",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "AssociateDelegationSignerToDomain",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CancelDomainTransferToAnotherAwsAccount",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CheckDomainAvailability",
        Decision::Deferred(
            "Required selectors DomainName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Availability, Price. Pinned purpose: This operation checks the availability of one domain name. Note that if the availability status of a domain is pending, you must submit another request to determine the availability of the domain name.",
        ),
    ),
    (
        "CheckDomainTransferability",
        Decision::Deferred(
            "Required selectors DomainName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Transferability, Message, Price. Pinned purpose: Checks whether a domain name can be transferred to Amazon Route 53.",
        ),
    ),
    ("DeleteDomain", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteTagsForDomain",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisableDomainAutoRenew",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisableDomainTransferLock",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisassociateDelegationSignerFromDomain",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "EnableDomainAutoRenew",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "EnableDomainTransferLock",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetContactReachabilityStatus",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members domainName, status. Pinned request/ownership contract: For operations that require confirmation that the email address for the registrant contact is valid, such as registering a new domain, this operation returns information about whether the registrant contact has responded. If you want us to resend the email, use the ResendContactReachabilityEmail operation.",
        ),
    ),
    (
        "GetDomainDetail",
        Decision::Deferred(
            "Required selectors DomainName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DomainName, Nameservers, AutoRenew, AdminContact, RegistrantContact, TechContact, AdminPrivacy, RegistrantPrivacy, TechPrivacy, RegistrarName, WhoIsServer, RegistrarUrl, AbuseContactEmail, AbuseContactPhone, RegistryDomainId, CreationDate, UpdatedDate, ExpirationDate, Reseller, DnsSec, StatusList, DnssecKeys, BillingContact, BillingPrivacy. Pinned purpose: This operation returns detailed information about a specified domain that is associated with the current Amazon Web Services account. Contact information for the domain is also returned as part of the output.",
        ),
    ),
    (
        "GetDomainPrices",
        Decision::Deferred(
            "Required selectors DomainName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DomainPrice. Pinned purpose: Returns the following prices for the specified domain: Registration Transfer Owner change Renewal Restoration For most domains, the prices match the standard prices for the TLD that the ListPrices operation returns. For a premium domain, the registry",
        ),
    ),
    (
        "GetDomainSuggestions",
        Decision::Deferred(
            "Required selectors DomainName, SuggestionCount, OnlyAvailable are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members SuggestionsList. Pinned purpose: The GetDomainSuggestions operation returns a list of suggested domain names.",
        ),
    ),
    (
        "GetOperationDetail",
        Decision::Deferred(
            "Required selectors OperationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members OperationId, Status, Message, DomainName, Type, SubmittedDate, LastUpdatedDate, StatusFlag. Pinned purpose: This operation returns the current status of an operation that is not completed.",
        ),
    ),
    (
        "ListDomains",
        Decision::Deferred(
            "route-53-domains has fixed control-plane routing (us-east-1), while the pinned endpoint rules are parameterized by Region and Account/HomeRegion uses caller home. Requires service-specific fixed routing; regional fan-out would misstate scope or fail.",
        ),
    ),
    (
        "ListOperations",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members Operations, NextPageMarker. Pinned request/ownership contract: Returns information about all of the operations that return an operation ID and that have ever been performed on domains that were registered by the current account. This command runs only in the us-east-1 Region.",
        ),
    ),
    ("ListPrices", Decision::Denied(DenyReason::ReferenceData)),
    (
        "ListTagsForDomain",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: DomainName. Pinned purpose: This operation returns all of the tags that are associated with the specified domain. All tag operations are eventually consistent; subsequent operations might not immediately represent all issued operations.",
        ),
    ),
    (
        "PushDomain",
        Decision::Deferred(
            "Required selectors DomainName, Target are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members . Pinned purpose: Moves a domain from Amazon Web Services to another registrar. Supported actions: Changes the IPS tags of a .uk domain, and pushes it to transit. Transit means that the domain is ready to be transferred to another registrar.",
        ),
    ),
    ("RegisterDomain", Decision::Denied(DenyReason::Mutating)),
    (
        "RejectDomainTransferFromAnotherAwsAccount",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "RenewDomain",
        Decision::Deferred(
            "Required selectors DomainName, CurrentExpiryYear are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members OperationId. Pinned purpose: This operation renews a domain for the specified number of years. The cost of renewing your domain is billed to your Amazon Web Services account. We recommend that you renew your domain several weeks before the expiration date. Some TLD registries de",
        ),
    ),
    (
        "ResendContactReachabilityEmail",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members domainName, emailAddress, isAlreadyVerified. Pinned request/ownership contract: For operations that require confirmation that the email address for the registrant contact is valid, such as registering a new domain, this operation resends the confirmation email to the current email address for the registrant contact.",
        ),
    ),
    (
        "ResendOperationAuthorization",
        Decision::Deferred(
            "Required selectors OperationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members . Pinned purpose: Resend the form of authorization email for this operation.",
        ),
    ),
    ("RestoreDomain", Decision::Denied(DenyReason::Mutating)),
    (
        "RetrieveDomainAuthCode",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "TransferDomain",
        Decision::Deferred(
            "Required selectors DomainName, AdminContact, RegistrantContact, TechContact are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members OperationId. Pinned purpose: Transfers a domain from another registrar to Amazon Route 53. For more information about transferring domains, see the following topics: For transfer requirements, a detailed procedure, and information about viewing the status of a domain that you're",
        ),
    ),
    (
        "TransferDomainToAnotherAwsAccount",
        Decision::Denied(DenyReason::Secret),
    ),
    (
        "UpdateDomainContact",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateDomainContactPrivacy",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateDomainNameservers",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateTagsForDomain",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ViewBilling",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members NextPageMarker, BillingRecords. Pinned request/ownership contract: Returns all the domain-related billing records for the current Amazon Web Services account for a specified period",
        ),
    ),
];
