use serde::{
    Deserialize,
    de::{self, MapAccess, SeqAccess, Visitor},
};
use std::{
    collections::BTreeMap,
    fs,
    io::{self, Read},
    path::{Component, Path, PathBuf},
};
use thiserror::Error;
const MAX_FILE_BYTES: u64 = 67_108_864;
const MAX_ENTRIES: usize = 65_536;

pub(crate) fn read_model<T: serde::de::DeserializeOwned>(file: &Path) -> Result<T, Error> {
    let bytes = read_bytes(file, MAX_FILE_BYTES)?;
    let invalid = |source| Error::InvalidJson {
        file: file.to_path_buf(),
        source,
    };
    let unbound = closure::unbound_operations(&bytes).map_err(invalid)?;
    if unbound.is_empty() {
        return serde_json::from_slice(&bytes).map_err(invalid);
    }
    let value = bound_value(&bytes, &unbound).map_err(invalid)?;
    serde_json::from_value(value).map_err(invalid)
}
pub(crate) fn read_value(file: &Path) -> Result<serde_json::Value, Error> {
    let bytes = read_bytes(file, MAX_FILE_BYTES)?;
    let invalid = |source| Error::InvalidJson {
        file: file.to_path_buf(),
        source,
    };
    let unbound = closure::unbound_operations(&bytes).map_err(invalid)?;
    bound_value(&bytes, &unbound).map_err(invalid)
}
fn bound_value(bytes: &[u8], unbound: &[String]) -> Result<serde_json::Value, serde_json::Error> {
    let mut value = serde_json::from_slice::<UniqueValue>(bytes)?.0;
    if let Some(shapes) = value
        .get_mut("shapes")
        .and_then(serde_json::Value::as_object_mut)
    {
        shapes.retain(|id, _| unbound.binary_search(id).is_err());
    }
    Ok(value)
}
mod closure;
struct UniqueValue(serde_json::Value);

impl<'de> Deserialize<'de> for UniqueValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor)
    }
}

struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = UniqueValue;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a JSON value with unique object keys")
    }

    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(UniqueValue(serde_json::Value::Null))
    }

    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
        Ok(UniqueValue(value.into()))
    }

    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
        Ok(UniqueValue(value.into()))
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
        Ok(UniqueValue(value.into()))
    }

    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> {
        serde_json::Number::from_f64(value)
            .map(serde_json::Value::Number)
            .map(UniqueValue)
            .ok_or_else(|| E::custom("invalid JSON number"))
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        Ok(UniqueValue(value.into()))
    }

    fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
        Ok(UniqueValue(value.into()))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
        let mut values = Vec::new();
        while let Some(value) = access.next_element::<UniqueValue>()? {
            values.push(value.0);
        }
        Ok(UniqueValue(values.into()))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
        let mut values = serde_json::Map::new();
        while let Some((key, value)) = access.next_entry::<String, UniqueValue>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom(format!("duplicate key {key:?}")));
            }
            values.insert(key, value.0);
        }
        Ok(UniqueValue(values.into()))
    }
}
pub(crate) fn discover_models(models: &Path) -> Result<Vec<PathBuf>, Error> {
    discover(models, MAX_ENTRIES)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{file}: {source}")]
    Io { file: PathBuf, source: io::Error },
    #[error("file exceeds {limit} bytes: {file}")]
    TooLarge { file: PathBuf, limit: u64 },
    #[error("more than {limit} directory entries under {directory}")]
    TooManyEntries { directory: PathBuf, limit: usize },
    #[error("invalid JSON in {file}: {source}")]
    InvalidJson {
        file: PathBuf,
        source: serde_json::Error,
    },
    #[error("no models found")]
    NoModels,
    #[error("invalid model path: {0}")]
    InvalidPath(PathBuf),
}
mod input;
#[cfg(test)]
pub(crate) use input::InputFailure;
pub(crate) use input::{InputError, InputModel, InputSpec, resolve_input};
#[cfg(test)]
pub(crate) fn read_json<T: serde::de::DeserializeOwned>(
    file: &Path,
    limit: u64,
) -> Result<T, Error> {
    let bytes = read_bytes(file, limit)?;
    serde_json::from_slice(&bytes).map_err(|source| Error::InvalidJson {
        file: file.to_path_buf(),
        source,
    })
}

