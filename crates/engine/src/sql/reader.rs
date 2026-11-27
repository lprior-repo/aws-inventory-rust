use super::SqlError;
use sqlx::{Connection, PgConnection, postgres::PgConnectOptions};
use std::{str::FromStr, time::Duration};

const READER_CHECK: &str = r"
SELECT (EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'inventory')
AND has_schema_privilege(current_user, 'inventory', 'USAGE')
AND NOT EXISTS (
    SELECT 1 FROM pg_roles r
    WHERE (r.rolname = current_user OR r.rolname = session_user
        OR pg_has_role(current_user, r.oid, 'MEMBER')
        OR pg_has_role(session_user, r.oid, 'MEMBER'))
    AND (r.rolsuper OR r.rolcreaterole OR r.rolcreatedb OR r.rolreplication OR r.rolbypassrls
        OR r.rolname IN ('pg_write_all_data', 'pg_signal_backend',
            'pg_execute_server_program', 'pg_write_server_files', 'pg_read_all_data',
            'pg_read_server_files', 'pg_monitor', 'pg_read_all_stats')
        OR has_schema_privilege(r.oid, 'inventory', 'CREATE')
        OR EXISTS (SELECT 1 FROM pg_database d WHERE d.datname = current_database()
            AND pg_has_role(r.oid, d.datdba, 'USAGE'))
        OR EXISTS (SELECT 1 FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'inventory' AND c.relkind IN ('r','p','v','m','f')
            AND (has_table_privilege(r.oid, c.oid, 'INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER')
                OR has_any_column_privilege(r.oid, c.oid, 'INSERT,UPDATE,REFERENCES')
                OR pg_has_role(r.oid, c.relowner, 'USAGE')))
        OR EXISTS (SELECT 1 FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'inventory'
            AND CASE WHEN c.relkind = 'S'
                THEN has_sequence_privilege(r.oid, c.oid, 'USAGE,UPDATE') ELSE false END))
)), pg_backend_pid()";

#[derive(Clone)]
pub struct ReaderUrl {
    options: PgConnectOptions,
    #[cfg(test)]
    pub(super) server_millis: Option<u64>,
}

impl ReaderUrl {
    #[tracing::instrument(skip_all)]
    pub async fn establish(url: &str) -> Result<Self, SqlError> {
        if url.len() > 8_192
            || !(url.starts_with("postgres://") || url.starts_with("postgresql://"))
        {
            return Err(SqlError::InvalidUrl);
        }
        let options = PgConnectOptions::from_str(url).map_err(|_| SqlError::InvalidUrl)?;
        let reader = Self {
            options,
            #[cfg(test)]
            server_millis: None,
        };
        let (connection, _) = reader.connect().await?;
        close(connection).await?;
        Ok(reader)
    }

    #[tracing::instrument(skip_all)]
    pub(super) async fn connect(&self) -> Result<(PgConnection, i32), SqlError> {
        tokio::time::timeout(Duration::from_secs(5), self.connect_checked())
            .await
            .map_err(|_| SqlError::Timeout)?
    }

    async fn connect_checked(&self) -> Result<(PgConnection, i32), SqlError> {
        let mut connection = PgConnection::connect_with(&self.options).await?;
        if !crate::postgres_version::supported(connection.server_version_num()) {
            return Err(SqlError::UnsupportedServerVersion);
        }
        sqlx::query("SET search_path = pg_catalog")
            .execute(&mut connection)
            .await?;
        let (reader, pid) = sqlx::query_as::<_, (bool, i32)>(READER_CHECK)
            .fetch_one(&mut connection)
            .await?;
        if !reader {
            return Err(SqlError::ReaderRequired);
        }
        let standard = sqlx::query_scalar::<_, bool>(
            "SELECT current_setting('standard_conforming_strings') = 'on'",
        )
        .fetch_one(&mut connection)
        .await?;
        if !standard {
            return Err(SqlError::SettingRequired {
                name: "standard_conforming_strings",
                required: "on",
            });
        }
        Ok((connection, pid))
    }

    pub(super) async fn terminate(&self, pid: i32) -> Result<(), SqlError> {
        tokio::time::timeout(Duration::from_secs(5), async {
            let mut connection = PgConnection::connect_with(&self.options).await?;
            sqlx::query("SET search_path = pg_catalog")
                .execute(&mut connection)
                .await?;
            let result = sqlx::query_scalar::<_, bool>(
                "SELECT CASE WHEN pg_terminate_backend($1, 1000) THEN true ELSE NOT EXISTS(SELECT 1 FROM pg_stat_activity WHERE pid=$1) END",
            )
            .bind(pid)
            .fetch_one(&mut connection)
            .await;
            let closed = close(connection).await;
            let stopped = result?;
            closed?;
            if !stopped {
                return Err(SqlError::Database(sqlx::Error::Protocol(
                    "PostgreSQL did not stop the timed-out reader backend".into(),
                )));
            }
            Ok(())
        })
        .await
        .map_err(|_| SqlError::Timeout)?
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
