use crate::record::{FieldId, RecordError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum CatalogKey {
    Format,

    ModelsCommit,

    Services,

    Name,

    Protocols,

    Operations,

    Shapes,

    Resources,

    Kind,

    Members,

    Element,

    MapKey,

    MapValue,

    Arn,

    Flags,

    Input,

    Output,

    ServiceShape,

    SdkId,

    EndpointPrefix,

    ArnNamespace,

    Sigv4Name,

    ApiVersion,

    Json10Payload,

    Json11Payload,

    CborPayload,

    XmlNamespace,

    PaginatedDefaults,

    HttpMethod,

    HttpUri,

    HttpSuccess,

    HostPrefix,

    PaginatedInput,

    PaginatedOutput,

    PaginatedResult,

    PaginatedLimit,

    PaginatedPageSize,

    Errors,

    HttpChecksum,

    JsonName,

    MemberXmlName,

    Ec2QueryName,

    HttpLabel,

    HttpQuery,

    HttpQueryParams,

    HttpHeader,

    HttpPrefixHeaders,

    HttpPayload,

    HttpResponseCode,

    MemberHostLabel,

    XmlFlattened,

    XmlAttribute,

    MemberXmlNamespace,

    TimestampFormat,

    IdempotencyToken,

    MediaType,

    Sensitive,

    ShapeXmlName,

    ShapeXmlNamespace,

    ShapeTimestampFormat,

    ListMemberName,

    MapKeyName,

    MapValueName,

    Sparse,

    Streaming,

    HttpError,

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
    DefaultValue,
    ClientOptional,
    S3UnwrappedXmlOutput,
}

impl CatalogKey {
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
            Self::DefaultValue => 91,
            Self::ClientOptional => 92,
            Self::S3UnwrappedXmlOutput => 93,
        }
    }
}

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

pub fn kind_code(name: &str) -> Option<usize> {
    SHAPE_KINDS.iter().position(|kind| *kind == name)
}
