use super::*;

#[derive(Subcommand, Debug)]
pub enum JobCommand {
    /// Persist a scan specification without loading credentials or making AWS calls.
    Submit {
        #[arg(long, default_value = "jobs.sqlite")]
        queue: PathBuf,
        #[arg(long)]
        job_id: Option<String>,
        #[arg(long, default_value_t = 3)]
        max_job_attempts: u32,
        #[arg(long, default_value_t = 0)]
        delay_seconds: u64,
        #[command(flatten)]
        scan: Box<ScanArgs>,
    },
    /// Claim and run up to this many available jobs, then exit; expired leases are recovered.
    Run {
        #[arg(long, default_value = "jobs.sqlite")]
        queue: PathBuf,
        #[arg(long, default_value_t = 1)]
        limit: usize,
        #[arg(long, default_value_t = 60)]
        lease_seconds: u64,
        #[arg(long)]
        worker: Option<String>,
    },
    List {
        #[arg(long, default_value = "jobs.sqlite")]
        queue: PathBuf,
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    Show {
        #[arg(long, default_value = "jobs.sqlite")]
        queue: PathBuf,
        #[arg(long)]
        job_id: String,
    },
    Cancel {
        #[arg(long, default_value = "jobs.sqlite")]
        queue: PathBuf,
        #[arg(long)]
        job_id: String,
    },
    /// Explicitly requeue a terminal job with a bounded number of additional attempts.
    Retry {
        #[arg(long, default_value = "jobs.sqlite")]
        queue: PathBuf,
        #[arg(long)]
        job_id: String,
        #[arg(long, default_value_t = 1)]
        additional_attempts: u32,
    },
}
