// Pinned operation purposes: .local/aws-models/models/route-53-domains/service/2014-05-15/route-53-domains-2014-05-15.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Accepts the transfer of a domain from another Amazon Web Services account to the currentAmazon Web Services account. You initiate a transfer between Amazon Web Services accounts using TransferDomainToAnotherAwsAccount . If you use the CLI command at accept-domain-transfer-from-another-aws-account , use JSON format as input instead of text because otherwise CLI will throw an error from domain trans
    (
        "AcceptDomainTransferFromAnotherAwsAccount",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a delegation signer (DS) record in the registry zone for this domain name. Note that creating DS record at the registry impacts DNSSEC validation of your DNS records. This action may render your domain name unavailable on the internet if the steps are completed in the wrong order, or with incorrect timing. For more information about DNSSEC signing, see Configuring DNSSEC signing in the Rou
    (
        "AssociateDelegationSignerToDomain",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Cancels the transfer of a domain from the current Amazon Web Services account to another Amazon Web Services account. You initiate a transfer betweenAmazon Web Services accounts using TransferDomainToAnotherAwsAccount . You must cancel the transfer before the other Amazon Web Services account accepts the transfer using AcceptDomainTransferFromAnotherAwsAccount . Use either ListOperations or GetOpe
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
    // Changes service state rather than reading inventory. Pinned purpose: This operation deletes the specified domain. This action is permanent. For more information, see Deleting a domain name registration . To transfer the domain registration to another registrar, use the transfer process that’s provided by the registrar to which you want to transfer the registration. Otherwise, the following apply: You can’t get a refund for the cost of a deleted domain registration.
    ("DeleteDomain", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: This operation deletes the specified tags for a domain. All tag operations are eventually consistent; subsequent operations might not immediately represent all issued operations.
    (
        "DeleteTagsForDomain",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: This operation disables automatic renewal of domain registration for the specified domain.
    (
        "DisableDomainAutoRenew",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: This operation removes the transfer lock on the domain (specifically the clientTransferProhibited status) to allow domain transfers. We recommend you refrain from performing this action unless you intend to transfer the domain to a different registrar. Successful submission returns an operation ID that you can use to track the progress and completion of the action. If the request is not completed
    (
        "DisableDomainTransferLock",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a delegation signer (DS) record in the registry zone for this domain name.
    (
        "DisassociateDelegationSignerFromDomain",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: This operation configures Amazon Route 53 to automatically renew the specified domain before the domain registration expires. The cost of renewing your domain registration is billed to your Amazon Web Services account. The period during which you can renew a domain name varies by TLD. For a list of TLDs and their renewal policies, see Domains That You Can Register with Amazon Route 53 in the Amazo
    (
        "EnableDomainAutoRenew",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: This operation sets the transfer lock on the domain (specifically the clientTransferProhibited status) to prevent domain transfers. Successful submission returns an operation ID that you can use to track the progress and completion of the action. If the request is not completed successfully, the domain registrant will be notified by email.
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
    // Public/service reference catalog or supported-capability enumeration, not account-owned resource inventory. Pinned purpose: Lists the following prices for either all the TLDs supported by Route 53, or the specified TLD: Registration Transfer Owner change Renewal Restoration The prices that ListPrices returns are the standard rates for a TLD. ListPrices doesn't return registry premium prices, which apply to individual domain names rather than to a TLD. To get accurate pr
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
    // Changes service state rather than reading inventory. Pinned purpose: This operation registers a domain. For some top-level domains (TLDs), this operation requires extra parameters. When you register a domain, Amazon Route 53 does the following: Creates a Route 53 hosted zone that has the same name as the domain. Route 53 assigns four name servers to your hosted zone and automatically updates your domain registration with the names of these name servers. Enables aut
    ("RegisterDomain", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Rejects the transfer of a domain from another Amazon Web Services account to the current Amazon Web Services account. You initiate a transfer betweenAmazon Web Services accounts using TransferDomainToAnotherAwsAccount . Use either ListOperations or GetOperationDetail to determine whether the operation succeeded. GetOperationDetail provides additional information, for example, Domain Transfer from
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
    // Changes service state rather than reading inventory. Pinned purpose: Restores a domain that has expired, or was deleted, before it becomes available for others to register. The cost of restoring your domain is billed to your Amazon Web Services account. Not every TLD registry supports restoring domains, and the period during which a domain can be restored varies by TLD. To find out whether your domain can be restored and the period during which you can restore it,
    ("RestoreDomain", Decision::Denied(DenyReason::Mutating)),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: This operation returns the authorization code for the domain. To transfer a domain to another registrar, you provide this value to the new registrar.
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
    // Full modeled response contains authentication/secret material Password. Pinned purpose: Transfers a domain from the current Amazon Web Services account to another Amazon Web Services account. Note the following: The Amazon Web Services account that you're transferring the domain to must accept the transfer. If the other account doesn't accept the transfer within 3 days, we cancel the t. Projection cannot authorize this call.
    (
        "TransferDomainToAnotherAwsAccount",
        Decision::Denied(DenyReason::Secret),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: This operation updates the contact information for a particular domain. You must specify information for at least one contact: registrant, administrator, or technical. If the update is successful, this method returns an operation ID that you can use to track the progress and completion of the operation. If the request is not completed successfully, the domain registrant will be notified by email.
    (
        "UpdateDomainContact",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: This operation updates the specified domain contact's privacy setting. When privacy protection is enabled, your contact information is replaced with contact information for the registrar or with the phrase "REDACTED FOR PRIVACY", or "On behalf of owner." While some domains may allow different privacy settings per contact, we recommend specifying the same privacy setting for all contacts. This oper
    (
        "UpdateDomainContactPrivacy",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: This operation replaces the current set of name servers for the domain with the specified set of name servers. If you use Amazon Route 53 as your DNS service, specify the four name servers in the delegation set for the hosted zone for the domain. If successful, this operation returns an operation ID that you can use to track the progress and completion of the action. If the request is not complete
    (
        "UpdateDomainNameservers",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: This operation adds or updates tags for a specified domain. All tag operations are eventually consistent; subsequent operations might not immediately represent all issued operations.
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
