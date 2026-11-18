// Explicit dispositions from .local/aws-models/models/elastic-load-balancing/service/2012-06-01/elastic-load-balancing-2012-06-01.json; never a runtime read-name approval rule.
pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "AddTags",
        "Requires LoadBalancerNames, Tags; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Adds the specified tags to the specified load balancer.",
    ),
    (
        "ApplySecurityGroupsToLoadBalancer",
        "Requires LoadBalancerName, SecurityGroups; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Associates one or more security groups with your load balancer in a virtual private cloud (VPC).",
    ),
    (
        "AttachLoadBalancerToSubnets",
        "Requires LoadBalancerName, Subnets; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Adds one or more subnets to the set of configured subnets for the specified load balancer.",
    ),
    (
        "ConfigureHealthCheck",
        "Requires LoadBalancerName, HealthCheck; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Specifies the health check settings to use when evaluating the health state of your EC2 instances.",
    ),
    (
        "CreateAppCookieStickinessPolicy",
        "Requires LoadBalancerName, PolicyName, CookieName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Generates a stickiness policy with sticky session lifetimes that follow that of an application-generated cookie.",
    ),
    (
        "CreateLBCookieStickinessPolicy",
        "Requires LoadBalancerName, PolicyName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Generates a stickiness policy with sticky session lifetimes controlled by the lifetime of the browser (user-agent) or a specified expiration period.",
    ),
    (
        "CreateLoadBalancer",
        "Requires LoadBalancerName, Listeners; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a Classic Load Balancer.",
    ),
    (
        "CreateLoadBalancerListeners",
        "Requires LoadBalancerName, Listeners; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates one or more listeners for the specified load balancer.",
    ),
    (
        "CreateLoadBalancerPolicy",
        "Requires LoadBalancerName, PolicyName, PolicyTypeName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a policy with the specified attributes for the specified load balancer.",
    ),
    (
        "DeleteLoadBalancer",
        "Requires LoadBalancerName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified load balancer.",
    ),
    (
        "DeleteLoadBalancerListeners",
        "Requires LoadBalancerName, LoadBalancerPorts; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified listeners from the specified load balancer.",
    ),
    (
        "DeleteLoadBalancerPolicy",
        "Requires LoadBalancerName, PolicyName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified policy from the specified load balancer.",
    ),
    (
        "DeregisterInstancesFromLoadBalancer",
        "Requires LoadBalancerName, Instances; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deregisters the specified instances from the specified load balancer.",
    ),
    (
        "DescribeAccountLimits",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Describes the current Elastic Load Balancing resource limits for your AWS account.",
    ),
    (
        "DescribeInstanceHealth",
        "Requires LoadBalancerName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Describes the state of the specified instances with respect to the specified load balancer.",
    ),
    (
        "DescribeLoadBalancerAttributes",
        "Requires LoadBalancerName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Describes the attributes for the specified load balancer.",
    ),
    (
        "DescribeLoadBalancerPolicies",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Describes the specified policies.",
    ),
    (
        "DescribeLoadBalancerPolicyTypes",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Describes the specified load balancer policy types or all load balancer policy types.",
    ),
    (
        "DescribeTags",
        "Requires LoadBalancerNames; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Describes the tags associated with the specified load balancers.",
    ),
    (
        "DetachLoadBalancerFromSubnets",
        "Requires LoadBalancerName, Subnets; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes the specified subnets from the set of configured subnets for the load balancer.",
    ),
    (
        "DisableAvailabilityZonesForLoadBalancer",
        "Requires LoadBalancerName, AvailabilityZones; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes the specified Availability Zones from the set of Availability Zones for the specified load balancer in EC2-Classic or a default VPC.",
    ),
    (
        "EnableAvailabilityZonesForLoadBalancer",
        "Requires LoadBalancerName, AvailabilityZones; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Adds the specified Availability Zones to the set of Availability Zones for the specified load balancer in EC2-Classic or a default VPC.",
    ),
    (
        "ModifyLoadBalancerAttributes",
        "Requires LoadBalancerName, LoadBalancerAttributes; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies the attributes of the specified load balancer.",
    ),
    (
        "RegisterInstancesWithLoadBalancer",
        "Requires LoadBalancerName, Instances; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Adds the specified instances to the specified load balancer.",
    ),
    (
        "RemoveTags",
        "Requires LoadBalancerNames, Tags; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes one or more tags from the specified load balancer.",
    ),
    (
        "SetLoadBalancerListenerSSLCertificate",
        "Requires LoadBalancerName, LoadBalancerPort, SSLCertificateId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Sets the certificate that terminates the specified listener's SSL connections.",
    ),
    (
        "SetLoadBalancerPoliciesForBackendServer",
        "Requires LoadBalancerName, InstancePort, PolicyNames; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Replaces the set of policies associated with the specified port on which the EC2 instance is listening with a new set of policies.",
    ),
    (
        "SetLoadBalancerPoliciesOfListener",
        "Requires LoadBalancerName, LoadBalancerPort, PolicyNames; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Replaces the current set of policies for the specified load balancer port with the specified set of policies.",
    ),
];
