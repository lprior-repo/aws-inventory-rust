use super::{ACCOUNT, REGION, TestResult, runtime::NativeRuntime};
use aws_wire::{
    sign::{
        Body, PathMode, PayloadHeader, Region as SigningRegion, Request as SignRequest, Scope,
        SigningName,
    },
    transport::{Client, Config},
};
use bytes::Bytes;
use http::{Method, Request, header::CONTENT_TYPE};
use inventory_core::{
    inventory::{AccountId, Edge, EdgeSource, Kind, Region, Relation, ResourceId, ResourceKey},
    record::Value,
};
use quick_xml::{
    Reader,
    events::{BytesStart, Event},
};
use std::{collections::BTreeMap, fmt::Write, time::SystemTime};

struct Describe {
    action: &'static str,
    kind: &'static str,
    container: &'static str,
    identity: &'static str,
    fields: &'static [(&'static str, &'static str)],
    vpc_member: bool,
}

const EC2: &[Describe] = &[
    Describe {
        action: "DescribeVpcs",
        kind: "AWS::EC2::VPC",
        container: "vpcSet",
        identity: "vpcId",
        fields: &[("cidrBlock", "CidrBlock")],
        vpc_member: false,
    },
    Describe {
        action: "DescribeSubnets",
        kind: "AWS::EC2::Subnet",
        container: "subnetSet",
        identity: "subnetId",
        fields: &[("cidrBlock", "CidrBlock")],
        vpc_member: true,
    },
    Describe {
        action: "DescribeRouteTables",
        kind: "AWS::EC2::RouteTable",
        container: "routeTableSet",
        identity: "routeTableId",
        fields: &[("routeTableId", "RouteTableId")],
        vpc_member: true,
    },
    Describe {
        action: "DescribeSecurityGroups",
        kind: "AWS::EC2::SecurityGroup",
        container: "securityGroupInfo",
        identity: "groupId",
        fields: &[
            ("groupName", "GroupName"),
            ("groupDescription", "Description"),
        ],
        vpc_member: true,
    },
];

const RDS: Describe = Describe {
    action: "DescribeDBParameterGroups",
    kind: "AWS::RDS::DBParameterGroup",
    container: "DBParameterGroups",
    identity: "DBParameterGroupArn",
    fields: &[
        ("DBParameterGroupName", "DBParameterGroupName"),
        ("DBParameterGroupFamily", "DBParameterGroupFamily"),
        ("Description", "Description"),
    ],
    vpc_member: false,
};

pub(super) type Listing = BTreeMap<ResourceKey, ListedResource>;

pub(super) struct ListedResource {
    pub fields: BTreeMap<String, Value>,
    vpc: Option<ResourceId>,
}

pub(super) fn vpc_edges(listed: &Listing) -> TestResult<impl Iterator<Item = Edge> + '_> {
    let vpc_kind = Kind::parse("AWS::EC2::VPC")?;
    let relation = Relation::parse("IN_VPC")?;
    Ok(listed.iter().filter_map(move |(from, resource)| {
        resource.vpc.as_ref().map(|id| Edge {
            from: from.clone(),
            to: ResourceKey {
                account: from.account.clone(),
                region: from.region.clone(),
                kind: vpc_kind.clone(),
                id: id.clone(),
            },
            relation: relation.clone(),
            source: EdgeSource::ReviewedRule,
        })
    }))
}

pub(super) fn supports(kind: &Kind) -> bool {
    kind.as_str() == RDS.kind || EC2.iter().any(|spec| spec.kind == kind.as_str())
}

pub(super) async fn read(native: &NativeRuntime) -> TestResult<Listing> {
    let client = Client::new(Config {
        extra_ca_bundle: Some(native.emulator().ca_bundle().to_path_buf()),
        ..Config::default()
    })?;
    let mut listed = Listing::new();
    for spec in EC2 {
        enumerate(native, &client, spec, Protocol::Ec2, &mut listed).await?;
    }
    enumerate(native, &client, &RDS, Protocol::Rds, &mut listed).await?;
    Ok(listed)
}