fn read_bytes(file: &Path, limit: u64) -> Result<Vec<u8>, Error> {
    let handle = fs::File::open(file).map_err(|source| Error::Io {
        file: file.to_path_buf(),
        source,
    })?;
    let mut bytes = Vec::new();
    handle
        .take(limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|source| Error::Io {
            file: file.to_path_buf(),
            source,
        })?;
    ensure_size(file, limit, bytes.len())?;
    Ok(bytes)
}

fn ensure_size(file: &Path, limit: u64, length: usize) -> Result<(), Error> {
    if u64::try_from(length).map_or(true, |length| length > limit) {
        return Err(Error::TooLarge {
            file: file.to_path_buf(),
            limit,
        });
    }
    Ok(())
}
pub(crate) fn discover(models: &Path, max_entries: usize) -> Result<Vec<PathBuf>, Error> {
    let mut pending = vec![models.to_path_buf()];
    let mut files = Vec::new();
    let mut remaining = max_entries;
    while let Some(directory) = pending.pop() {
        discover_directory(
            &directory,
            max_entries,
            &mut remaining,
            &mut pending,
            &mut files,
        )?;
    }
    files.sort();
    Ok(files)
}

fn discover_directory(
    directory: &Path,
    max_entries: usize,
    remaining: &mut usize,
    pending: &mut Vec<PathBuf>,
    files: &mut Vec<PathBuf>,
) -> Result<(), Error> {
    fs::read_dir(directory)
        .map_err(|source| Error::Io {
            file: directory.to_path_buf(),
            source,
        })?
        .try_for_each(|entry| {
            discover_entry(entry, directory, max_entries, remaining, pending, files)
        })
}

fn discover_entry(
    entry: io::Result<fs::DirEntry>,
    directory: &Path,
    max_entries: usize,
    remaining: &mut usize,
    pending: &mut Vec<PathBuf>,
    files: &mut Vec<PathBuf>,
) -> Result<(), Error> {
    let entry = entry.map_err(|source| Error::Io {
        file: directory.to_path_buf(),
        source,
    })?;
    *remaining = remaining
        .checked_sub(1)
        .ok_or_else(|| Error::TooManyEntries {
            directory: directory.to_path_buf(),
            limit: max_entries,
        })?;
    collect_entry(entry, pending, files)
}

fn collect_entry(
    entry: fs::DirEntry,
    pending: &mut Vec<PathBuf>,
    files: &mut Vec<PathBuf>,
) -> Result<(), Error> {
    let path = entry.path();
    let kind = entry.file_type().map_err(|source| Error::Io {
        file: path.clone(),
        source,
    })?;
    if kind.is_dir() {
        pending.push(path);
    } else if kind.is_file() && path.extension().is_some_and(|ext| ext == "json") {
        files.push(path);
    }
    Ok(())
}

pub(crate) fn service_name<'a>(models: &Path, file: &'a Path) -> Result<&'a str, Error> {
    let relative = file
        .strip_prefix(models)
        .map_err(|_| Error::InvalidPath(file.to_path_buf()))?;
    if file.extension().is_none_or(|extension| extension != "json") {
        return Err(Error::InvalidPath(file.to_path_buf()));
    }
    let mut components = relative.components();
    let Some(Component::Normal(service)) = components.next() else {
        return Err(Error::InvalidPath(file.to_path_buf()));
    };
    if !matches!(components.next(), Some(Component::Normal(_))) {
        return Err(Error::InvalidPath(file.to_path_buf()));
    }
    if components.any(|component| !matches!(component, Component::Normal(_))) {
        return Err(Error::InvalidPath(file.to_path_buf()));
    }
    service
        .to_str()
        .ok_or_else(|| Error::InvalidPath(file.to_path_buf()))
}

pub(crate) fn unique_map<'de, D, V>(deserializer: D) -> Result<BTreeMap<String, V>, D::Error>
where
    D: de::Deserializer<'de>,
    V: Deserialize<'de>,
{
    struct MapVisitor<V>(std::marker::PhantomData<V>);
    impl<'de, V: Deserialize<'de>> Visitor<'de> for MapVisitor<V> {
        type Value = BTreeMap<String, V>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a map with unique string keys")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
            let mut values = BTreeMap::new();
            while let Some((key, value)) = access.next_entry::<String, V>()? {
                if values.insert(key.clone(), value).is_some() {
                    return Err(de::Error::custom(format!("duplicate key {key:?}")));
                }
            }
            Ok(values)
        }
    }
    deserializer.deserialize_map(MapVisitor(std::marker::PhantomData))
}
