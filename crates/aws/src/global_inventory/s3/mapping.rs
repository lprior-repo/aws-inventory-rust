use super::*;

pub(super) fn map_bucket(
    ctx: &Context,
    bucket: &Bucket,
    region: Option<&str>,
    sections: &[(&str, Option<Value>)],
    tags: Option<BTreeMap<String, String>>,
) -> Result<PageBody, AwsScanError> {
    let name = required(bucket.name(), "bucket name")?;
    validate_bucket_name(&name)?;
    let arn = format!("arn:{}:s3:::{name}", ctx.partition);
    if bucket.bucket_arn().is_some_and(|value| value != arn) {
        return Err(AwsScanError::InvalidData(
            "ListBuckets ARN disagrees with bucket identity".into(),
        ));
    }
    let mut keys = BTreeSet::new();
    if sections.len() > SECTIONS.len()
        || sections
            .iter()
            .any(|(key, _)| !SECTIONS.contains(key) || !keys.insert(*key))
    {
        return Err(AwsScanError::InvalidData(
            "invalid or repeated bucket metadata section".into(),
        ));
    }
    let mut unavailable: Vec<&str> = SECTIONS
        .iter()
        .copied()
        .filter(|key| {
            !sections
                .iter()
                .any(|(section, value)| section == key && value.is_some())
        })
        .collect();
    if tags.is_none() {
        unavailable.push("tags");
    }
    let created = bucket
        .creation_date()
        .map(|value| value.to_millis())
        .transpose()
        .map_err(|_| AwsScanError::InvalidData("bucket creation timestamp out of range".into()))?;
    let mut properties = json!({"name":name,"bucket_name":name,"creation_date":created,"bucket_region":region,
        "region_resolved":region.is_some(),"details_available":region.is_some() && unavailable.is_empty() && sections.len() == SECTIONS.len(),
        "tags_available":tags.is_some(),
        "unavailable_sections":unavailable,"object_data_collected":false,"raw_policy_collected":false,
        "observation_detail":if region.is_some() {"control_plane_metadata"} else {"list_only"}});
    let object = properties
        .as_object_mut()
        .ok_or_else(|| AwsScanError::InvalidData("bucket properties are not an object".into()))?;
    for key in SECTIONS {
        let value = sections
            .iter()
            .find(|(section, _)| *section == key)
            .and_then(|(_, value)| value.clone())
            .unwrap_or(Value::Null);
        object.insert(key.to_owned(), value);
    }
    let mut resource = ctx.resource(KIND, arn, Some(name), properties, tags.unwrap_or_default())?;
    resource.region = region.unwrap_or("global").to_owned();
    resource.validate()?;
    let edges = bucket_edges(ctx, &resource)?;
    Ok(single(resource, edges))
}

pub(super) fn bucket_edges(ctx: &Context, resource: &Resource) -> Result<Vec<Edge>, AwsScanError> {
    let mut edges = Vec::new();
    if let Some(arn) = resource
        .properties
        .pointer("/replication/role_arn")
        .and_then(Value::as_str)
    {
        edges.push(reference(
            ctx,
            resource,
            arn,
            "ASSUMES_ROLE",
            json!({"field":"/replication/role_arn"}),
        )?);
    }
    if let Some(name) = resource
        .properties
        .pointer("/logging/target_bucket")
        .and_then(Value::as_str)
    {
        validate_bucket_name(name)?;
        edges.push(reference(
            ctx,
            resource,
            &format!("arn:{}:s3:::{name}", ctx.partition),
            "LOGS_TO",
            json!({"field":"logging.target_bucket","target_owner_resolution":"unknown"}),
        )?);
    }
    if let Some(host) = resource
        .properties
        .pointer("/website/redirect_host")
        .and_then(Value::as_str)
    {
        edges.push(reference(
            ctx,
            resource,
            &dns_reference(host)?,
            "REDIRECTS_TO_DNS",
            json!({"field":"website.redirect_host"}),
        )?);
    }
    for (path, field, relation) in [
        ("/encryption/rules", "kms_key_id", "ENCRYPTED_WITH"),
        ("/replication/rules", "destination_bucket", "REPLICATES_TO"),
        (
            "/replication/rules",
            "destination_kms_key_id",
            "REPLICA_ENCRYPTED_WITH",
        ),
        ("/notifications/topics", "arn", "NOTIFIES_TOPIC"),
        ("/notifications/queues", "arn", "NOTIFIES_QUEUE"),
        ("/notifications/functions", "arn", "NOTIFIES_FUNCTION"),
    ] {
        if let Some(values) = resource.properties.pointer(path).and_then(Value::as_array) {
            components(values.len(), "S3 references")?;
            for value in values {
                if let Some(arn) = value
                    .get(field)
                    .and_then(Value::as_str)
                    .filter(|value| value.starts_with("arn:"))
                {
                    if edges.len() >= MAX_COMPONENTS {
                        return Err(AwsScanError::ChildLimit);
                    }
                    edges.push(reference(ctx, resource, arn, relation, json!({"field":format!("{path}/{field}"),"configuration_id":value.get("id")}))?);
                }
            }
        }
    }
    edge_set(edges)
}

pub(super) fn validate_bucket_name(name: &str) -> Result<(), AwsScanError> {
    if name.is_empty()
        || name.len() > 255
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    {
        return Err(AwsScanError::InvalidData("invalid bucket name".into()));
    }
    Ok(())
}
