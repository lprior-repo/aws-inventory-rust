use std::path::PathBuf;

pub(super) const USAGE: &str = "usage:\n  inventory-harness snapshot freeze --evidence <dir> --output <file> \
--binary <file> [--root <dir>]\n  inventory-harness install <destination> [--root <dir>]\n  \
inventory-harness install-graph <destination> [--jobs N]\n  \
inventory-harness catalog universal --root <dir> [--check]\n  \
inventory-harness probe --catalog <file> --endpoint <https-loopback> \
--ca-bundle <file> --region <name> --report <file> [--service <name>]...\n  \
inventory-harness seed validate --seeds <dir> --catalog <file> [--service <name>]...\n  \
inventory-harness seed run --seeds <dir> --catalog <file> --endpoint <https-loopback> \
--ca-bundle <file> --region <name> [--service <name>]... [--concurrency <1..64>] --oracle <file> --matrix <file>\n  \
inventory-harness seed s3 --bucket <name>\n  \
inventory-harness seed cloud-control --prefix <inventory-page-...> --count <n>\n  \
inventory-harness assert cloud-control --db <file> --scan-id <id> [--seed-results <file>] \
[--root <dir>]\n  \
inventory-harness assert-global --db <file> --scan-id <id> [--seed-results <file>] \
[--root <dir>]\n  \
inventory-harness assert-parents --db <file> --scan-id <id> [--seed-results <file>] \
[--root <dir>]\n  \
inventory-harness assert-regional --db <file> --scan-id <id> [--seed-results <file>] \
[--root <dir>] [--region <region>]\n  \
inventory-harness postgres up [--runtime docker|podman] | down [--volumes] | url | prune";

pub(super) enum Command {
    Freeze {
        evidence: PathBuf,
        output: PathBuf,
        binary: PathBuf,
        root: PathBuf,
    },
    Install {
        destination: PathBuf,
        root: PathBuf,
    },
    InstallGraph {
        destination: PathBuf,
        jobs: usize,
    },
    CatalogUniversal {
        root: PathBuf,
        check: bool,
    },
    AssertCloudControl {
        database: PathBuf,
        scan_id: String,
        seed_results: Option<PathBuf>,
        root: PathBuf,
    },
    AssertGlobal {
        database: PathBuf,
        scan_id: String,
        seed_results: Option<PathBuf>,
        root: PathBuf,
    },
    AssertParents {
        database: PathBuf,
        scan_id: String,
        seed_results: Option<PathBuf>,
        root: PathBuf,
    },
    AssertRegional {
        database: PathBuf,
        scan_id: String,
        seed_results: Option<PathBuf>,
        root: PathBuf,
        region: Option<String>,
    },
    SeedS3 {
        bucket: String,
    },
    SeedCloudControl {
        prefix: String,
        count: usize,
    },
    Mutate {
        emulator: PathBuf,
        binary: PathBuf,
        branch_probe: PathBuf,
        output: PathBuf,
        workspace: PathBuf,
        redis_server: Option<PathBuf>,
        falkor_module: Option<PathBuf>,
        command_deadline: i64,
        command: Vec<String>,
    },
    VerifyDelivery {
        binary: PathBuf,
        redis_cli: PathBuf,
        database: PathBuf,
        output: PathBuf,
    },
    CatalogOperations {
        root: PathBuf,
        emulator_summary: Option<PathBuf>,
        check_additions: bool,
        output: PathBuf,
    },
}