pub(super) async fn add_topic_policies(
    native: &NativeRuntime,
    records: &mut super::safe_fields::Expectations,
) -> TestResult {
    let client = Client::new(Config {
        extra_ca_bundle: Some(native.emulator().ca_bundle().to_path_buf()),
        ..Config::default()
    })?;
    let action = "GetDataProtectionPolicy";
    for (key, fields) in records {
        if key.kind.as_str() != "AWS::SNS::Topic" {
            continue;
        }
        let mut form = format!("Action={action}&Version=2010-03-31&ResourceArn=");
        append_form(&mut form, key.id.as_str())?;
        let body = send_query(native, &client, "sns", action, form).await?;
        let policy = topic_policy(&body)?;
        assert!(
            fields
                .insert("DataProtectionPolicy".into(), Value::Text(policy))
                .is_none(),
            "duplicate independent topic policy expectation"
        );
    }
    Ok(())
}

fn topic_policy(body: &[u8]) -> TestResult<String> {
    let mut reader = Reader::from_reader(body);
    reader.config_mut().expand_empty_elements = true;
    find_start(&mut reader, "GetDataProtectionPolicyResponse")?;
    find_start(&mut reader, "GetDataProtectionPolicyResult")?;
    let mut policy = None;
    while let Some(start) = next_child(&mut reader)? {
        if start.local_name().into_inner() == "DataProtectionPolicy" {
            if policy.replace(text(&mut reader, &start)?).is_some() {
                return Err("duplicate raw SNS DataProtectionPolicy".into());
            }
        } else {
            reader.read_to_end(start.name())?;
        }
    }
    policy.ok_or_else(|| "raw SNS response lacks DataProtectionPolicy".into())
}

#[derive(Clone, Copy)]
enum Protocol {
    Ec2,
    Rds,
}

impl Protocol {
    fn request_parts(self) -> (&'static str, &'static str, &'static str, &'static str) {
        match self {
            Self::Ec2 => ("ec2", "2016-11-15", "MaxResults=100", "NextToken"),
            Self::Rds => ("rds", "2014-10-31", "MaxRecords=20", "Marker"),
        }
    }

    fn response_parts(self) -> (&'static str, &'static str) {
        match self {
            Self::Ec2 => ("item", "nextToken"),
            Self::Rds => ("DBParameterGroup", "Marker"),
        }
    }
}

async fn enumerate(
    native: &NativeRuntime,
    client: &Client,
    spec: &Describe,
    protocol: Protocol,
    listed: &mut Listing,
) -> TestResult {
    let account = AccountId::parse(ACCOUNT)?;
    let region = Region::parse(REGION)?;
    let kind = Kind::parse(spec.kind)?;
    let (service, version, limit, parameter) = protocol.request_parts();
    let mut cursor = None;
    for _ in 0..100 {
        let mut form = format!("Action={}&Version={version}&{limit}", spec.action);
        if let Some(cursor) = cursor.as_deref() {
            write!(&mut form, "&{parameter}=")?;
            append_form(&mut form, cursor)?;
        }
        let body = send_query(native, client, service, spec.action, form).await?;
        let next = read_page(&body, spec, protocol, &account, &region, &kind, listed)?;
        if next.is_none() {
            return Ok(());
        }
        if next == cursor {
            return Err(format!("{} repeated its pagination token", spec.action).into());
        }
        cursor = next;
    }
    Err(format!("{} exceeded the listing page budget", spec.action).into())
}

async fn send_query(
    native: &NativeRuntime,
    client: &Client,
    service: &str,
    action: &str,
    form: String,
) -> TestResult<Bytes> {
    let endpoint = native.emulator().endpoint();
    let mut request = Request::builder()
        .method(Method::POST)
        .uri(endpoint)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Bytes::from(form))?;
    let scope = Scope {
        region: SigningRegion::new(REGION)?,
        name: SigningName::new(service)?,
    };
    let signed = aws_wire::sign::sign(
        &native.credentials,
        &scope,
        SystemTime::now(),
        SignRequest {
            method: request.method(),
            uri: endpoint,
            headers: request.headers(),
            body: Body::Bytes(request.body()),
            payload_header: PayloadHeader::Include,
        },
        PathMode::Standard,
    )?;
    request.headers_mut().extend(signed);
    let response = client.send(request).await?;
    if !response.status.is_success() {
        return Err(format!("{action} returned {}", response.status).into());
    }
    Ok(response.body)
}

