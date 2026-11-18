// Pinned operation purposes: .local/aws-models/models/shield/service/2016-06-02/shield-2016-06-02.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Authorizes the Shield Response Team (SRT) to access the specified Amazon S3 bucket containing log data such as Application Load Balancer access logs, CloudFront logs, or logs from third party sources. You can associate up to 10 Amazon S3 buckets with your subscription. To use the services of the SRT and make an AssociateDRTLogBucket request, you must be subscribed to the Business Support plan or t
    (
        "AssociateDRTLogBucket",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Authorizes the Shield Response Team (SRT) using the specified role, to access your Amazon Web Services account to assist with DDoS attack mitigation during potential attacks. This enables the SRT to inspect your WAF configuration and create or update WAF rules and web ACLs. You can associate only one RoleArn with your subscription. If you submit an AssociateDRTRole request for an account that alre
    ("AssociateDRTRole", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Adds health-based detection to the Shield Advanced protection for a resource. Shield Advanced health-based detection uses the health of your Amazon Web Services resource to improve responsiveness and accuracy in attack detection and response. You define the health check in Route 53 and then associate it with your Shield Advanced protection. For more information, see Shield Advanced Health-Based De
    (
        "AssociateHealthCheck",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Initializes proactive engagement and sets the list of contacts for the Shield Response Team (SRT) to use. You must provide at least one phone number in the emergency contact list. After you have initialized proactive engagement using this call, to disable or enable proactive engagement, use the calls DisableProactiveEngagement and EnableProactiveEngagement . This call defines the list of email add
    (
        "AssociateProactiveEngagementDetails",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Enables Shield Advanced for a specific Amazon Web Services resource. The resource can be an Amazon CloudFront distribution, Amazon Route 53 hosted zone, Global Accelerator standard accelerator, Elastic IP Address, Application Load Balancer, or a Classic Load Balancer. You can protect Amazon EC2 instances and Network Load Balancers by association with protected Amazon EC2 Elastic IP addresses. You
    ("CreateProtection", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a grouping of protected resources so they can be handled as a collective. This resource grouping improves the accuracy of detection and reduces false positives.
    (
        "CreateProtectionGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Activates Shield Advanced for an account. For accounts that are members of an Organizations organization, Shield Advanced subscriptions are billed against the organization's payer account, regardless of whether the payer account itself is subscribed. When you initially create a subscription, your subscription is set to be automatically renewed at the end of the existing subscription period. You ca
    ("CreateSubscription", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an Shield Advanced Protection .
    ("DeleteProtection", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes the specified protection group.
    (
        "DeleteProtectionGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Removes Shield Advanced from an account. Shield Advanced requires a 1-year subscription commitment. You cannot delete a subscription prior to the completion of that commitment.
    ("DeleteSubscription", Decision::Denied(DenyReason::Mutating)),
    (
        "DescribeAttack",
        Decision::Deferred(
            "Required selectors AttackId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Attack. Pinned purpose: Describes the details of a DDoS attack.",
        ),
    ),
    (
        "DescribeAttackStatistics",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members TimeRange, DataItems. Pinned request/ownership contract: Provides information about the number and type of attacks Shield has detected in the last year for all resources that belong to your account, regardless of whether you've defined Shield protections for them. This operation is available to Shield customers as well as to Shield Advanced customers. The operation returns data for the time range of midnight UTC, one year ago, to midnight UTC, today. For example, if the current time is 2020-10-26 15:39",
        ),
    ),
    (
        "DescribeDRTAccess",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members RoleArn, LogBucketList. Pinned request/ownership contract: Returns the current role and list of Amazon S3 log buckets used by the Shield Response Team (SRT) to access your Amazon Web Services account while assisting with attack mitigation.",
        ),
    ),
    (
        "DescribeEmergencyContactSettings",
        Decision::Deferred(
            "Contains human emergency contact email/phone records, not resource configuration identity.",
        ),
    ),
    (
        "DescribeProtection",
        Decision::Deferred(
            "Requires either ProtectionId or ResourceArn although both are optional in the pinned structure. ListProtections supplies the complete protection root.",
        ),
    ),
    (
        "DescribeProtectionGroup",
        Decision::Deferred(
            "Required selectors ProtectionGroupId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ProtectionGroup. Pinned purpose: Returns the specification for the specified protection group.",
        ),
    ),
    (
        "DescribeSubscription",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members Subscription. Pinned request/ownership contract: Provides details about the Shield Advanced subscription for an account.",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Disable the Shield Advanced automatic application layer DDoS mitigation feature for the protected resource. This stops Shield Advanced from creating, verifying, and applying WAF rules for attacks that it detects for the resource.
    (
        "DisableApplicationLayerAutomaticResponse",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Removes authorization from the Shield Response Team (SRT) to notify contacts about escalations to the SRT and to initiate proactive customer support.
    (
        "DisableProactiveEngagement",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Removes the Shield Response Team's (SRT) access to the specified Amazon S3 bucket containing the logs that you shared previously.
    (
        "DisassociateDRTLogBucket",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Removes the Shield Response Team's (SRT) access to your Amazon Web Services account.
    (
        "DisassociateDRTRole",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Removes health-based detection from the Shield Advanced protection for a resource. Shield Advanced health-based detection uses the health of your Amazon Web Services resource to improve responsiveness and accuracy in attack detection and response. You define the health check in Route 53 and then associate or disassociate it with your Shield Advanced protection. For more information, see Shield Adv
    (
        "DisassociateHealthCheck",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Enable the Shield Advanced automatic application layer DDoS mitigation for the protected resource. This feature is available for Amazon CloudFront distributions and Application Load Balancers only. This causes Shield Advanced to create, verify, and apply WAF rules for DDoS attacks that it detects for the resource. Shield Advanced applies the rules in a Shield rule group inside the web ACL that you
    (
        "EnableApplicationLayerAutomaticResponse",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Authorizes the Shield Response Team (SRT) to use email and phone to notify contacts about escalations to the SRT and to initiate proactive customer support.
    (
        "EnableProactiveEngagement",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetSubscriptionState",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members SubscriptionState. Pinned request/ownership contract: Returns the SubscriptionState , either Active or Inactive .",
        ),
    ),
    (
        "ListResourcesInProtectionGroup",
        Decision::Deferred(
            "Required selectors ProtectionGroupId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ResourceArns, NextToken. Pinned purpose: Retrieves the resources that are included in the protection group.",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: ResourceARN. Pinned purpose: Gets information about Amazon Web Services tags for a specified Amazon Resource Name (ARN) in Shield.",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Adds or updates tags for a resource in Shield.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes tags from a resource in Shield.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing Shield Advanced automatic application layer DDoS mitigation configuration for the specified resource.
    (
        "UpdateApplicationLayerAutomaticResponse",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the details of the list of email addresses and phone numbers that the Shield Response Team (SRT) can use to contact you if you have proactive engagement enabled, for escalations to the SRT and to initiate proactive customer support.
    (
        "UpdateEmergencyContactSettings",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing protection group. A protection group is a grouping of protected resources so they can be handled as a collective. This resource grouping improves the accuracy of detection and reduces false positives.
    (
        "UpdateProtectionGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the details of an existing subscription. Only enter values for parameters you want to change. Empty parameters are not updated. For accounts that are members of an Organizations organization, Shield Advanced subscriptions are billed against the organization's payer account, regardless of whether the payer account itself is subscribed.
    ("UpdateSubscription", Decision::Denied(DenyReason::Mutating)),
];
