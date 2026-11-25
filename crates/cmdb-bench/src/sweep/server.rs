use futures_util::{StreamExt, TryStreamExt, stream};
use sqlx::{Connection, PgConnection};
const SETTINGS: &[&str] = &[
    "server_version",
    "shared_buffers",
    "effective_cache_size",
    "work_mem",
    "max_wal_size",
    "checkpoint_timeout",
    "synchronous_commit",
    "fsync",
    "full_page_writes",
    "wal_level",
    "max_connections",
    "max_locks_per_transaction",
];
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Counters {
    pub wal: u64,
    pub logged: u64,
    pub unlogged: u64,
}
impl Counters {
    pub(crate) fn since(self, earlier: Self) -> Result<Self, String> {
        let difference = |now: u64, before| {
            now.checked_sub(before)
                .ok_or_else(|| "PostgreSQL counter moved backwards".to_string())
        };
        Ok(Self {
            wal: difference(self.wal, earlier.wal)?,
            logged: difference(self.logged, earlier.logged)?,
            unlogged: difference(self.unlogged, earlier.unlogged)?,
        })
    }
}
pub(crate) async fn connect(url: &str) -> Result<PgConnection, String> {
    PgConnection::connect(url)
        .await
        .map_err(|error| error.to_string())
}
pub(crate) async fn settings(
    connection: &mut PgConnection,
) -> Result<Vec<(String, String)>, String> {
    sqlx::query_as("SELECT 'server.' || name, current_setting(name) FROM unnest($1::text[]) WITH ORDINALITY AS settings(name, ordinal) ORDER BY ordinal")
        .bind(SETTINGS).fetch_all(connection).await.map_err(|error| error.to_string())
}
pub(crate) fn publishable(settings: &[(String, String)]) -> bool {
    [
        ("server.fsync", "on"),
        ("commit_state", "clean"),
        ("collector.success", "yes"),
    ]
    .iter()
    .all(|(key, expected)| {
        settings
            .iter()
            .any(|(actual, value)| actual == key && value == expected)
    }) && settings
        .iter()
        .any(|(key, value)| key == "os" && value.starts_with("Linux "))
}
pub(crate) async fn database_bytes(connection: &mut PgConnection) -> Result<u64, String> {
    let size: i64 = sqlx::query_scalar("SELECT pg_database_size(current_database())")
        .fetch_one(connection)
        .await
        .map_err(|error| error.to_string())?;
    unsigned(size)
}
pub(crate) async fn counters(connection: &mut PgConnection) -> Result<Counters, String> {
    settle(connection).await?;
    sqlx::query("SELECT pg_stat_clear_snapshot()")
        .execute(&mut *connection)
        .await
        .map_err(|error| error.to_string())?;
    let (wal, logged, unlogged): (i64, i64, i64) = sqlx::query_as(
        "SELECT pg_wal_lsn_diff(pg_current_wal_lsn(), '0/0')::bigint, \
         COALESCE(sum(s.n_tup_ins + s.n_tup_upd + s.n_tup_del) FILTER (WHERE c.relpersistence = 'p'), 0)::bigint, \
         COALESCE(sum(s.n_tup_ins + s.n_tup_upd + s.n_tup_del) FILTER (WHERE c.relpersistence = 'u'), 0)::bigint \
         FROM pg_stat_user_tables s JOIN pg_class c ON c.oid = s.relid")
        .fetch_one(connection).await.map_err(|error| error.to_string())?;
    Ok(Counters {
        wal: unsigned(wal)?,
        logged: unsigned(logged)?,
        unlogged: unsigned(unlogged)?,
    })
}
async fn settle(connection: &mut PgConnection) -> Result<(), String> {
    let readings = stream::try_unfold(connection, |connection| async move {
        let others: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_stat_activity WHERE datname = current_database() AND pid <> pg_backend_pid()")
            .fetch_one(&mut *connection).await.map_err(|error| error.to_string())?;
        if others != 0 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        Ok::<_, String>(Some((others == 0, connection)))
    });
    let ready = readings.take(50).try_any(|ready| async move { ready });
    match tokio::time::timeout(std::time::Duration::from_secs(5), ready).await {
        Ok(Ok(true)) => Ok(()),
        Ok(Err(error)) => Err(error),
        Ok(Ok(false)) | Err(_) => Err("another session stayed connected for five seconds".into()),
    }
}
fn unsigned(value: i64) -> Result<u64, String> {
    u64::try_from(value).map_err(|error| error.to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    use inventory_harness::postgres::{TestDatabase, admin_url};
    #[tokio::test]
    #[ignore = "requires INVENTORY_POSTGRES_URL"]
    async fn server_settings_and_logged_unlogged_deltas_follow_real_writes()
    -> Result<(), Box<dyn std::error::Error>> {
        let database = TestDatabase::create(&admin_url()?).await?;
        let result = exercise(database.url()).await;
        database.finish().await?;
        Ok(result?)
    }
    async fn exercise(url: &str) -> Result<(), String> {
        let mut reader = connect(url).await?;
        let settings = settings(&mut reader).await?;
        assert_eq!(settings.len(), SETTINGS.len());
        assert!(settings.iter().all(|(_, value)| !value.is_empty()));
        let before = counters(&mut reader).await?;
        let mut writer = connect(url).await?;
        sqlx::raw_sql("CREATE TABLE kept(n int); CREATE UNLOGGED TABLE scratch(n int); INSERT INTO kept VALUES (1),(2),(3); INSERT INTO scratch VALUES (1),(2)")
            .execute(&mut writer).await.map_err(|error| error.to_string())?;
        writer.close().await.map_err(|error| error.to_string())?;
        let delta = counters(&mut reader).await?.since(before)?;
        assert_eq!(delta.logged, 3);
        assert_eq!(delta.unlogged, 2);
        assert!(delta.wal > 0);
        assert!(database_bytes(&mut reader).await? > 0);
        reader.close().await.map_err(|error| error.to_string())
    }
    #[test]
    fn fsync_is_required_and_counter_resets_are_rejected() {
        assert!(!publishable(&[("server.fsync".into(), "off".into())]));
        assert!(publishable(&[
            ("server.fsync".into(), "on".into()),
            ("commit_state".into(), "clean".into()),
            ("collector.success".into(), "yes".into()),
            ("os".into(), "Linux fixture".into())
        ]));
        assert!(
            Counters::default()
                .since(Counters {
                    wal: 1,
                    ..Counters::default()
                })
                .is_err()
        );
    }
    #[test]
    fn failed_or_dirty_runs_are_not_publishable() {
        let clean = vec![
            ("server.fsync".into(), "on".into()),
            ("commit_state".into(), "clean".into()),
            ("collector.success".into(), "yes".into()),
            ("os".into(), "Linux fixture".into()),
        ];
        assert!(publishable(&clean));
        let mut failed = clean.clone();
        failed
            .iter_mut()
            .filter(|(key, _)| key == "collector.success")
            .for_each(|(_, value)| *value = "no".into());
        assert!(!publishable(&failed));
        let mut dirty = clean;
        dirty
            .iter_mut()
            .filter(|(key, _)| key == "commit_state")
            .for_each(|(_, value)| *value = "dirty".into());
        assert!(!publishable(&dirty));
    }
    #[tokio::test]
    #[ignore = "requires INVENTORY_POSTGRES_URL"]
    async fn connected_writer_exhausts_the_virtual_five_second_settle_bound()
    -> Result<(), Box<dyn std::error::Error>> {
        let database = TestDatabase::create(&admin_url()?).await?;
        let mut reader = connect(database.url()).await?;
        let writer = connect(database.url()).await?;
        tokio::time::pause();
        let started = tokio::time::Instant::now();
        let result = settle(&mut reader).await;
        let elapsed = started.elapsed();
        tokio::time::resume();
        writer.close().await?;
        reader.close().await?;
        database.finish().await?;
        assert_eq!(
            result,
            Err("another session stayed connected for five seconds".into())
        );
        assert!(
            (std::time::Duration::from_secs(5)..=std::time::Duration::from_millis(5001))
                .contains(&elapsed)
        );
        Ok(())
    }
}
