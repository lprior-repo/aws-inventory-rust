 

use crate::S;

fn t(text: &str) -> S {
    S::Text(text.to_owned())
}

fn tags(count: usize) -> S {
    S::List(
        (0..count)
            .map(|i| {
                S::Struct(vec![
                    ("Key", t(&format!("team:cost-center-{i}"))),
                    ("Value", t(&format!("platform-engineering-{i}"))),
                ])
            })
            .collect(),
    )
}

fn group(i: usize) -> S {
    S::Struct(vec![
        ("GroupId", t(&format!("sg-0a1b2c3d4e5f6071{i}"))),
        ("GroupName", t(&format!("app-tier-{i}"))),
    ])
}

fn volume(i: usize) -> S {
    S::Struct(vec![
        (
            "DeviceName",
            t(&format!("/dev/sd{}", (b'a' + i as u8) as char)),
        ),
        (
            "Ebs",
            S::Struct(vec![
                ("AttachTime", S::Time(1_714_566_896, 0)),
                ("DeleteOnTermination", S::Bool(true)),
                ("Status", t("attached")),
                ("VolumeId", t(&format!("vol-0123456789abcde{i:02}"))),
            ]),
        ),
    ])
}

fn interface(i: usize) -> S {
    S::Struct(vec![
        (
            "Attachment",
            S::Struct(vec![
                ("AttachTime", S::Time(1_714_566_895, 0)),
                ("AttachmentId", t("eni-attach-0fedcba987654321")),
                ("DeleteOnTermination", S::Bool(true)),
                ("DeviceIndex", S::Int(i as i64)),
                ("Status", t("attached")),
                ("NetworkCardIndex", S::Int(0)),
            ]),
        ),
        ("Description", t("Primary network interface")),
        ("Groups", S::List(vec![group(0), group(1)])),
        ("Ipv6Addresses", S::List(vec![])),
        ("MacAddress", t("0a:1b:2c:3d:4e:5f")),
        ("NetworkInterfaceId", t(&format!("eni-0123456789abcdef{i}"))),
        ("OwnerId", t("123456789012")),
        (
            "PrivateDnsName",
            t("ip-10-0-12-34.eu-west-1.compute.internal"),
        ),
        ("PrivateIpAddress", t("10.0.12.34")),
        (
            "PrivateIpAddresses",
            S::List(vec![S::Struct(vec![
                ("Primary", S::Bool(true)),
                (
                    "PrivateDnsName",
                    t("ip-10-0-12-34.eu-west-1.compute.internal"),
                ),
                ("PrivateIpAddress", t("10.0.12.34")),
            ])]),
        ),
        ("SourceDestCheck", S::Bool(true)),
        ("Status", t("in-use")),
        ("SubnetId", t("subnet-0123456789abcdef0")),
        ("VpcId", t("vpc-0123456789abcdef0")),
        ("InterfaceType", t("interface")),
    ])
}

 
pub fn instance() -> S {
    S::Struct(vec![
        ("AmiLaunchIndex", S::Int(0)),
        ("ImageId", t("ami-0123456789abcdef0")),
        ("InstanceId", t("i-0123456789abcdef0")),
        ("InstanceType", t("m7g.2xlarge")),
        ("KeyName", t("platform-prod")),
        ("LaunchTime", S::Time(1_714_566_895, 0)),
        ("Monitoring", S::Struct(vec![("State", t("enabled"))])),
        (
            "Placement",
            S::Struct(vec![
                ("AvailabilityZone", t("eu-west-1b")),
                ("GroupName", t("")),
                ("Tenancy", t("default")),
            ]),
        ),
        (
            "PrivateDnsName",
            t("ip-10-0-12-34.eu-west-1.compute.internal"),
        ),
        ("PrivateIpAddress", t("10.0.12.34")),
        ("ProductCodes", S::List(vec![])),
        ("PublicDnsName", t("")),
        (
            "State",
            S::Struct(vec![("Code", S::Int(16)), ("Name", t("running"))]),
        ),
        ("StateTransitionReason", t("")),
        ("SubnetId", t("subnet-0123456789abcdef0")),
        ("VpcId", t("vpc-0123456789abcdef0")),
        ("Architecture", t("arm64")),
        ("BlockDeviceMappings", S::List((0..3).map(volume).collect())),
        ("ClientToken", t("5f2c8a1e-7b3d-4c9a-9e1f-0a2b4c6d8e0f")),
        ("EbsOptimized", S::Bool(true)),
        ("EnaSupport", S::Bool(true)),
        ("Hypervisor", t("xen")),
        (
            "IamInstanceProfile",
            S::Struct(vec![
                (
                    "Arn",
                    t("arn:aws:iam::123456789012:instance-profile/app-tier-node"),
                ),
                ("Id", t("AIPAJ2UCCR6DPCEXAMPLE")),
            ]),
        ),
        (
            "NetworkInterfaces",
            S::List((0..2).map(interface).collect()),
        ),
        ("RootDeviceName", t("/dev/sda")),
        ("RootDeviceType", t("ebs")),
        ("SecurityGroups", S::List(vec![group(0), group(1)])),
        ("SourceDestCheck", S::Bool(true)),
        ("Tags", tags(10)),
        ("VirtualizationType", t("hvm")),
        (
            "CpuOptions",
            S::Struct(vec![
                ("CoreCount", S::Int(8)),
                ("ThreadsPerCore", S::Int(1)),
            ]),
        ),
        (
            "CapacityReservationSpecification",
            S::Struct(vec![("CapacityReservationPreference", t("open"))]),
        ),
        (
            "HibernationOptions",
            S::Struct(vec![("Configured", S::Bool(false))]),
        ),
        (
            "MetadataOptions",
            S::Struct(vec![
                ("State", t("applied")),
                ("HttpTokens", t("required")),
                ("HttpPutResponseHopLimit", S::Int(2)),
                ("HttpEndpoint", t("enabled")),
                ("HttpProtocolIpv6", t("disabled")),
                ("InstanceMetadataTags", t("disabled")),
            ]),
        ),
        (
            "EnclaveOptions",
            S::Struct(vec![("Enabled", S::Bool(false))]),
        ),
        ("BootMode", t("uefi")),
        ("PlatformDetails", t("Linux/UNIX")),
        ("UsageOperation", t("RunInstances")),
        ("UsageOperationUpdateTime", S::Time(1_714_566_895, 0)),
        (
            "PrivateDnsNameOptions",
            S::Struct(vec![
                ("HostnameType", t("ip-name")),
                ("EnableResourceNameDnsARecord", S::Bool(false)),
                ("EnableResourceNameDnsAAAARecord", S::Bool(false)),
            ]),
        ),
        (
            "MaintenanceOptions",
            S::Struct(vec![("AutoRecovery", t("default"))]),
        ),
        ("CurrentInstanceBootMode", t("uefi")),
        (
            "Attributes",
            S::Map(vec![
                ("cpu-credits".to_owned(), t("standard")),
                ("spot-price".to_owned(), S::Float(0.1342)),
                ("user-data-sha".to_owned(), S::Bytes(vec![0xAB; 32])),
                ("retired".to_owned(), S::Null),
            ]),
        ),
        ("OwnerId", t("123456789012")),
    ])
}

 
pub fn small() -> S {
    S::Struct(vec![
        (
            "QueueArn",
            t("arn:aws:sqs:eu-west-1:123456789012:orders-events-prod"),
        ),
        (
            "QueueUrl",
            t("https://sqs.eu-west-1.amazonaws.com/123456789012/orders-events-prod"),
        ),
        ("Name", t("orders-events-prod")),
        ("CreatedTimestamp", S::Time(1_690_000_000, 0)),
        ("LastModifiedTimestamp", S::Time(1_714_566_895, 0)),
        ("VisibilityTimeout", S::Int(30)),
        ("MessageRetentionPeriod", S::Int(345_600)),
        ("SqsManagedSseEnabled", S::Bool(false)),
        (
            "KmsMasterKeyId",
            t("arn:aws:kms:eu-west-1:123456789012:key/1234abcd-12ab-34cd-56ef-1234567890ab"),
        ),
        (
            "Policy",
            t(
                "{\"Version\":\"2012-10-17\",\"Statement\":[{\"Effect\":\"Allow\",\"Principal\":{\"Service\":\"sns.amazonaws.com\"},\"Action\":\"sqs:SendMessage\",\"Resource\":\"*\"}]}",
            ),
        ),
    ])
}
