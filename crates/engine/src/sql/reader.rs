use super::SqlError;
use sqlx::{Connection, PgConnection, postgres::PgConnectOptions};
use std::{str::FromStr, time::Duration};

const PRODUCT_SCHEMAS: [&str; 2] = ["inventory", "native_scan"];

const READER_CHECK: &str = r"
SELECT (EXISTS (SELECT 1 FROM pg_catalog.pg_namespace WHERE nspname = $1::pg_catalog.text)
AND pg_catalog.has_schema_privilege(current_user, $1::pg_catalog.text, 'USAGE')
AND NOT EXISTS (
    SELECT 1 FROM pg_catalog.pg_roles r
    WHERE (r.rolname = current_user OR r.rolname = session_user
        OR pg_catalog.pg_has_role(current_user, r.oid, 'MEMBER')
        OR pg_catalog.pg_has_role(session_user, r.oid, 'MEMBER'))
    AND (r.rolsuper OR r.rolcreaterole OR r.rolcreatedb OR r.rolreplication OR r.rolbypassrls
        OR r.rolname IN ('pg_write_all_data', 'pg_signal_backend',
            'pg_execute_server_program', 'pg_write_server_files', 'pg_read_all_data',
            'pg_read_server_files', 'pg_monitor', 'pg_read_all_stats')
        OR EXISTS (SELECT 1 FROM pg_catalog.pg_namespace n
            WHERE n.nspname::pg_catalog.text = ANY($2::pg_catalog.text[])
            AND pg_catalog.has_schema_privilege(r.oid, n.oid, 'CREATE'))
        OR EXISTS (SELECT 1 FROM pg_catalog.pg_database d WHERE d.datname = pg_catalog.current_database()
            AND pg_catalog.pg_has_role(r.oid, d.datdba, 'USAGE'))
        OR EXISTS (SELECT 1 FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname::pg_catalog.text = ANY($2::pg_catalog.text[]) AND c.relkind IN ('r','p','v','m','f')
            AND (pg_catalog.has_table_privilege(r.oid, c.oid, 'INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER')
                OR pg_catalog.has_any_column_privilege(r.oid, c.oid, 'INSERT,UPDATE,REFERENCES')
                OR pg_catalog.pg_has_role(r.oid, c.relowner, 'USAGE')))
        OR EXISTS (SELECT 1 FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname::pg_catalog.text = ANY($2::pg_catalog.text[])
            AND CASE WHEN c.relkind = 'S'
                THEN pg_catalog.has_sequence_privilege(r.oid, c.oid, 'USAGE,UPDATE') ELSE false END))
)), pg_catalog.pg_backend_pid(), (SELECT (EXTRACT(EPOCH FROM a.backend_start) * 1000000)::bigint
    FROM pg_catalog.pg_stat_activity a WHERE a.pid = pg_catalog.pg_backend_pid())";

const TERMINATE_SQL: &str = "SELECT pg_catalog.pg_terminate_backend(a.pid, 1000) FROM pg_catalog.pg_stat_activity a WHERE a.pid = $1 AND (EXTRACT(EPOCH FROM a.backend_start) * 1000000)::bigint = $2";

#[derive(Clone, Copy)]
pub(super) struct Backend {
    pub(super) pid: i32,
    started_us: i64,
}

#[derive(Clone, Copy)]
pub(super) enum Namespace {
    Inventory,
    NativeScan,
}

impl Namespace {
    pub(super) const fn schema(self) -> &'static str {
        match self {
            Self::Inventory => "inventory",
            Self::NativeScan => "native_scan",
        }
    }
}

#[derive(Clone)]
pub struct ReaderUrl {
    options: PgConnectOptions,
    pub(super) namespace: Namespace,
    #[cfg(test)]
    pub(super) server_millis: Option<u64>,
}

impl ReaderUrl {
    pub async fn establish(url: &str) -> Result<Self, SqlError> {
        Self::establish_in(url, Namespace::Inventory).await
    }

    pub async fn establish_native(url: &str) -> Result<Self, SqlError> {
        Self::establish_in(url, Namespace::NativeScan).await
    }

    #[tracing::instrument(skip_all)]
    async fn establish_in(url: &str, namespace: Namespace) -> Result<Self, SqlError> {
        if url.len() > 8_192
            || !(url.starts_with("postgres://") || url.starts_with("postgresql://"))
        {
            return Err(SqlError::InvalidUrl);
        }
        let options = PgConnectOptions::from_str(url).map_err(|_| SqlError::InvalidUrl)?;
        let reader = Self {
            options,
            namespace,
            #[cfg(test)]
            server_millis: None,
        };
        let (connection, _) = reader.connect().await?;
        close(connection).await?;
        Ok(reader)
    }

    #[tracing::instrument(skip_all)]
    pub(super) async fn connect(&self) -> Result<(PgConnection, Backend), SqlError> {
        tokio::time::timeout(Duration::from_secs(5), self.connect_checked())
            .await
            .map_err(|_| SqlError::Timeout)?
    }

    async fn connect_checked(&self) -> Result<(PgConnection, Backend), SqlError> {
        let mut connection = PgConnection::connect_with(&self.options).await?;
        if !crate::postgres_version::supported(connection.server_version_num()) {
            return Err(SqlError::UnsupportedServerVersion);
        }
        sqlx::query("SET search_path = pg_catalog")
            .execute(&mut connection)
            .await?;
        let (reader, pid, started) =
            sqlx::query_as::<_, (bool, i32, Option<i64>)>(READER_CHECK)
                .bind(self.namespace.schema())
                .bind(PRODUCT_SCHEMAS.to_vec())
                .fetch_one(&mut connection)
                .await?;
        if !reader {
            return Err(SqlError::ReaderRequired);
        }
        let started = started.ok_or(SqlError::SettingRequired {
            name: "track_activities",
            required: "on",
        })?;
        let standard = sqlx::query_scalar::<_, bool>(
            "SELECT pg_catalog.current_setting('standard_conforming_strings') = 'on'",
        )
        .fetch_one(&mut connection)
        .await?;
        if !standard {
            return Err(SqlError::SettingRequired {
                name: "standard_conforming_strings",
                required: "on",
            });
        }
        Ok((connection, Backend { pid, started_us: started }))
    }

    pub(super) async fn terminate(&self, backend: &Backend) -> Result<(), SqlError> {
        tokio::time::timeout(Duration::from_secs(5), async {
            let mut connection = PgConnection::connect_with(&self.options).await?;
            let result = signal_and_wait(&mut connection, backend).await;
            let closed = close(connection).await;
            result?;
            closed
        })
        .await
        .map_err(|_| SqlError::Timeout)?
    }
}

async fn signal_and_wait(connection: &mut PgConnection, backend: &Backend) -> Result<(), SqlError> {
    sqlx::query("SET search_path = pg_catalog")
        .execute(&mut *connection)
        .await?;
    let stopped = sqlx::query_scalar::<_, bool>(TERMINATE_SQL)
        .bind(backend.pid)
        .bind(backend.started_us)
        .fetch_optional(connection)
        .await?;
    match stopped {
        Some(false) => Err(SqlError::Timeout),
        Some(true) | None => Ok(()),
    }
}

pub(super) async fn close(connection: PgConnection) -> Result<(), SqlError> {
    tokio::time::timeout(Duration::from_secs(1), connection.close())
        .await
        .map_err(|_| SqlError::Timeout)?
        .map_err(SqlError::Database)
}
#[cfg(test)]
mod tests;
