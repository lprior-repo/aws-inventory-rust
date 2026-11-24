use super::*;

pub(super) fn fields(shape: &Shape) -> Result<Vec<(CatalogKey, Value)>, CompileError> {
    let mut fields = traits(&shape.traits)?;
    if let ShapeKind::Resource {
        list,
        read,
        identifiers,
    } = &shape.kind
    {
        let operations = list
            .iter()
            .chain(read.iter())
            .map(|target| super::super::encode::name(&target.target))
            .collect();
        fields.push((CatalogKey::ResourceOperations, Value::List(operations)));
        fields.push((
            CatalogKey::ResourceIdentifiers,
            Value::List(
                identifiers
                    .keys()
                    .map(|name| Value::Text(name.clone()))
                    .collect(),
            ),
        ));
    }
    Ok(fields)
}

pub(super) fn traits(
    traits: &BTreeMap<String, serde_json::Value>,
) -> Result<Vec<(CatalogKey, Value)>, CompileError> {
    [
        ("smithy.api#references", CatalogKey::References),
        ("aws.api#arnReference", CatalogKey::ArnReference),
    ]
    .into_iter()
    .filter_map(|(name, key)| traits.get(name).map(|value| (name, key, value)))
    .map(|(name, key, value)| {
        serde_json::to_string(value)
            .map(|payload| (key, Value::Text(payload)))
            .map_err(|_| CompileError::Trait(name.to_owned()))
    })
    .collect()
}
