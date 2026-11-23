use super::*;
use sqlx::{Connection, postgres::PgConnection};
use std::{
    io::Write,
    path::Path,
    process::{Child, Command, Stdio},
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

struct StubChild(Child);
impl StubChild {
    fn start() -> Result<Self, HarnessError> {
        Ok(Self(
            Command::new("sh")
                .args(["-c", "exec sleep 300"])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?,
        ))
    }
    fn stop(&mut self) -> Result<(), HarnessError> {
        if self.0.try_wait()?.is_none() {
            self.0.kill()?;
        }
        self.0.wait()?;
        Ok(())
    }
}
impl Drop for StubChild {
    fn drop(&mut self) {
        if let Err(error) = self.stop() {
            eprintln!("stub cleanup failed: {error}");
        }
    }
}

struct StubSession {
    root: PathBuf,
    database: TestDatabase,
    children: [StubChild; 2],
    fail: bool,
    baseline: Option<verify::Baseline>,
}

impl Session for StubSession {
    async fn stage(&mut self, stage: Stage) -> Result<(), HarnessError> {
        event(&self.root, stage.name())?;
        match stage {
            Stage::Baseline => self.baseline().await,
            Stage::Seed => seed_database(self.database.url(), &self.root).await,
            Stage::Counts => self.counts().await,
            Stage::Sweep | Stage::Resweep => stub_sweep(&self.root, stage, self.fail).await,
        }
    }
    async fn finish(mut self, _interrupted: bool) -> Result<(), HarnessError> {
        event(&self.root, "stop")?;
        self.children.iter_mut().try_for_each(StubChild::stop)?;
        self.database.finish().await
    }
}

impl StubSession {
    async fn baseline(&mut self) -> Result<(), HarnessError> {
        let mut connection = PgConnection::connect(self.database.url()).await?;
        sqlx::raw_sql("CREATE SCHEMA inventory; CREATE TABLE inventory.resources(kind text,account text,region text,id text); INSERT INTO inventory.resources VALUES('AWS::EC2::VPC','a','r','implicit')")
            .execute(&mut connection).await?;
        connection.close().await?;
        self.baseline = Some(
            verify::capture_baseline(self.database.url(), &self.root.join("baseline.tsv")).await?,
        );
        Ok(())
    }

    async fn counts(&self) -> Result<(), HarnessError> {
        let coverage = verify::Coverage::load(
            &workspace_root().join("catalog/aws-api.catalog"),
            &workspace_root().join("crates/harness/seeds"),
            &["ec2".into()],
        )?;
        let baseline = self
            .baseline
            .as_ref()
            .ok_or_else(|| HarnessError::invalid("test baseline missing"))?;
        verify::counts(
            self.database.url(),
            &self.root.join("oracle.tsv"),
            baseline,
            &coverage,
        )
        .await
    }
}

async fn stub_sweep(root: &Path, stage: Stage, fail: bool) -> Result<(), HarnessError> {
    let mut command = tokio::process::Command::new("sh");
    let code = if fail { "9" } else { "0" };
    command.args(["-c", "printf 'resource_writes_scope=invocation resource_writes=0 rows=1 complete=1 partial=0 unsupported=0 failed=0 work_remaining=0 graph_pending=0\\n'; exit \"$1\"", "stub", code]);
    let summary = collector::execute(command, &root.join(format!("{}.log", stage.name()))).await?;
    verify::zero(&summary, "resource_writes")
}

fn event(root: &Path, value: &str) -> Result<(), HarnessError> {
    writeln!(
        std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(root.join("events"))?,
        "{value}"
    )?;
    Ok(())
}

async fn seed_database(database: &str, root: &Path) -> Result<(), HarnessError> {
    let mut connection = PgConnection::connect(database).await?;
    sqlx::query("INSERT INTO inventory.resources VALUES('AWS::EC2::VPC','a','r','seeded')")
        .execute(&mut connection)
        .await?;
    connection.close().await?;
    std::fs::write(
        root.join("oracle.tsv"),
        "resource\tAWS::EC2::VPC\ta\tr\tseeded\n",
    )?;
    Ok(())
}

async fn workflow_case(fail: bool) -> TestResult {
    let admin = crate::postgres::admin_url()?;
    let root = tempfile::tempdir()?;
    let database = TestDatabase::create(&admin).await?;
    let name = database.name().to_owned();
    let children = [StubChild::start()?, StubChild::start()?];
    let pids = children.each_ref().map(|child| child.0.id());
    let result = workflow::drive(StubSession {
        root: root.path().into(),
        database,
        children,
        fail,
        baseline: None,
    })
    .await;
    assert_eq!(result.is_err(), fail);
    assert_events(root.path(), fail)?;
    pids.iter()
        .for_each(|pid| assert!(!PathBuf::from(format!("/proc/{pid}")).exists()));
    assert_database_dropped(&admin, &name).await
}

fn assert_events(root: &Path, fail: bool) -> TestResult {
    let expected = if fail {
        "baseline\nseed\nsweep\nstop\n"
    } else {
        "baseline\nseed\nsweep\nverify-counts\nresweep\nstop\n"
    };
    assert_eq!(std::fs::read_to_string(root.join("events"))?, expected);
    Ok(())
}

async fn assert_database_dropped(admin: &str, name: &str) -> TestResult {
    let mut connection = PgConnection::connect(admin).await?;
    let present: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname=$1)")
            .bind(name)
            .fetch_one(&mut connection)
            .await?;
    connection.close().await?;
    assert!(!present);
    Ok(())
}

#[tokio::test]
#[ignore = "requires an owned PostgreSQL server via INVENTORY_POSTGRES_URL"]
async fn given_stub_collectors_when_all_stages_pass_then_order_and_process_database_cleanup_hold()
-> TestResult {
    workflow_case(false).await
}

#[tokio::test]
#[ignore = "requires an owned PostgreSQL server via INVENTORY_POSTGRES_URL"]
async fn given_failed_collector_when_run_then_later_stages_stop_and_owned_resources_are_removed()
-> TestResult {
    workflow_case(true).await
}

#[test]
fn given_nonempty_work_when_prepared_then_existing_contents_are_untouched() -> TestResult {
    let root = tempfile::tempdir()?;
    std::fs::write(root.path().join("user-file"), "untouched")?;
    assert!(prepare_work(root.path()).is_err());
    assert_eq!(
        std::fs::read_to_string(root.path().join("user-file"))?,
        "untouched"
    );
    Ok(())
}

#[tokio::test]
#[ignore = "requires an owned PostgreSQL server via INVENTORY_POSTGRES_URL"]
async fn given_explicit_keep_when_database_owner_is_consumed_then_database_remains_available()
-> TestResult {
    let admin = crate::postgres::admin_url()?;
    let database = TestDatabase::create(&admin).await?;
    let name = database.name().to_owned();
    let url = database.url().to_owned();
    database.keep();
    let connection = PgConnection::connect(&url).await?;
    connection.close().await?;
    let mut admin_connection = PgConnection::connect(&admin).await?;
    sqlx::QueryBuilder::<sqlx::Postgres>::new("DROP DATABASE \"")
        .push(&name)
        .push("\" WITH (FORCE)")
        .build()
        .execute(&mut admin_connection)
        .await?;
    admin_connection.close().await?;
    assert_database_dropped(&admin, &name).await
}
