use super::commit::quote;
use super::{BinaryStore, StoreError};
use inventory_core::inventory::{Observation, ScalarType};
use inventory_core::record::{FieldId, ValueRef};
use sqlx::{QueryBuilder, Row, Sqlite, Transaction};

pub(super) async fn upsert_kind(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    observation: &Observation,
    now: i64,
) -> Result<(), StoreError> {
    let schema = store
        .registry
        .schemas()
        .iter()
        .find(|schema| schema.kind() == &observation.key().kind)
        .ok_or(StoreError::Invalid(
            "resource kind is absent from the catalog schema",
        ))?;
    let first_seen = sqlx::query(
        "SELECT first_seen FROM resources WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4",
    )
    .bind(observation.key().account.as_str())
    .bind(observation.key().region.as_str())
    .bind(observation.key().kind.as_str())
    .bind(observation.key().id.as_str())
    .fetch_one(&mut **tx)
    .await?
    .try_get::<i64, _>("first_seen")?;
    let mut query = QueryBuilder::<Sqlite>::new("INSERT INTO ");
    query
        .push(quote(schema.table().as_str()))
        .push("(")
        .push("account,region,id,content_hash,first_seen,last_changed,record");
    for column in schema.columns() {
        append_column_name(&mut query, column.name(), column.scalar());
    }
    query
        .push(") VALUES(")
        .push_bind(observation.key().account.as_str())
        .push(",")
        .push_bind(observation.key().region.as_str())
        .push(",")
        .push_bind(observation.key().id.as_str())
        .push(",")
        .push_bind(observation.content_hash().as_bytes().as_slice())
        .push(",")
        .push_bind(first_seen)
        .push(",")
        .push_bind(now)
        .push(",")
        .push_bind(observation.record().as_bytes());
    for column in schema.columns() {
        query.push(",");
        bind_column(&mut query, observation, column.path(), column.scalar())?;
    }
    query.push(") ON CONFLICT(account,region,id) DO UPDATE SET content_hash=excluded.content_hash,last_changed=excluded.last_changed,record=excluded.record");
    for column in schema.columns() {
        append_column_update(&mut query, column.name(), column.scalar());
    }
    query.build().execute(&mut **tx).await?;
    Ok(())
}

fn append_column_name(query: &mut QueryBuilder<Sqlite>, name: &str, scalar: ScalarType) {
    match scalar {
        ScalarType::Real => {
            query
                .push(",")
                .push(quote(name))
                .push(",")
                .push(quote(&format!("{name}__nonfinite")));
        }
        ScalarType::Timestamp => {
            query
                .push(",")
                .push(quote(&format!("{name}__seconds")))
                .push(",")
                .push(quote(&format!("{name}__nanos")));
        }
        _ => {
            query.push(",").push(quote(name));
        }
    }
}

fn append_column_update(query: &mut QueryBuilder<Sqlite>, name: &str, scalar: ScalarType) {
    match scalar {
        ScalarType::Real => {
            query
                .push(",")
                .push(quote(name))
                .push("=excluded.")
                .push(quote(name));
            let flag = format!("{name}__nonfinite");
            query
                .push(",")
                .push(quote(&flag))
                .push("=excluded.")
                .push(quote(&flag));
        }
        ScalarType::Timestamp => {
            for suffix in ["__seconds", "__nanos"] {
                let column = format!("{name}{suffix}");
                query
                    .push(",")
                    .push(quote(&column))
                    .push("=excluded.")
                    .push(quote(&column));
            }
        }
        _ => {
            query
                .push(",")
                .push(quote(name))
                .push("=excluded.")
                .push(quote(name));
        }
    }
}

fn bind_column(
    query: &mut QueryBuilder<Sqlite>,
    observation: &Observation,
    path: &[FieldId],
    scalar: ScalarType,
) -> Result<(), StoreError> {
    let value = path.iter().try_fold(
        Some(ValueRef::Map(observation.record().root())),
        |current, field| match current {
            Some(ValueRef::Map(map)) => map.get_field(*field).map_err(StoreError::from),
            _ => Ok(None),
        },
    )?;
    let value = if path.is_empty() {
        inventory_core::record::scalar_item(observation.record().root())?
    } else {
        value
    };
    match scalar {
        ScalarType::Bool => {
            let value = match value {
                Some(ValueRef::Bool(value)) => Some(i64::from(value)),
                None | Some(ValueRef::Null) => None,
                _ => {
                    return Err(StoreError::Invalid(
                        "record scalar does not match boolean schema",
                    ));
                }
            };
            query.push_bind(value);
        }
        ScalarType::Integer => {
            let value = match value {
                Some(ValueRef::Int(value)) => Some(value),
                None | Some(ValueRef::Null) => None,
                _ => {
                    return Err(StoreError::Invalid(
                        "record scalar does not match integer schema",
                    ));
                }
            };
            query.push_bind(value);
        }
        ScalarType::Text => {
            let value = match value {
                Some(ValueRef::Text(value)) => Some(value),
                None | Some(ValueRef::Null) => None,
                _ => {
                    return Err(StoreError::Invalid(
                        "record scalar does not match text schema",
                    ));
                }
            };
            query.push_bind(value);
        }
        ScalarType::Blob => {
            let value = match value {
                Some(ValueRef::Bytes(value)) => Some(value),
                None | Some(ValueRef::Null) => None,
                _ => {
                    return Err(StoreError::Invalid(
                        "record scalar does not match blob schema",
                    ));
                }
            };
            query.push_bind(value);
        }
        ScalarType::Real => {
            let (value, flag) = real_value(value)?;
            query.push_bind(value).push_bind(flag);
        }
        ScalarType::Timestamp => {
            let (seconds, nanos) = time_value(value)?;
            query.push_bind(seconds).push_bind(nanos);
        }
    }
    Ok(())
}

fn real_value(value: Option<ValueRef<'_>>) -> Result<(Option<f64>, Option<i64>), StoreError> {
    match value {
        None | Some(ValueRef::Null) => Ok((None, None)),
        Some(ValueRef::Float(value)) => {
            let number = value.to_f64();
            if number.is_nan() {
                Ok((None, Some(2)))
            } else if number.is_infinite() {
                Ok((None, Some(if number.is_sign_positive() { 1 } else { -1 })))
            } else {
                Ok((Some(number), Some(0)))
            }
        }
        _ => Err(StoreError::Invalid(
            "record scalar does not match real schema",
        )),
    }
}

fn time_value(value: Option<ValueRef<'_>>) -> Result<(Option<i64>, Option<i64>), StoreError> {
    match value {
        None | Some(ValueRef::Null) => Ok((None, None)),
        Some(ValueRef::Time(value)) => {
            Ok((Some(value.seconds()), Some(i64::from(value.nanoseconds()))))
        }
        _ => Err(StoreError::Invalid(
            "record scalar does not match timestamp schema",
        )),
    }
}
