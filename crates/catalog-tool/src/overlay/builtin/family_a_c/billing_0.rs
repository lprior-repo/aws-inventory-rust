use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "AssociateContract",
            deferred(
                "Model requires contractArn, contractDocumentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Private Pricing contracts let eligible payer accounts manage private pricing agreements with Amazon Web Services. This operation associates a private pricing contract with the caller's Amazon Web Services account. The caller must…",
            ),
        ),
        (
            "AssociateSourceViews",
            deferred(
                "Model requires arn, sourceViews. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates one or more source billing views with an existing billing view. This allows creating aggregate billing views that combine data from multiple sources.",
            ),
        ),
        (
            "CancelContractualInvestmentAttestation",
            deferred(
                "Model requires attestationId, contractArn, reason. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Cancels a contractual investment attestation. This operation is idempotent, so retrying is safe. You can cancel an attestation only while it is still pending (its isCancellable value is true ).",
            ),
        ),
        (
            "CreateBillingView",
            deferred(
                "Model requires name, sourceViews. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a billing view with the specified billing view attributes.",
            ),
        ),
        (
            "CreateContractualInvestmentAttestation",
            deferred(
                "Model requires contractArn, investmentSummaryId, investmentId, contractDocumentId, approverDetails, attestationDetails. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an attestation that claims an investment award for a completed milestone or outcome within a private pricing contract. Supply a clientToken to make the request idempotent so a safe retry does not create a duplicate…",
            ),
        ),
        (
            "DeleteBillingView",
            deferred(
                "Model requires arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified billing view.",
            ),
        ),
        (
            "DisassociateContract",
            deferred(
                "Model requires contractArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes the association between a private pricing contract and your Amazon Web Services account. The account must have been associated with the contract previously. This operation is idempotent, so retrying is safe.",
            ),
        ),
        (
            "DisassociateSourceViews",
            deferred(
                "Model requires arn, sourceViews. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes the association between one or more source billing views and an existing billing view. This allows modifying the composition of aggregate billing views.",
            ),
        ),
        (
            "GetBillingPreferences",
            deferred(
                "Model requires features. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves billing preferences for the specified feature. Each feature controls a distinct billing capability: which accounts can share Reserved Instances or credits, whether billing alerts are enabled, the historical record of…",
            ),
        ),
        (
            "GetBillingView",
            deferred(
                "Model requires arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the metadata associated to the specified billing view ARN.",
            ),
        ),
        (
            "GetContractInformation",
            deferred(
                "Model requires contractArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about a specific private pricing contract, including its term (duration), eligible payer accounts, investment accounts, and legal entities. This operation is read-only.",
            ),
        ),
        (
            "GetCreditAllocationHistory",
            deferred(
                "Model requires accountId, startDate, endDate. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the per-billing-month allocation history for credits applied to an Amazon Web Services account's bills. Traverses the consolidated billing family to capture cross-account credit applications. Supports pagination and…",
            ),
        ),
        (
            "GetCredits",
            deferred(
                "Model requires accountId, startDate. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the list of Amazon Web Services account credits for the specified account. Each credit includes its identifier, type, monetary amounts, applicable products, expiration, sharing configuration, and current enabled status.…",
            ),
        ),
        (
            "GetEnterpriseSupportChargeSummary",
            deferred(
                "Model requires billingMonth. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a summary of Enterprise Support data aggregated across all accounts in the Enterprise Support profile.",
            ),
        ),
        (
            "GetEnterpriseSupportContractDetails",
            deferred(
                "Model requires billingMonth. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns Enterprise Support contract details.",
            ),
        ),
        (
            "GetResourcePolicy",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the resource-based policy document attached to the resource in JSON format.",
            ),
        ),
        (
            "ListBillingViewSegments",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is items; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists the segments of a billing view over a given time period. Each segment identifies the billing domain ( PRO_FORMA or BILLABLE ) and the account relationships that apply during its time range. If you don't provide an arn , the…",
            ),
        ),
        (
            "ListBillingViews",
            OperationOverlay {
                queryable: vec![path("arn")?],
                ..read(
                    "billingViews",
                    Identity::Arn(path("arn")?),
                    Scope::Account,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListBusinessSupportAccountCharges",
            deferred(
                "Model requires billingMonth. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns Business Support charges broken down at the linked account level for a given billing month.",
            ),
        ),
        (
            "ListBusinessSupportSubscriptionHistory",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is subscriptionContracts; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns the history of Business Support subscription contracts across accounts.",
            ),
        ),
        (
            "ListContracts",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is items; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists the private pricing contracts that your Amazon Web Services account is an eligible payer account of. This operation is read-only and paginated.",
            ),
        ),
        (
            "ListContractualInvestmentAttestations",
            deferred(
                "Model requires contractArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the investment attestations submitted under a specific private pricing contract. Use this operation to track attestation status, including the terminal statuses DISBURSED , REJECTED , and CANCELLED . This operation is…",
            ),
        ),
        (
            "ListContractualInvestmentSummaries",
            deferred(
                "Model requires contractArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the investment summaries within a specific private pricing contract. This operation is read-only and paginated.",
            ),
        ),
        (
            "ListContractualInvestments",
            deferred(
                "Model requires contractArn, investmentSummaryId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the investments within a specific investment summary. This operation is read-only and paginated.",
            ),
        ),
        (
            "ListEnterpriseSupportLinkedAccountCharges",
            deferred(
                "Model requires billingMonth. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns Support-eligible spend broken down at linked account level.",
            ),
        ),
        (
            "ListSourceViewsForBillingView",
            deferred(
                "Model requires arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the source views (managed Amazon Web Services billing views) associated with the billing view.",
            ),
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists tags associated with the billing view resource.",
            ),
        ),
        (
            "RedeemCredits",
            deferred(
                "Model requires promoCode. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Redeems an Amazon Web Services promotional credit code on behalf of the calling account. On success, a new credit is added to the account's credit ledger with the amount, validity period, and applicable products defined by the…",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires resourceArn, resourceTags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: An API operation for adding one or more tags (key-value pairs) to a resource.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires resourceArn, resourceTagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes one or more tags from a resource. Specify only tag keys in your request. Don't specify the value.",
            ),
        ),
        (
            "UpdateBillingPreferences",
            deferred(
                "Model requires feature, billingPreferencesPerKey. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates billing preferences for the specified feature. Each feature targets a distinct billing capability and has its own set of supported keys. The action sets the value for each provided key; keys not present in the request are…",
            ),
        ),
        (
            "UpdateBillingView",
            deferred(
                "Model requires arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: An API to update the attributes of the billing view.",
            ),
        ),
    ])
}
