use super::support::*;
use crate::overlay::{TagEndpoint, TagEndpointParent, TagEndpointRequest, TagEndpointSelector};

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "AddTagsToCertificate",
            deferred(
                "Model requires CertificateArn, Tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds one or more tags to an ACM certificate. Tags are labels that you can use to identify and organize your Amazon Web Services resources. Each tag consists of a key and an optional value . You specify the certificate on input by…",
            ),
        ),
        (
            "CreateAcmeDomainValidation",
            deferred(
                "Model requires AcmeEndpointArn, DomainName, PrevalidationOptions. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a domain validation for an ACME endpoint. Domain validations authorize the endpoint to issue certificates for specified domain names. You configure prevalidation to prove domain ownership.",
            ),
        ),
        (
            "CreateAcmeEndpoint",
            deferred(
                "Model requires AuthorizationBehavior, CertificateAuthority. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an ACME endpoint, which is a managed ACME server with a unique endpoint URL. After creation, ACME clients can use the endpoint URL to automate certificate issuance using the ACME protocol.",
            ),
        ),
        (
            "CreateAcmeExternalAccountBinding",
            deferred(
                "Model requires AcmeEndpointArn, RoleArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an external account binding (EAB) for an ACME endpoint. An EAB provides credentials that authorize an ACME client to register an account with the endpoint. Each EAB is associated with an IAM role that controls what…",
            ),
        ),
        (
            "DeleteAcmeDomainValidation",
            deferred(
                "Model requires AcmeDomainValidationArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a domain validation. After deletion, the ACME endpoint can no longer issue certificates for the associated domain.",
            ),
        ),
        (
            "DeleteAcmeEndpoint",
            deferred(
                "Model requires AcmeEndpointArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an ACME endpoint. After deletion, the endpoint URL is no longer accessible and ACME clients cannot issue certificates through it. Any existing external account bindings and domain validations associated with the endpoint…",
            ),
        ),
        (
            "DeleteAcmeExternalAccountBinding",
            deferred(
                "Model requires AcmeExternalAccountBindingArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an external account binding. Previously fetched credentials for this binding will no longer be usable for account registration. A deleted binding cannot be recovered.",
            ),
        ),
        (
            "DeleteCertificate",
            deferred(
                "Model requires CertificateArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a certificate and its associated private key. If this action succeeds, the certificate is not available for use by Amazon Web Services services integrated with ACM. Deleting a certificate is eventually consistent. The may…",
            ),
        ),
        (
            "DescribeAcmeAccount",
            deferred(
                "Model requires AcmeEndpointArn, AccountUrl. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns detailed metadata about the specified ACME account, including its status, public key thumbprint, and associated external account binding.",
            ),
        ),
        (
            "DescribeAcmeDomainValidation",
            deferred(
                "Model requires AcmeDomainValidationArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns detailed metadata about the specified domain validation, including its status, domain scope, and DNS resource records required for validation.",
            ),
        ),
        (
            "DescribeAcmeEndpoint",
            deferred(
                "Model requires AcmeEndpointArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns detailed metadata about the specified ACME endpoint, including its status, URL, authorization behavior, and certificate authority configuration.",
            ),
        ),
        (
            "DescribeAcmeExternalAccountBinding",
            deferred(
                "Model requires AcmeExternalAccountBindingArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns detailed metadata about the specified external account binding, including the associated IAM role, expiration time, and usage history.",
            ),
        ),
        (
            "DescribeCertificate",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("CertificateArn")?,
                    Binding::From {
                        operation: name("ListCertificates")?,
                        path: path("CertificateArn")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("CertificateArn")?],
                ..read(
                    "Certificate",
                    Identity::Arn(path("CertificateArn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "ExportCertificate",
            deferred(
                "Model requires CertificateArn, Passphrase. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Exports a private certificate issued by a private certificate authority (CA) or a public certificate for use anywhere. The exported file contains the certificate, the certificate chain, and the encrypted private key associated…",
            ),
        ),
        (
            "GetAccountConfiguration",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is ExpiryEvents; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns the account configuration options associated with an Amazon Web Services account.",
            ),
        ),
        (
            "GetAcmeExternalAccountBindingCredentials",
            denied(DenyReason::Secret),
        ),
        (
            "GetCertificate",
            deferred(
                "Model requires CertificateArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a certificate and its certificate chain. The certificate may be either a public or private certificate issued using the ACM RequestCertificate action, or a certificate imported into ACM using the ImportCertificate…",
            ),
        ),
        (
            "ImportCertificate",
            deferred(
                "Model requires Certificate, PrivateKey. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Imports a certificate into Certificate Manager (ACM) to use with services that are integrated with ACM. Note that integrated services allow only certificate types and keys they support to be associated with their resources.…",
            ),
        ),
        (
            "ListAcmeAccounts",
            deferred(
                "Model requires AcmeEndpointArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a list of ACME accounts registered with the specified ACME endpoint. ACME accounts are created when clients use external account binding credentials to register.",
            ),
        ),
        (
            "ListAcmeDomainValidations",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("AcmeEndpointArn")?,
                    Binding::From {
                        operation: name("ListAcmeEndpoints")?,
                        path: path("AcmeEndpointArn")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("AcmeDomainValidationArn")?],
                ..read(
                    "AcmeDomainValidations",
                    Identity::Arn(path("AcmeDomainValidationArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListAcmeEndpoints",
            OperationOverlay {
                queryable: vec![path("AcmeEndpointArn")?],
                ..read(
                    "AcmeEndpoints",
                    Identity::Arn(path("AcmeEndpointArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "ListAcmeExternalAccountBindings",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("AcmeEndpointArn")?,
                    Binding::From {
                        operation: name("ListAcmeEndpoints")?,
                        path: path("AcmeEndpointArn")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("AcmeExternalAccountBindingArn")?],
                ..read(
                    "ExternalAccountBindings",
                    Identity::Arn(path("AcmeExternalAccountBindingArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListCertificateDomainValidations",
            deferred(
                "Model requires CertificateArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns per-domain validation summaries for an ACM certificate. Each summary includes the domain name, the active validation configuration, and the requested validation configuration when a validation method migration is in…",
            ),
        ),
        (
            "ListCertificates",
            OperationOverlay {
                fixed: [
                    (
                        path("Includes.keyTypes")?,
                        FixedValue::Strings(vec![
                            "RSA_1024".to_owned(),
                            "RSA_2048".to_owned(),
                            "RSA_3072".to_owned(),
                            "RSA_4096".to_owned(),
                            "EC_prime256v1".to_owned(),
                            "EC_secp384r1".to_owned(),
                            "EC_secp521r1".to_owned(),
                        ]),
                    ),
                    (
                        path("CertificateKeyPairOrigins")?,
                        FixedValue::Strings(vec![
                            "AWS_MANAGED".to_owned(),
                            "CUSTOMER_PROVIDED".to_owned(),
                            "ACME".to_owned(),
                        ]),
                    ),
                ]
                .into_iter()
                .collect(),
                queryable: vec![path("CertificateArn")?],
                ..read(
                    "CertificateSummaryList",
                    Identity::Arn(path("CertificateArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListTagsForCertificate",
            TagEndpoint {
                parents: vec![TagEndpointParent {
                    operation: name("ListCertificates")?,
                    request: TagEndpointRequest::Member {
                        input: path("CertificateArn")?,
                        parent: path("CertificateArn")?,
                    },
                }],
                selector: TagEndpointSelector::List {
                    path: path("Tags")?,
                    key: path("Key")?,
                    value: path("Value")?,
                },
            }
            .operation(Scope::Region, Paginator::None)?,
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires ResourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the tags associated with an ACM resource. Use this action for all ACM resource types except the certificate resource type. For certificate resources, use ListTagsForCertificate instead. To add one or more tags, use the…",
            ),
        ),
        (
            "PutAccountConfiguration",
            deferred(
                "Model requires IdempotencyToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds or modifies account-level configurations in ACM. The supported configuration option is DaysBeforeExpiry . This option specifies the number of days prior to certificate expiration when ACM starts generating EventBridge…",
            ),
        ),
        (
            "RemoveTagsFromCertificate",
            deferred(
                "Model requires CertificateArn, Tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Remove one or more tags from an ACM certificate. A tag consists of a key-value pair. If you do not specify the value portion of the tag when calling this function, the tag will be removed regardless of value. If you specify a…",
            ),
        ),
        (
            "RenewCertificate",
            deferred(
                "Model requires CertificateArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Renews an eligible ACM certificate . In order to renew your Amazon Web Services Private CA certificates with ACM, you must first grant the ACM service principal permission to do so . For more information, see Testing Managed…",
            ),
        ),
        (
            "RequestCertificate",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Requests an ACM certificate for use with other Amazon Web Services services. To request an ACM certificate, you must specify a fully qualified domain name (FQDN) in the DomainName parameter. You can also specify additional FQDNs…",
            ),
        ),
        (
            "ResendValidationEmail",
            deferred(
                "Model requires CertificateArn, Domain, ValidationDomain. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Resends the email that requests domain ownership validation. The domain owner or an authorized representative must approve the ACM certificate before it can be issued. The certificate can be approved by clicking a link in the…",
            ),
        ),
        (
            "RevokeAcmeAccount",
            deferred(
                "Model requires AcmeEndpointArn, AccountUrl. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Revokes an ACME account, preventing it from requesting or revoking certificates. This operation is irreversible.",
            ),
        ),
        (
            "RevokeAcmeExternalAccountBinding",
            deferred(
                "Model requires AcmeExternalAccountBindingArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Revokes an external account binding, preventing new ACME accounts from being registered using this binding. Existing ACME accounts that were previously registered using the binding are not affected and must be revoked separately.",
            ),
        ),
        (
            "RevokeCertificate",
            deferred(
                "Model requires CertificateArn, RevocationReason. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Revokes a public ACM certificate. You can only revoke certificates that have been previously exported. Once a certificate is revoked, you cannot reuse the certificate. Revoking a certificate is permanent.",
            ),
        ),
        (
            "SearchCertificates",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is Results; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Retrieves a list of certificates matching search criteria. You can filter certificates by X.509 attributes and ACM specific properties like certificate status, type and renewal eligibility. This operation provides more flexible…",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires ResourceArn, Tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds one or more tags to an ACM resource. Tags are labels that you can use to identify and organize your Amazon Web Services resources. Each tag consists of a key and an optional value . Use this action for all ACM resource types…",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires ResourceArn, TagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes one or more tags from an ACM resource. Use this action for all ACM resource types except the certificate resource type. For certificate resources, use RemoveTagsFromCertificate instead. To add one or more tags, use the…",
            ),
        ),
        (
            "UpdateAcmeDomainValidation",
            deferred(
                "Model requires AcmeDomainValidationArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the prevalidation configuration of an existing domain validation.",
            ),
        ),
        (
            "UpdateAcmeEndpoint",
            deferred(
                "Model requires AcmeEndpointArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the configuration of an existing ACME endpoint. You can change the authorization behavior, contact requirement, or certificate authority settings.",
            ),
        ),
        (
            "UpdateCertificateOptions",
            deferred(
                "Model requires CertificateArn, Options. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates certificate options. You can use this operation to change the domain validation method or specify whether to export your certificate. For more information, see Migrate from email to DNS validation and Certificate Manager…",
            ),
        ),
    ])
}
