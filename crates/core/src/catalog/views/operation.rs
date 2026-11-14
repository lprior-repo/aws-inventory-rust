use super::{CatalogError, CatalogKey, bool_field, integer, optional_list, optional_text};
use crate::record::{MapRef, ValueRef};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HttpOperation<'a> {
    pub method: &'a str,
    pub uri: &'a str,
    pub code: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Paginator<'a> {
    pub input_token: Option<&'a str>,
    pub output_token: Option<&'a str>,
    pub items: Option<&'a str>,
    pub page_size: Option<&'a str>,
    pub max_page_size: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompressionAlgorithm {
    Gzip,
}

#[derive(Clone, Copy, Debug)]
pub struct Operation<'a> {
    map: MapRef<'a>,
    shape_count: usize,
}
impl<'a> Operation<'a> {
    pub(super) const fn new(map: MapRef<'a>, shape_count: usize) -> Self {
        Self { map, shape_count }
    }
    pub fn request_compression(self) -> Result<Option<CompressionAlgorithm>, CatalogError> {
        let key = CatalogKey::RequestCompression;
        let Some(items) = optional_list(self.map, key)? else {
            return Ok(None);
        };
        let mut found = None;
        let mut any = false;
        for item in items {
            any = true;
            match item? {
                ValueRef::Text(value) if value.eq_ignore_ascii_case("gzip") => {
                    found.get_or_insert(CompressionAlgorithm::Gzip);
                }
                _ => return Err(CatalogError::Field(key)),
            }
        }
        if !any {
            return Err(CatalogError::Field(key));
        }
        Ok(found)
    }
    pub fn name(self) -> Result<&'a str, CatalogError> {
        super::super::text(self.map, CatalogKey::Name)
    }
    pub fn flags(self) -> Result<u8, CatalogError> {
        let flags = u8::try_from(integer(self.map, CatalogKey::Flags)?)
            .map_err(|_| CatalogError::Reference)?;
        if flags > 31 {
            return Err(CatalogError::Reference);
        }
        Ok(flags)
    }
    pub fn input(self) -> Result<super::Target, CatalogError> {
        super::shape::target(integer(self.map, CatalogKey::Input)?, self.shape_count)
    }
    pub fn output(self) -> Result<super::Target, CatalogError> {
        super::shape::target(integer(self.map, CatalogKey::Output)?, self.shape_count)
    }
    pub fn http(self) -> Result<Option<HttpOperation<'a>>, CatalogError> {
        let method = optional_text(self.map, CatalogKey::HttpMethod)?;
        let uri = optional_text(self.map, CatalogKey::HttpUri)?;
        match (method, uri) {
            (None, None) => Ok(None),
            (Some(method), Some(uri)) => Ok(Some(HttpOperation {
                method,
                uri,
                code: optional_code(self.map, CatalogKey::HttpSuccess)?.unwrap_or(200),
            })),
            _ => Err(CatalogError::Reference),
        }
    }
    pub fn host_prefix(self) -> Result<Option<&'a str>, CatalogError> {
        optional_text(self.map, CatalogKey::HostPrefix)
    }
    pub fn checksum_required(self) -> Result<bool, CatalogError> {
        bool_field(self.map, CatalogKey::HttpChecksum)
    }
    pub fn s3_unwrapped_xml_output(self) -> Result<bool, CatalogError> {
        bool_field(self.map, CatalogKey::S3UnwrappedXmlOutput)
    }
    pub fn paginator(self) -> Result<Option<Paginator<'a>>, CatalogError> {
        if self.flags()? & 2 == 0 {
            return Ok(None);
        }
        let paginator = Paginator {
            input_token: optional_text(self.map, CatalogKey::PaginatedInput)?,
            output_token: optional_text(self.map, CatalogKey::PaginatedOutput)?,
            items: optional_text(self.map, CatalogKey::PaginatedResult)?,
            page_size: optional_text(self.map, CatalogKey::PaginatedPageSize)?,
            max_page_size: optional_u64(self.map, CatalogKey::PaginatedLimit)?,
        };
        Ok((paginator.input_token.is_some()
            || paginator.output_token.is_some()
            || paginator.items.is_some()
            || paginator.page_size.is_some()
            || paginator.max_page_size.is_some())
        .then_some(paginator))
    }
    pub fn errors(
        self,
    ) -> Result<impl Iterator<Item = Result<super::Target, CatalogError>> + 'a, CatalogError> {
        Ok(optional_list(self.map, CatalogKey::Errors)?
            .into_iter()
            .flatten()
            .map(move |value| {
                super::shape::target(super::super::number(value?)?, self.shape_count)
            }))
    }
    pub(super) fn validate(self) -> Result<(), CatalogError> {
        self.name()?;
        self.flags()?;
        self.input()?;
        self.output()?;
        self.http()?;
        self.host_prefix()?;
        self.checksum_required()?;
        self.s3_unwrapped_xml_output()?;
        self.paginator()?;
        self.errors()?.try_for_each(|item| item.map(|_| ()))?;
        self.request_compression()?;
        Ok(())
    }
}

fn optional_code(map: MapRef<'_>, key: CatalogKey) -> Result<Option<u16>, CatalogError> {
    let Some(value) = map.get_field(key.field()?)? else {
        return Ok(None);
    };
    let ValueRef::Int(value) = value else {
        return Err(CatalogError::Field(key));
    };
    let code = u16::try_from(value).map_err(|_| CatalogError::Field(key))?;
    if !(100..=599).contains(&code) {
        return Err(CatalogError::Field(key));
    }
    Ok(Some(code))
}

fn optional_u64(map: MapRef<'_>, key: CatalogKey) -> Result<Option<u64>, CatalogError> {
    match map.get_field(key.field()?)? {
        None => Ok(None),
        Some(ValueRef::Int(value)) => u64::try_from(value)
            .map(Some)
            .map_err(|_| CatalogError::Field(key)),
        _ => Err(CatalogError::Field(key)),
    }
}