fn append_form(output: &mut String, value: &str) -> TestResult {
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            output.push(char::from(byte));
        } else {
            write!(output, "%{byte:02X}")?;
        }
    }
    Ok(())
}

fn read_page(
    body: &[u8],
    spec: &Describe,
    protocol: Protocol,
    account: &AccountId,
    region: &Region,
    kind: &Kind,
    listed: &mut Listing,
) -> TestResult<Option<String>> {
    let mut reader = Reader::from_reader(body);
    reader.config_mut().expand_empty_elements = true;
    find_start(&mut reader, &format!("{}Response", spec.action))?;
    if matches!(protocol, Protocol::Rds) {
        find_start(&mut reader, "DescribeDBParameterGroupsResult")?;
    }
    let (entry, token) = protocol.response_parts();
    let mut present = false;
    let mut next = None;
    while let Some(start) = next_child(&mut reader)? {
        let name = start.local_name().into_inner();
        if name == spec.container && !present {
            present = true;
            while let Some(start) = next_child(&mut reader)? {
                if start.local_name().into_inner() != entry {
                    return Err(format!("{} returned an unexpected list entry", spec.action).into());
                }
                let (id, resource) = read_row(&mut reader, spec)?;
                let key = ResourceKey {
                    account: account.clone(),
                    region: region.clone(),
                    kind: kind.clone(),
                    id,
                };
                if listed.insert(key, resource).is_some() {
                    return Err(format!("{} listed a duplicate resource", spec.action).into());
                }
            }
        } else if name == token && next.is_none() {
            next = Some(text(&mut reader, &start)?);
        } else if name == spec.container || name == token {
            return Err(format!("{} returned a duplicate listing member", spec.action).into());
        } else {
            reader.read_to_end(start.name())?;
        }
    }
    if !present {
        return Err(format!("{} response lacks its resource list", spec.action).into());
    }
    Ok(next.filter(|token| !token.is_empty()))
}

fn read_row(
    reader: &mut Reader<&[u8]>,
    spec: &Describe,
) -> TestResult<(ResourceId, ListedResource)> {
    let mut id = None;
    let mut vpc = None;
    let mut fields = BTreeMap::new();
    while let Some(start) = next_child(reader)? {
        let name = start.local_name().into_inner();
        let identity = name == spec.identity;
        let field = spec.fields.iter().find(|(wire, _)| name == *wire);
        let vpc_member = spec.vpc_member && name == "vpcId";
        if !identity && field.is_none() && !vpc_member {
            reader.read_to_end(start.name())?;
            continue;
        }
        let value = text(reader, &start)?;
        if identity && id.replace(ResourceId::parse(&value)?).is_some() {
            return Err(format!("{} list entry repeated its identity", spec.action).into());
        }
        if vpc_member && vpc.replace(ResourceId::parse(&value)?).is_some() {
            return Err(format!("{} list entry repeated its VPC", spec.action).into());
        }
        if let Some((_, field)) = field
            && fields.insert((*field).into(), Value::Text(value)).is_some()
        {
            return Err(format!("{} list entry repeated {field}", spec.action).into());
        }
    }
    let id = id.ok_or_else(|| format!("{} list entry lacks its identity", spec.action))?;
    if fields.len() != spec.fields.len() {
        return Err(format!("{} list entry {id} lacks safe fields", spec.action).into());
    }
    Ok((id, ListedResource { fields, vpc }))
}

fn find_start(reader: &mut Reader<&[u8]>, name: &str) -> TestResult {
    while let Some(start) = next_child(reader)? {
        if start.local_name().into_inner() == name {
            return Ok(());
        }
        reader.read_to_end(start.name())?;
    }
    Err(format!("XML lacks {name}").into())
}

fn next_child<'a>(reader: &mut Reader<&'a [u8]>) -> TestResult<Option<BytesStart<'a>>> {
    loop {
        match reader.read_event()? {
            Event::Start(start) => return Ok(Some(start)),
            Event::End(_) => return Ok(None),
            Event::Eof => return Err("incomplete listing XML".into()),
            _ => {}
        }
    }
}

fn text(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> TestResult<String> {
    let text = reader.read_text(start.name())?;
    Ok(quick_xml::escape::unescape(&text.xml10_content())?.into_owned())
}
