use super::super::super::CatalogError;
use crate::record::FieldId;

pub(in crate::catalog) fn field_name<'a>(
    data: &'a [u8],
    offsets: &'a [u8],
    id: FieldId,
) -> Result<&'a str, CatalogError> {
    let index = usize::try_from(id.value())
        .map_err(|_| CatalogError::Reference)?
        .checked_sub(1)
        .ok_or(CatalogError::Reference)?;
    let start = index.checked_mul(8).ok_or(CatalogError::Overflow)?;
    let end = start.checked_add(8).ok_or(CatalogError::Overflow)?;
    let pair = offsets.get(start..end).ok_or(CatalogError::Reference)?;
    let start = u32::from_be_bytes(
        pair.get(..4)
            .ok_or(CatalogError::Reference)?
            .try_into()
            .map_err(|_| CatalogError::Reference)?,
    );
    let end = u32::from_be_bytes(
        pair.get(4..8)
            .ok_or(CatalogError::Reference)?
            .try_into()
            .map_err(|_| CatalogError::Reference)?,
    );
    let start = usize::try_from(start).map_err(|_| CatalogError::Reference)?;
    let end = usize::try_from(end).map_err(|_| CatalogError::Reference)?;
    let bytes = data.get(start..end).ok_or(CatalogError::Reference)?;
    std::str::from_utf8(bytes).map_err(|_| CatalogError::Reference)
}
