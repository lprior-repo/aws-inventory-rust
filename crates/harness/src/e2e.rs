pub mod collector;
pub mod command;
pub mod graph;
mod outcome;
mod seed;
pub mod stack;
pub mod verify;
mod workflow;

use crate::{HarnessError, postgres::TestDatabase};
use command::RunArgs;
use stack::Stack;
use std::{path::PathBuf, time::Instant};
use workflow::{Session, Stage};

pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

struct Run {
    args: RunArgs,
    stack: Stack,
    database: TestDatabase,
    baseline: Option<verify::Baseline>,
}

pub async fn execute(args: RunArgs, admin: &str) -> Result<(), HarnessError> {
    prepare_work(&args.work)?;
    let started = Instant::now();
    let session = start(args, admin).await;
    workflow::report("start", started, &session);
    let session = session?;
    workflow::drive(session).await
}

pub fn prepare_work(work: &std::path::Path) -> Result<(), HarnessError> {
    std::fs::create_dir_all(work)?;
    if std::fs::read_dir(work)?.next().is_some() {
        return Err(HarnessError::invalid("--work must be an empty directory"));
    }
    Ok(())
}

async fn start(args: RunArgs, admin: &str) -> Result<Run, HarnessError> {
    let work = args.work.clone();
    let stack = tokio::task::spawn_blocking(move || Stack::start(&work))
        .await
        .map_err(|error| HarnessError::invalid(format!("stack startup task failed: {error}")))??;
    let database = TestDatabase::create(admin).await?;
    Ok(Run {
        args,
        stack,
        database,
        baseline: None,
    })
}

impl Session for Run {
    async fn stage(&mut self, stage: Stage) -> Result<(), HarnessError> {
        match stage {
            Stage::Baseline => self.baseline().await,
            Stage::Seed => seed::run(&self.args, &self.stack).await,
            Stage::Counts => self.verify_counts().await,
            Stage::Sweep | Stage::Resweep => self.sweep(stage).await,
        }
    }

    async fn finish(self, interrupted: bool) -> Result<(), HarnessError> {
        if self.args.keep && !interrupted {
            println!("database {}", self.database.url());
            println!("emulator {}", self.stack.emulator().endpoint());
            tokio::signal::ctrl_c().await?;
            self.database.keep();
            return stop_stack(self.stack).await;
        }
        let children = stop_stack(self.stack).await;
        let database = self.database.finish().await;
        children.and(database)
    }
}

impl Run {
    async fn baseline(&mut self) -> Result<(), HarnessError> {
        self.sweep(Stage::Baseline).await?;
        self.baseline = Some(
            verify::capture_baseline(self.database.url(), &self.args.work.join("baseline.tsv"))
                .await?,
        );
        Ok(())
    }

    async fn verify_counts(&self) -> Result<(), HarnessError> {
        let baseline = self
            .baseline
            .as_ref()
            .ok_or_else(|| HarnessError::invalid("pre-seed baseline missing"))?;
        let coverage = verify::Coverage::load(
            &self.args.catalog,
            &self.args.seeds,
            &seed::selected_recipes(&self.args),
        )?;
        verify::counts(
            self.database.url(),
            &self.args.work.join("oracle.tsv"),
            baseline,
            &coverage,
        )
        .await
    }

    async fn sweep(&self, stage: Stage) -> Result<(), HarnessError> {
        let summary = collector::sweep(
            &self.stack,
            &self.args.collector,
            &self.args.catalog,
            self.database.url(),
            &self.args.services,
            &self.args.work.join(format!("{}.log", stage.name())),
        )
        .await?;
        verify::failed_operations(self.database.url(), &summary).await?;
        if stage == Stage::Resweep {
            verify::zero(&summary, "resource_writes")?;
        }
        Ok(())
    }
}

async fn stop_stack(stack: Stack) -> Result<(), HarnessError> {
    tokio::task::spawn_blocking(move || stack.finish())
        .await
        .map_err(|error| HarnessError::invalid(format!("stack shutdown task failed: {error}")))?
}

#[cfg(test)]
mod tests;
