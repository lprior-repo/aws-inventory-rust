use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        (
            "ListInvoiceUnits",
            policy::query(
                policy::scope(
                    policy::enumerate(
                        "InvoiceUnits",
                        policy::arn("InvoiceUnitArn")?,
                        Some(("NextToken", "NextToken")),
                    )?,
                    crate::overlay::Scope::Account,
                ),
                &["InvoiceUnitArn"],
            )?,
        ),
        (
            "ListProcurementPortalPreferences",
            policy::query(
                policy::scope(
                    policy::enumerate(
                        "ProcurementPortalPreferences",
                        policy::arn("ProcurementPortalPreferenceArn")?,
                        Some(("NextToken", "NextToken")),
                    )?,
                    crate::overlay::Scope::Account,
                ),
                &["ProcurementPortalPreferenceArn", "AwsAccountId", "Selector"],
            )?,
        ),
    ];
    policy::finish(
        "invoicing",
        approved,
        DEFERRED,
        "Account-scoped invoice-unit definitions and procurement preference summaries are approved. Full selectors contain only invoice-unit ARN and seller IDs; summary fields carry no shared secrets or invoice documents. Procurement details, PDFs, financial content and provider portal/supplier reference lists remain unapproved; mutations and validations stay denied.",
        &[(policy::DenyReason::Mutating, MUTATING)],
    )
}

const DEFERRED: &[&str] = &[
    "BatchGetInvoiceProfile",
    "GetInvoicePDF",
    "GetInvoiceUnit",
    "GetProcurementPortalPreference",
    "ListInvoiceSummaries",
    "ListProcurementPortalSuppliers",
    "ListProcurementPortals",
    "ListTagsForResource",
];

const MUTATING: &[&str] = &[
    "CreateInvoiceUnit",
    "CreateProcurementPortalPreference",
    "DeleteInvoiceUnit",
    "DeleteProcurementPortalPreference",
    "PutProcurementPortalPreference",
    "SendProcurementPortalValidation",
    "TagResource",
    "UntagResource",
    "UpdateInvoiceUnit",
    "UpdateProcurementPortalPreferenceStatus",
    "VerifyProcurementPortalValidation",
];
