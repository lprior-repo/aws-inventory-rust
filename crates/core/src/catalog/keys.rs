//! Catalog-only keys; member identities use `record::FieldId`, never these numbers.
use crate::record::{FieldId, RecordError};
/// Catalog map keys (a namespace separate from member IDs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum CatalogKey {
    /// Root format version.
    Format,
    /// Pinned upstream Git commit.
    ModelsCommit,
    /// Service records, each stored as bytes.
    Services,
    /// Service directory name, or unqualified shape/operation name.
    Name,
    /// Protocol trait IDs.
    Protocols,
    /// Operation maps.
    Operations,
    /// Shape maps in full-ID byte order.
    Shapes,
    /// Resource names.
    Resources,
    /// Shape kind code.
    Kind,
    /// Structure/union member records with wire metadata.
    Members,
    /// List element target.
    Element,
    /// Map key target.
    MapKey,
    /// Map value target.
    MapValue,
    /// ARN template, or null.
    Arn,
    /// Operation flags: readonly=1, paginated=2, List/Describe=4, input-free=8, read-named=16.
    Flags,
    /// Operation input target.
    Input,
    /// Operation output target.
    Output,
    /// Full Smithy service shape ID (protocol targets and error namespace).
    ServiceShape,
    /// SDK ID (aws.sdkId trait).
    SdkId,
    /// Endpoint prefix (aws.api#endpointPrefix trait).
    EndpointPrefix,
    /// ARN namespace (aws.api#arnNamespace trait).
    ArnNamespace,
    /// SigV4 signing name (aws.auth#sigv4Name trait).
    Sigv4Name,
    /// API version (aws.api#version trait).
    ApiVersion,
    /// JSON 1.0 payload trait (aws.protocols#json10Payload).
    Json10Payload,
    /// JSON 1.1 payload trait (aws.protocols#json11Payload).
    Json11Payload,
    /// CBOR payload trait (aws.protocols#cborPayload).
    CborPayload,
    /// XML namespace (aws.xmlNamespace trait).
    XmlNamespace,
    /// Paginated defaults (aws.protocols#paginatedDefaults).
    PaginatedDefaults,
    /// HTTP method (http trait).
    HttpMethod,
    /// HTTP URI (http trait).
    HttpUri,
    /// HTTP success status code (httpSuccess trait).
    HttpSuccess,
    /// Endpoint host prefix (aws.hostPrefix trait).
    HostPrefix,
    /// Paginated input token member name.
    PaginatedInput,
    /// Paginated output token member name.
    PaginatedOutput,
    /// Paginated result key member name.
    PaginatedResult,
    /// Paginated page size limit trait.
    PaginatedLimit,
    /// Paginated page size member name.
    PaginatedPageSize,
    /// Error shape references (aws.api#error trait targets).
    Errors,
    /// HTTP checksum required (aws.httpChecksumRequired trait).
    HttpChecksum,
    /// JSON wire name (jsonName trait).
    JsonName,
    /// XML wire name (xmlName trait).
    MemberXmlName,
    /// EC2 Query wire name (ec2QueryName trait).
    Ec2QueryName,
    /// HTTP label binding (httpLabel trait).
    HttpLabel,
    /// HTTP query binding (httpQuery trait).
    HttpQuery,
    /// HTTP query params binding (httpQueryParams trait).
    HttpQueryParams,
    /// HTTP header binding (httpHeader trait).
    HttpHeader,
    /// HTTP prefix headers binding (httpPrefixHeaders trait).
    HttpPrefixHeaders,
    /// HTTP payload binding (httpPayload trait).
    HttpPayload,
    /// HTTP response code binding (httpResponseCode trait).
    HttpResponseCode,
    /// Host label binding (hostLabel trait).
    MemberHostLabel,
    /// XML flattened trait.
    XmlFlattened,
    /// XML attribute trait.
    XmlAttribute,
    /// XML namespace trait on member.
    MemberXmlNamespace,
    /// Timestamp format trait.
    TimestampFormat,
    /// Idempotency token trait.
    IdempotencyToken,
    /// Media type trait (smithy.api#mediaType).
    MediaType,
    /// Sensitive trait (smithy.api#sensitive).
    Sensitive,
    /// Shape-level XML name trait.
    ShapeXmlName,
    /// Shape-level XML namespace trait.
    ShapeXmlNamespace,
    /// Shape-level timestamp format trait.
    ShapeTimestampFormat,
    /// List member wire name (aws.jsonName for list members).
    ListMemberName,
    /// Map key wire name (aws.jsonName for map keys).
    MapKeyName,
    /// Map value wire name (aws.jsonName for map values).
    MapValueName,
    /// Sparse list trait.
    Sparse,
    /// Streaming trait.
    Streaming,
    /// HTTP error status code (httpError trait).
    HttpError,
    /// AWS query error code (awsQueryError trait).
    AwsQueryError,
    ErrorRetryable,
    MemberId,
    MemberTarget,
    MemberRequired,
    FieldNames,
    FieldNameOffsets,
    XmlPrefix,
    XmlUri,
    AwsQueryStatus,
    ErrorThrottling,
    MemberXmlUri,
    MemberXmlPrefix,
    ShapeXmlPrefix,
    ServiceVersion,
    NoErrorWrapping,
    AwsQueryCompatible,
    CommonErrors,
    PaginatorMaxPageSize,
    ElementWire,
    MapKeyWire,
    MapValueWire,
    ErrorKind,
    RequestCompression,
}

