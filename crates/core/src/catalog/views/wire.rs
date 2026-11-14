pub(in crate::catalog) mod field_name;
use super::super::{CatalogError, CatalogKey};
use crate::record::{MapRef, ValueRef};
pub(in crate::catalog) use field_name::field_name;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimestampFormat {
    DateTime,
    HttpDate,
    EpochSeconds,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HttpBinding<'a> {
    Document,
    Label,
    Query(&'a str),
    QueryParams,
    Header(&'a str),
    PrefixHeaders(&'a str),
    Payload,
    ResponseCode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct XmlNamespace<'a> {
    pub uri: &'a str,
    pub prefix: Option<&'a str>,
}

#[derive(Clone, Copy, Debug)]
pub struct WireTraits<'a> {
    map: Option<MapRef<'a>>,
    role: WireRole,
}

#[derive(Clone, Copy, Debug)]
enum WireRole {
    Shape,
    Member,
    Service,
    Empty,
}

impl<'a> WireTraits<'a> {
    pub(super) const fn empty() -> Self {
        Self {
            map: None,
            role: WireRole::Empty,
        }
    }

    pub(super) const fn shape(map: MapRef<'a>) -> Self {
        Self {
            map: Some(map),
            role: WireRole::Shape,
        }
    }

    pub(super) const fn member(map: MapRef<'a>) -> Self {
        Self {
            map: Some(map),
            role: WireRole::Member,
        }
    }

    pub(super) const fn service(map: MapRef<'a>) -> Self {
        Self {
            map: Some(map),
            role: WireRole::Service,
        }
    }

    pub(super) fn nested(map: MapRef<'a>, key: CatalogKey) -> Result<Self, CatalogError> {
        match map.get_field(key.field()?)? {
            None => Ok(Self::empty()),
            Some(ValueRef::Map(map)) => Ok(Self::member(map)),
            _ => Err(CatalogError::Field(key)),
        }
    }

    fn text(self, shape: CatalogKey, member: CatalogKey) -> Result<Option<&'a str>, CatalogError> {
        let key = match self.role {
            WireRole::Shape => shape,
            WireRole::Member => member,
            WireRole::Service | WireRole::Empty => return Ok(None),
        };
        match self
            .map
            .ok_or(CatalogError::Reference)?
            .get_field(key.field()?)?
        {
            None => Ok(None),
            Some(ValueRef::Text(text)) => Ok(Some(text)),
            _ => Err(CatalogError::Field(key)),
        }
    }

    fn flag(self, key: CatalogKey) -> Result<bool, CatalogError> {
        match self.role {
            WireRole::Empty => Ok(false),
            WireRole::Shape | WireRole::Member | WireRole::Service => {
                match self
                    .map
                    .ok_or(CatalogError::Reference)?
                    .get_field(key.field()?)?
                {
                    None => Ok(false),
                    Some(ValueRef::Bool(value)) => Ok(value),
                    _ => Err(CatalogError::Field(key)),
                }
            }
        }
    }

    pub fn json_name(self) -> Result<Option<&'a str>, CatalogError> {
        if matches!(self.role, WireRole::Member) {
            self.text(CatalogKey::JsonName, CatalogKey::JsonName)
        } else {
            Ok(None)
        }
    }
    pub fn xml_name(self) -> Result<Option<&'a str>, CatalogError> {
        self.text(CatalogKey::ShapeXmlName, CatalogKey::MemberXmlName)
    }
    pub fn ec2_query_name(self) -> Result<Option<&'a str>, CatalogError> {
        if matches!(self.role, WireRole::Member) {
            self.text(CatalogKey::Ec2QueryName, CatalogKey::Ec2QueryName)
        } else {
            Ok(None)
        }
    }
    pub fn media_type(self) -> Result<Option<&'a str>, CatalogError> {
        self.text(CatalogKey::MediaType, CatalogKey::MediaType)
    }
    pub fn xml_namespace(self) -> Result<Option<XmlNamespace<'a>>, CatalogError> {
        match self.role {
            WireRole::Empty => Ok(None),
            WireRole::Service => self.service_namespace(),
            WireRole::Shape | WireRole::Member => self.shape_or_member_namespace(),
        }
    }

    fn service_namespace(self) -> Result<Option<XmlNamespace<'a>>, CatalogError> {
        let key = CatalogKey::XmlNamespace;
        match self
            .map
            .ok_or(CatalogError::Reference)?
            .get_field(key.field()?)?
        {
            None => Ok(None),
            Some(ValueRef::Map(map)) => {
                let uri = match map.get_field(CatalogKey::XmlUri.field()?)? {
                    Some(ValueRef::Text(uri)) => uri,
                    _ => return Err(CatalogError::Field(CatalogKey::XmlUri)),
                };
                let prefix = match map.get_field(CatalogKey::XmlPrefix.field()?)? {
                    None => None,
                    Some(ValueRef::Text(prefix)) => Some(prefix),
                    _ => return Err(CatalogError::Field(CatalogKey::XmlPrefix)),
                };
                Ok(Some(XmlNamespace { uri, prefix }))
            }
            _ => Err(CatalogError::Field(key)),
        }
    }

    fn shape_or_member_namespace(self) -> Result<Option<XmlNamespace<'a>>, CatalogError> {
        let (uri_key, prefix_key) = match self.role {
            WireRole::Shape => (CatalogKey::ShapeXmlNamespace, CatalogKey::ShapeXmlPrefix),
            WireRole::Member => (CatalogKey::MemberXmlUri, CatalogKey::MemberXmlPrefix),
            WireRole::Service | WireRole::Empty => return Ok(None),
        };
        let map = self.map.ok_or(CatalogError::Reference)?;
        let uri = match map.get_field(uri_key.field()?)? {
            None => return Ok(None),
            Some(ValueRef::Text(uri)) => uri,
            _ => return Err(CatalogError::Field(uri_key)),
        };
        let prefix = match map.get_field(prefix_key.field()?)? {
            None => None,
            Some(ValueRef::Text(prefix)) => Some(prefix),
            _ => return Err(CatalogError::Field(prefix_key)),
        };
        Ok(Some(XmlNamespace { uri, prefix }))
    }
    pub fn timestamp_format(self) -> Result<Option<TimestampFormat>, CatalogError> {
        let key = match self.role {
            WireRole::Shape => CatalogKey::ShapeTimestampFormat,
            WireRole::Member => CatalogKey::TimestampFormat,
            WireRole::Service | WireRole::Empty => return Ok(None),
        };
        match self
            .map
            .ok_or(CatalogError::Reference)?
            .get_field(key.field()?)?
        {
            None => Ok(None),
            Some(ValueRef::Text("date-time")) => Ok(Some(TimestampFormat::DateTime)),
            Some(ValueRef::Text("http-date")) => Ok(Some(TimestampFormat::HttpDate)),
            Some(ValueRef::Text("epoch-seconds")) => Ok(Some(TimestampFormat::EpochSeconds)),
            _ => Err(CatalogError::Field(key)),
        }
    }
    pub fn binding(self) -> Result<HttpBinding<'a>, CatalogError> {
        let Some(map) = self.map else {
            return Ok(HttpBinding::Document);
        };
        let found = [
            CatalogKey::HttpLabel,
            CatalogKey::HttpQuery,
            CatalogKey::HttpQueryParams,
            CatalogKey::HttpHeader,
            CatalogKey::HttpPrefixHeaders,
            CatalogKey::HttpPayload,
            CatalogKey::HttpResponseCode,
        ]
        .into_iter()
        .try_fold(None, |found, key| {
            let value = map.get_field(key.field()?)?;
            match (found, value) {
                (Some(_), Some(_)) => Err(CatalogError::Reference),
                (Some(existing), None) => Ok(Some(existing)),
                (None, Some(value)) => Ok(Some((key, value))),
                (None, None) => Ok(None),
            }
        })?;
        match found {
            None => Ok(HttpBinding::Document),
            Some((CatalogKey::HttpLabel, ValueRef::Bool(true))) => Ok(HttpBinding::Label),
            Some((CatalogKey::HttpQuery, ValueRef::Text(value))) => Ok(HttpBinding::Query(value)),
            Some((CatalogKey::HttpQueryParams, ValueRef::Bool(true))) => {
                Ok(HttpBinding::QueryParams)
            }
            Some((CatalogKey::HttpHeader, ValueRef::Text(value))) => Ok(HttpBinding::Header(value)),
            Some((CatalogKey::HttpPrefixHeaders, ValueRef::Text(value))) => {
                Ok(HttpBinding::PrefixHeaders(value))
            }
            Some((CatalogKey::HttpPayload, ValueRef::Bool(true))) => Ok(HttpBinding::Payload),
            Some((CatalogKey::HttpResponseCode, ValueRef::Bool(true))) => {
                Ok(HttpBinding::ResponseCode)
            }
            _ => Err(CatalogError::Reference),
        }
    }
    pub fn xml_flattened(self) -> Result<bool, CatalogError> {
        self.flag(CatalogKey::XmlFlattened)
    }
    pub fn xml_attribute(self) -> Result<bool, CatalogError> {
        self.flag(CatalogKey::XmlAttribute)
    }
    pub fn sparse(self) -> Result<bool, CatalogError> {
        self.flag(CatalogKey::Sparse)
    }
    pub fn streaming(self) -> Result<bool, CatalogError> {
        self.flag(CatalogKey::Streaming)
    }
    pub fn sensitive(self) -> Result<bool, CatalogError> {
        self.flag(CatalogKey::Sensitive)
    }
    pub fn idempotency_token(self) -> Result<bool, CatalogError> {
        self.flag(CatalogKey::IdempotencyToken)
    }
    pub fn host_label(self) -> Result<bool, CatalogError> {
        self.flag(CatalogKey::MemberHostLabel)
    }
    pub(super) fn validate(self) -> Result<(), CatalogError> {
        self.json_name()?;
        self.xml_name()?;
        self.ec2_query_name()?;
        self.media_type()?;
        self.xml_namespace()?;
        self.timestamp_format()?;
        self.binding()?;
        self.xml_flattened()?;
        self.xml_attribute()?;
        self.sparse()?;
        self.streaming()?;
        self.sensitive()?;
        self.idempotency_token()?;
        self.host_label()?;
        Ok(())
    }
}