impl CatalogKey {
    /// Convert a catalog key to the record writer's sole field identity.
    /// # Errors
    /// Only fails if the fixed nonzero vocabulary is changed incorrectly.
    pub fn field(self) -> Result<FieldId, RecordError> {
        FieldId::new(self.number())
    }

    fn number(self) -> u32 {
        match self {
            Self::Format => 1,
            Self::ModelsCommit => 2,
            Self::Services => 3,
            Self::Name => 4,
            Self::Protocols => 5,
            Self::Operations => 6,
            Self::Shapes => 7,
            Self::Resources => 8,
            Self::Kind => 9,
            Self::Members => 10,
            Self::Element => 11,
            Self::MapKey => 12,
            Self::MapValue => 13,
            Self::Arn => 14,
            Self::Flags => 15,
            Self::Input => 16,
            Self::Output => 17,
            Self::ServiceShape => 18,
            Self::SdkId => 19,
            Self::EndpointPrefix => 20,
            Self::ArnNamespace => 21,
            Self::Sigv4Name => 22,
            Self::ApiVersion => 23,
            Self::Json10Payload => 24,
            Self::Json11Payload => 25,
            Self::CborPayload => 26,
            Self::XmlNamespace => 27,
            Self::PaginatedDefaults => 28,
            Self::HttpMethod => 29,
            Self::HttpUri => 30,
            Self::HttpSuccess => 31,
            Self::HostPrefix => 32,
            Self::PaginatedInput => 33,
            Self::PaginatedOutput => 34,
            Self::PaginatedResult => 35,
            Self::PaginatedLimit => 36,
            Self::PaginatedPageSize => 37,
            Self::Errors => 38,
            Self::HttpChecksum => 39,
            Self::JsonName => 40,
            Self::MemberXmlName => 41,
            Self::Ec2QueryName => 42,
            Self::HttpLabel => 43,
            Self::HttpQuery => 44,
            Self::HttpQueryParams => 45,
            Self::HttpHeader => 46,
            Self::HttpPrefixHeaders => 47,
            Self::HttpPayload => 48,
            Self::HttpResponseCode => 49,
            Self::MemberHostLabel => 50,
            Self::XmlFlattened => 51,
            Self::XmlAttribute => 52,
            Self::MemberXmlNamespace => 53,
            Self::TimestampFormat => 54,
            Self::IdempotencyToken => 55,
            Self::MediaType => 56,
            Self::Sensitive => 57,
            Self::ShapeXmlName => 58,
            Self::ShapeXmlNamespace => 59,
            Self::ShapeTimestampFormat => 60,
            Self::ListMemberName => 61,
            Self::MapKeyName => 62,
            Self::MapValueName => 63,
            Self::Sparse => 64,
            Self::Streaming => 65,
            Self::HttpError => 66,
            Self::AwsQueryError => 67,
            Self::ErrorRetryable => 68,
            Self::MemberId => 69,
            Self::MemberTarget => 70,
            Self::MemberRequired => 71,
            Self::FieldNames => 72,
            Self::FieldNameOffsets => 73,
            Self::XmlPrefix => 74,
            Self::XmlUri => 75,
            Self::AwsQueryStatus => 76,
            Self::ErrorThrottling => 77,
            Self::MemberXmlUri => 78,
            Self::MemberXmlPrefix => 79,
            Self::ShapeXmlPrefix => 80,
            Self::ServiceVersion => 81,
            Self::NoErrorWrapping => 82,
            Self::AwsQueryCompatible => 83,
            Self::CommonErrors => 84,
            Self::PaginatorMaxPageSize => 85,
            Self::ElementWire => 86,
            Self::MapKeyWire => 87,
            Self::MapValueWire => 88,
            Self::ErrorKind => 89,
            Self::RequestCompression => 90,
        }
    }
}
/// Shape kind codes. Position is the wire code, shared by compiler and reader.
/// Scalars occupy codes 0..=13; containers and Smithy metadata follow them.
pub const SHAPE_KINDS: [&str; 23] = [
    "blob",
    "boolean",
    "string",
    "byte",
    "short",
    "integer",
    "long",
    "float",
    "double",
    "bigInteger",
    "bigDecimal",
    "timestamp",
    "document",
    "unit",
    "structure",
    "union",
    "list",
    "map",
    "enum",
    "intEnum",
    "service",
    "operation",
    "resource",
];

/// Resolve a Smithy kind to its stable wire code.
pub fn kind_code(name: &str) -> Option<usize> {
    SHAPE_KINDS.iter().position(|kind| *kind == name)
}
