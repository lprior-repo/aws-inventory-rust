use super::*;
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[tokio::test]
async fn given_nonzero_tool_when_executed_then_exit_is_a_failure() -> TestResult {
    let root = tempfile::tempdir()?;
    let mut command = Command::new("sh");
    command.args([
        "-c",
        "printf 'not a result'; printf 'endpoint rejected' >&2; exit 7",
    ]);
    let error = execute_process(command, root.path(), "collect")
        .await
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "collect exited exit status: 7; see collect.stderr"
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("collect.stderr"))?,
        "endpoint rejected"
    );
    Ok(())
}

#[tokio::test]
async fn given_valid_counts_stdout_when_command_exits_nonzero_then_no_counts_are_returned()
-> TestResult {
    let root = tempfile::tempdir()?;
    let path = root.path().join("counts-fails.toml");
    std::fs::write(
        &path,
        r#"services = ['s3']
version = ['true']
reset = ['true']
run = ['true']
counts = ['sh', '-c', 'printf "vpc\t1\n"; exit 7']
"#,
    )?;
    let definition = Definition::load("other", &path)?;

    let outcome = run_external(&definition, &BTreeMap::new(), root.path()).await;

    assert_eq!(
        std::fs::read_to_string(root.path().join("counts.stdout"))?,
        "vpc\t1\n"
    );
    assert!(matches!(outcome, Err(HarnessError::Invalid(_))));
    Ok(())
}

#[test]
fn given_runtime_without_io_driver_when_tool_runs_then_owned_child_is_collected() -> TestResult {
    let root = tempfile::tempdir()?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()?;
    let mut command = Command::new("sh");
    command.args(["-c", "printf complete"]);

    let output = runtime.block_on(execute_process(command, root.path(), "collect"))?;

    assert_eq!(output.stdout, "complete");
    assert!(!output.left_processes);
    Ok(())
}

#[tokio::test]
async fn given_exit_two_when_executed_then_it_is_not_accepted_as_timed_success() -> TestResult {
    let root = tempfile::tempdir()?;
    let mut command = Command::new("sh");
    command.args(["-c", "exit 2"]);
    let error = execute_process(command, root.path(), "collect")
        .await
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "collect exited exit status: 2; see collect.stderr"
    );
    Ok(())
}

#[tokio::test]
async fn given_successful_count_command_when_executed_then_exact_counts_are_read() -> TestResult {
    let root = tempfile::tempdir()?;
    let mut command = Command::new("sh");
    command.args(["-c", "printf 'AWS::EC2::VPC\t6\nAWS::S3::Bucket\t2\n'"]);
    let text = execute_process(command, root.path(), "counts").await?;
    assert_eq!(
        parse_counts(&text.stdout)?,
        Counts::from([("AWS::EC2::VPC".into(), 6), ("AWS::S3::Bucket".into(), 2)])
    );
    assert!(!text.left_processes);
    Ok(())
}

#[test]
fn given_duplicate_or_invalid_counts_when_parsing_then_exact_error_is_returned() -> TestResult {
    [
        ("vpc\t1\nvpc\t2\n", "empty or duplicate count kind"),
        ("\t1\n", "empty or duplicate count kind"),
        ("vpc 1", "count rows must be kind<TAB>unsigned-count"),
        ("vpc\t-1", "count rows must be kind<TAB>unsigned-count"),
        (
            "vpc\t18446744073709551616",
            "count rows must be kind<TAB>unsigned-count",
        ),
    ]
    .into_iter()
    .try_for_each(|(text, message)| -> TestResult {
        assert_eq!(
            parse_counts(text)
                .err()
                .ok_or("expected an error")?
                .to_string(),
            message
        );
        Ok(())
    })?;
    assert_eq!(parse_counts("vpc\t0")?, Counts::from([("vpc".into(), 0)]));
    assert_eq!(parse_counts("")?, Counts::new());
    Ok(())
}

#[test]
fn given_external_definition_when_loaded_then_commands_and_environment_are_preserved() -> TestResult
{
    let root = tempfile::tempdir()?;
    let path = root.path().join("tool.toml");
    std::fs::write(
        &path,
        "services = ['ec2', 's3']\nrun = ['tool', 'collect']\nreset = ['tool', 'reset']\ncounts = ['tool', 'counts']\nversion = ['tool', '--version']\napi_calls = ['tool', 'calls']\n[environment]\nAWS_ENDPOINT_URL = '${H2H_ENDPOINT}'\n",
    )?;
    let definition = Definition::load("competitor", &path)?;
    assert_eq!(definition.name, "competitor");
    assert_eq!(definition.version, vec!["tool", "--version"]);
    assert_eq!(definition.services, Some(vec!["ec2".into(), "s3".into()]));
    assert_eq!(
        definition
            .environment
            .get("AWS_ENDPOINT_URL")
            .map(String::as_str),
        Some("${H2H_ENDPOINT}")
    );
    let Tool::External {
        run,
        reset,
        counts,
        api_calls,
    } = definition.tool
    else {
        return Err("expected external definition".into());
    };
    assert_eq!(
        (run, reset, counts, api_calls),
        (
            vec!["tool".into(), "collect".into()],
            vec!["tool".into(), "reset".into()],
            vec!["tool".into(), "counts".into()],
            Some(vec!["tool".into(), "calls".into()])
        )
    );
    Ok(())
}

#[test]
fn given_invalid_tool_name_when_loaded_then_no_path_can_escape_work_directory() -> TestResult {
    ["../escape", "ours", "", "two words"]
        .into_iter()
        .try_for_each(|name| -> TestResult {
            let error = Definition::load(name, Path::new("absent.toml"))
                .err()
                .ok_or("expected an error")?;
            assert_eq!(
                error.to_string(),
                "tool name must be 1..64 ASCII letters, digits, '-' or '_'; ours is reserved"
            );
            Ok(())
        })
}

#[test]
fn given_summary_when_reporting_api_calls_then_attempts_include_retries() -> TestResult {
    assert_eq!(
        reported_calls("metrics api_attempts=42 api_retries=3"),
        Some(42)
    );
    assert_eq!(reported_calls("no metrics"), None);
    assert_eq!(
        expand(
            "${H2H_ENDPOINT}/aws",
            &BTreeMap::from([("H2H_ENDPOINT".into(), "https://localhost:123".into())])
        ),
        "https://localhost:123/aws"
    );
    Ok(())
}

#[test]
fn given_collector_environment_when_loading_then_even_empty_table_is_rejected() -> TestResult {
    let root = tempfile::tempdir()?;
    let path = root.path().join("collector.toml");
    let binary = toml::Value::String(std::env::current_exe()?.to_string_lossy().into_owned());
    std::fs::write(
        &path,
        format!("collector = {binary}\nversion = ['true']\n[environment]\n"),
    )?;
    assert_eq!(
        Definition::load("collector", &path)
            .err()
            .ok_or("expected an error")?
            .to_string(),
        format!(
            "tool collector definition {}: collector definitions cannot declare environment",
            path.display()
        )
    );
    Ok(())
}

#[tokio::test]
async fn given_per_tool_environment_when_executed_then_values_expand_without_inheriting_home()
-> TestResult {
    let root = tempfile::tempdir()?;
    let variables = BTreeMap::from([
        ("H2H_ENDPOINT".into(), "https://127.0.0.1:12345".into()),
        ("H2H_CA_BUNDLE".into(), "/private/ca.pem".into()),
        ("H2H_WORK".into(), "/private/work".into()),
        ("H2H_CATALOG".into(), "/private/catalog".into()),
        ("H2H_SERVICES".into(), "ec2,s3".into()),
        (
            "AWS_SHARED_CREDENTIALS_FILE".into(),
            "/private/credentials".into(),
        ),
    ]);
    let argv = ["sh", "-c", r#"printf '%s\n' "$AWS_ENDPOINT_URL" "$AWS_CA_BUNDLE" "$AWS_SHARED_CREDENTIALS_FILE" "$H2H_SERVICES" "$TOOL_OUTPUT" "${HOME-unset}" "$1""#,
        "adapter", "${H2H_CATALOG}"].map(str::to_owned);
    for name in ["first", "second"] {
        let environment = BTreeMap::from([
            ("AWS_ENDPOINT_URL".into(), "${H2H_ENDPOINT}".into()),
            ("AWS_CA_BUNDLE".into(), "${H2H_CA_BUNDLE}".into()),
            ("TOOL_OUTPUT".into(), format!("${{H2H_WORK}}/{name}")),
        ]);
        let command = external_command(&argv, &variables, &environment)?;
        let output = execute_process(command, root.path(), name).await?;
        assert_eq!(
            output.stdout,
            format!(
                "https://127.0.0.1:12345\n/private/ca.pem\n/private/credentials\nec2,s3\n/private/work/{name}\nunset\n/private/catalog\n"
            )
        );
    }
    Ok(())
}

#[tokio::test]
async fn given_postgres_that_never_responds_when_counting_then_deadline_ends_connection()
-> TestResult {
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
    let address = listener.local_addr()?;
    let error = tokio::time::timeout(
        Duration::from_secs(5),
        read_counts(
            &format!("postgres://inv@{address}/postgres"),
            Duration::from_millis(20),
        ),
    )
    .await?
    .err()
    .ok_or("expected an error")?;
    assert_eq!(error.to_string(), "PostgreSQL counts exceeded 20ms");
    Ok(())
}

#[tokio::test]
async fn given_shell_with_descendant_when_timed_out_then_group_is_stopped_and_leader_reaped()
-> TestResult {
    tokio::time::timeout(Duration::from_secs(35), async {
        let (child, descendant) = spawn_waiting_group().await?;
        let leader = child.pid();
        let result = wait_process(child, "collect", Duration::ZERO).await;
        let error = result.err().ok_or("expected an error")?;
        assert_eq!(
            error.to_string(),
            "collect exceeded 0ns; process group stopped"
        );
        assert_reaped(leader);
        assert!(descendant_stopped(descendant).await?);
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await??;
    Ok(())
}

#[test]
fn given_timeout_when_termination_also_fails_then_both_causes_are_reported() {
    let error = timeout_error(
        "collect",
        Duration::from_secs(3),
        Err(HarnessError::invalid("kill refused")),
    );
    assert_eq!(
        error.to_string(),
        "collect exceeded 3s; process group termination failed: kill refused"
    );
}

async fn spawn_waiting_group() -> Result<(ChildOwner, i32), Box<dyn std::error::Error>> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut child = ChildOwner::spawn(
        std::process::Command::new("sh")
            .args(["-c", "sleep 60 & echo $!; wait"])
            .stdout(Stdio::piped()),
    )?;
    let stdout = child.child_mut().stdout.take().ok_or("missing stdout")?;
    let mut line = String::new();
    BufReader::new(async_stdout(stdout))
        .read_line(&mut line)
        .await?;
    Ok((child, line.trim().parse()?))
}

fn async_stdout(stdout: std::process::ChildStdout) -> tokio::fs::File {
    let fd: std::os::fd::OwnedFd = stdout.into();
    tokio::fs::File::from_std(fd.into())
}

fn assert_reaped(leader: rustix::process::Pid) {
    use rustix::process::{WaitId, WaitIdOptions};
    let flags = WaitIdOptions::EXITED | WaitIdOptions::NOHANG | WaitIdOptions::NOWAIT;
    assert!(matches!(
        rustix::process::waitid(WaitId::Pid(leader), flags),
        Err(rustix::io::Errno::CHILD)
    ));
}

#[tokio::test]
async fn given_lost_wait_ownership_when_cleaning_up_then_error_is_returned_without_signalling()
-> TestResult {
    use std::io::Write;
    use tokio::io::AsyncReadExt;

    tokio::time::timeout(Duration::from_secs(10), async {
        let mut child = ChildOwner::spawn(
            std::process::Command::new("sh")
                .args([
                    "-c",
                    r#"exec 3<&0
sh -c 'IFS= read -r line && printf "%s\n" "$line"' <&3 &
exit 0"#,
                ])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped()),
        )?;
        let mut input = child.child_mut().stdin.take().ok_or("missing stdin")?;
        let mut output = async_stdout(child.child_mut().stdout.take().ok_or("missing stdout")?);
        wait_exited(&child).await?;
        let status =
            rustix::process::waitpid(Some(child.pid()), rustix::process::WaitOptions::NOHANG)?;
        assert!(matches!(status, Some((_, exit)) if exit.exit_status() == Some(0)));

        let result = wait_process(child, "collect", Duration::from_secs(5)).await;

        assert!(matches!(result, Err(HarnessError::Io(error))
            if error.raw_os_error() == Some(rustix::io::Errno::CHILD.raw_os_error())));
        input.write_all(b"worker still responsive\n")?;
        drop(input);
        let mut reply = String::new();
        output.read_to_string(&mut reply).await?;
        assert_eq!(reply, "worker still responsive\n");
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await??;
    Ok(())
}

pub(super) async fn descendant_stopped(pid: i32) -> Result<bool, std::io::Error> {
    let deadline = tokio::time::Instant::now()
        .checked_add(Duration::from_secs(30))
        .ok_or_else(|| std::io::Error::other("invalid process polling deadline"))?;
    let polls = futures_util::stream::iter(0..3001)
        .then(|_| async {
            if tokio::time::Instant::now() >= deadline {
                return Ok(Some(false));
            }
            match std::fs::read_to_string(format!("/proc/{pid}/stat")) {
                Ok(stat)
                    if stat
                        .rsplit_once(") ")
                        .and_then(|(_, tail)| tail.chars().next())
                        == Some('Z') =>
                {
                    Ok(Some(true))
                }
                Ok(_) => {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    Ok(None)
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Some(true)),
                Err(error) => Err(error),
            }
        })
        .filter_map(|result| std::future::ready(result.transpose()));
    use futures_util::StreamExt;
    tokio::pin!(polls);
    polls
        .next()
        .await
        .transpose()
        .map(|stopped| stopped == Some(true))
}

#[test]
fn given_collector_services_when_loading_then_even_empty_scope_is_rejected() -> TestResult {
    let root = tempfile::tempdir()?;
    let path = root.path().join("collector.toml");
    let binary = toml::Value::String(std::env::current_exe()?.to_string_lossy().into_owned());
    for services in ["[]", "['ec2']"] {
        std::fs::write(
            &path,
            format!("collector = {binary}\nversion = ['true']\nservices = {services}\n"),
        )?;
        assert_eq!(
            Definition::load("collector", &path)
                .err()
                .ok_or("expected an error")?
                .to_string(),
            format!(
                "tool collector definition {}: collector definitions cannot declare services",
                path.display()
            )
        );
    }
    Ok(())
}

#[test]
fn given_missing_external_reset_or_services_when_loading_then_definition_is_rejected() -> TestResult
{
    let root = tempfile::tempdir()?;
    let path = root.path().join("external.toml");
    for (extra, expected) in [
        ("services = ['ec2']", "tool reset must be a command array"),
        ("reset = ['true']", "tool services must be a command array"),
    ] {
        std::fs::write(
            &path,
            format!("run = ['true']\ncounts = ['true']\nversion = ['true']\n{extra}\n"),
        )?;
        assert_eq!(
            Definition::load("external", &path)
                .err()
                .ok_or("expected an error")?
                .to_string(),
            format!("tool external definition {}: {expected}", path.display())
        );
    }
    Ok(())
}

#[test]
fn given_command_argument_bounds_when_parsing_then_only_up_to_two_hundred_fifty_six_are_accepted()
-> TestResult {
    let values = vec![toml::Value::String("arg".into()); 256];
    let mut table = toml::Table::from_iter([("run".into(), toml::Value::Array(values))]);
    assert_eq!(command(&table, "run")?, vec!["arg"; 256]);
    table
        .get_mut("run")
        .and_then(toml::Value::as_array_mut)
        .ok_or("missing arguments")?
        .push(toml::Value::String("extra".into()));
    assert_eq!(
        command(&table, "run")
            .err()
            .ok_or("expected an error")?
            .to_string(),
        "tool command must contain 1..256 arguments"
    );
    Ok(())
}

#[test]
fn given_count_kind_bounds_when_parsing_then_only_ten_thousand_kinds_are_accepted() -> TestResult {
    let expected: Counts = (0..10000)
        .map(|index| (format!("kind{index}"), 1))
        .collect();
    let mut text: String = expected.keys().map(|kind| format!("{kind}\t1\n")).collect();
    assert_eq!(parse_counts(&text)?, expected);
    text.push_str("extra\t1\n");
    assert_eq!(
        parse_counts(&text)
            .err()
            .ok_or("expected an error")?
            .to_string(),
        "counts exceed 10000 kinds"
    );
    Ok(())
}

#[tokio::test]
async fn given_failed_reset_when_executing_external_stages_then_collection_is_not_started()
-> TestResult {
    let root = tempfile::tempdir()?;
    let path = root.path().join("external.toml");
    std::fs::write(
        &path,
        "services = ['ec2']\nversion = ['true']\nreset = ['sh', '-c', 'exit 1']\nrun = ['sh', '-c', 'touch collected']\ncounts = ['true']\n",
    )?;
    let definition = Definition::load("external", &path)?;
    let error = run_external(&definition, &BTreeMap::new(), root.path())
        .await
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "reset exited exit status: 1; see reset.stderr"
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("reset.stderr"))?,
        ""
    );
    for name in ["collected", "collect.stdout", "counts.stdout"] {
        assert_eq!(
            std::fs::metadata(root.path().join(name))
                .err()
                .ok_or("expected missing collection output")?
                .kind(),
            std::io::ErrorKind::NotFound
        );
    }
    Ok(())
}

#[tokio::test]
async fn given_tool_leaves_background_writer_when_reported_then_cleanup_is_flagged_and_timed()
-> TestResult {
    tokio::time::timeout(Duration::from_secs(35), async {
        let root = tempfile::tempdir()?;
        let path = root.path().join("external.toml");
        std::fs::write(&path, "services = ['ec2']\nversion = ['true']\nreset = ['true']\nrun = ['sh', '-c', '(while :; do printf x >> writes; sleep 0.01; done) & echo $! > writer.pid']\ncounts = ['sh', '-c', 'printf \"vpc\\t1\\n\"']\n")?;
        let definition = Definition::load("external", &path)?;
        let run = run_external(&definition, &BTreeMap::new(), root.path()).await?;
        assert_eq!(run.counts, Counts::from([("vpc".into(), 1)]));
        assert_eq!(run.api_calls, None);
        assert!(run.left_processes);
        let seconds = run.elapsed.as_secs_f64();
        let samples = [super::super::report::Sample { tool: "external".into(), run: 0, outcome: run.into() }];
        let report = super::super::report::build(&["external".into()], &samples, &super::super::report::Evidence {
            versions: serde_json::json!({}), dataset: serde_json::json!({}), machine: serde_json::json!({}),
            seeded_counts: Counts::new(),
            services: vec!["ec2".into()],
            scopes: BTreeMap::from([("external".into(), vec!["ec2".into()])]),
        });
        assert_eq!(report.pointer("/tools/0/runs/0/left_processes_running"), Some(&serde_json::json!(true)));
        assert_eq!(report.pointer("/tools/0/runs/0/process_cleanup"), Some(&serde_json::json!("left processes running after exit; killed")));
        assert_eq!(report.pointer("/tools/0/runs/0/wall_seconds"), Some(&serde_json::json!(seconds)));
        assert_eq!(report.pointer("/tools/0/comparison_timing/median_seconds"), Some(&serde_json::json!(seconds)));
        let pid = std::fs::read_to_string(root.path().join("writer.pid"))?.trim().parse()?;
        assert!(descendant_stopped(pid).await?, "background writer remained alive");
        Ok::<(), Box<dyn std::error::Error>>(())
    }).await??;
    Ok(())
}

#[test]
fn given_stack_values_when_assembling_commands_then_all_placeholders_and_aws_values_match()
-> TestResult {
    let configured = BTreeMap::from([
        (
            "AWS_SHARED_CREDENTIALS_FILE".into(),
            "/stack/credentials".into(),
        ),
        ("AWS_CONFIG_FILE".into(), "/stack/config".into()),
        ("AWS_PROFILE".into(), "default".into()),
        ("AWS_EC2_METADATA_DISABLED".into(), "true".into()),
    ]);
    let actual = command_variables(
        configured,
        "https://127.0.0.1:12345",
        Path::new("/stack/ca.pem"),
        Path::new("/catalog/aws-api.catalog"),
        &["ec2".into(), "s3".into()],
        Path::new("/work/run-0"),
    );
    let expected = [
        ("H2H_ENDPOINT", "https://127.0.0.1:12345"),
        ("H2H_CA_BUNDLE", "/stack/ca.pem"),
        ("H2H_REGION", "us-east-1"),
        ("H2H_SERVICES", "ec2,s3"),
        ("H2H_WORK", "/work/run-0"),
        ("H2H_CATALOG", "/catalog/aws-api.catalog"),
        ("AWS_ENDPOINT_URL", "https://127.0.0.1:12345"),
        ("AWS_CA_BUNDLE", "/stack/ca.pem"),
        ("AWS_REGION", "us-east-1"),
        ("AWS_DEFAULT_REGION", "us-east-1"),
        ("AWS_SHARED_CREDENTIALS_FILE", "/stack/credentials"),
        ("AWS_CONFIG_FILE", "/stack/config"),
        ("AWS_PROFILE", "default"),
        ("AWS_EC2_METADATA_DISABLED", "true"),
    ]
    .into_iter()
    .map(|(key, value)| (key.into(), value.into()))
    .collect();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn given_collector_definition_when_assembling_variables_then_binary_placeholder_is_supplied()
-> TestResult {
    let definition = Definition::ours(PathBuf::from("/collector/aws-inventory"));
    let actual = definition_variables(
        &definition,
        BTreeMap::from([("H2H_WORK".into(), "/work".into())]),
    );
    assert_eq!(
        actual,
        BTreeMap::from([
            ("H2H_WORK".into(), "/work".into()),
            ("H2H_COLLECTOR".into(), "/collector/aws-inventory".into()),
        ])
    );
    Ok(())
}

#[test]
fn given_empty_run_array_when_loading_then_command_is_rejected() -> TestResult {
    let root = tempfile::tempdir()?;
    let path = root.path().join("external.toml");
    std::fs::write(
        &path,
        "services = ['s3']\nversion = ['true']\nreset = ['true']\nrun = []\ncounts = ['true']\n",
    )?;
    assert_eq!(
        Definition::load("external", &path)
            .err()
            .ok_or("expected an error")?
            .to_string(),
        format!(
            "tool external definition {}: tool command must contain 1..256 arguments",
            path.display()
        )
    );
    Ok(())
}

#[test]
fn given_postgres_count_rows_when_bounding_then_ten_thousand_is_the_exact_limit() -> TestResult {
    let rows: Vec<_> = (0..10000).collect();
    assert_eq!(bounded_rows(rows.clone())?, rows);
    assert_eq!(
        bounded_rows((0..10001).collect::<Vec<_>>())
            .err()
            .ok_or("expected an error")?
            .to_string(),
        "counts exceed 10000 kinds"
    );
    Ok(())
}

#[tokio::test]
async fn given_api_calls_command_prints_forty_two_when_running_external_then_calls_are_recorded()
-> TestResult {
    let root = tempfile::tempdir()?;
    let path = root.path().join("external.toml");
    std::fs::write(
        &path,
        "services = ['s3']\nversion = ['true']\nreset = ['true']\nrun = ['true']\ncounts = ['sh', '-c', 'printf \"bucket\\t2\\n\"']\napi_calls = ['sh', '-c', 'printf 42']\n",
    )?;
    let definition = Definition::load("external", &path)?;
    let run = run_external(&definition, &BTreeMap::new(), root.path()).await?;
    assert_eq!(run.api_calls, Some(42));
    assert_eq!(run.counts, Counts::from([("bucket".into(), 2)]));
    assert!(!run.left_processes);
    Ok(())
}

#[tokio::test]
async fn given_api_calls_command_prints_x_when_running_external_then_exact_error_is_reported()
-> TestResult {
    let root = tempfile::tempdir()?;
    let path = root.path().join("external.toml");
    std::fs::write(
        &path,
        "services = ['s3']\nversion = ['true']\nreset = ['true']\nrun = ['true']\ncounts = ['true']\napi_calls = ['sh', '-c', 'printf x']\n",
    )?;
    let definition = Definition::load("external", &path)?;
    assert_eq!(
        run_external(&definition, &BTreeMap::new(), root.path())
            .await
            .err()
            .ok_or("expected an error")?
            .to_string(),
        "API calls command must return one unsigned integer"
    );
    Ok(())
}

#[tokio::test]
async fn given_early_failure_after_spawning_group_when_guard_drops_then_every_process_is_stopped()
-> TestResult {
    tokio::time::timeout(Duration::from_secs(35), async {
        let (child, descendant) = spawn_waiting_group().await?;
        let leader = child.pid();
        let result = fail_with_group(child);
        assert_eq!(
            result.err().ok_or("expected an error")?.to_string(),
            "early failure"
        );
        assert_reaped(leader);
        assert!(descendant_stopped(descendant).await?);
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await??;
    Ok(())
}

fn fail_with_group(_child: ChildOwner) -> Result<(), HarnessError> {
    Err(HarnessError::invalid("early failure"))
}

#[test]
fn given_relative_collector_when_loading_then_definition_directory_is_used() -> TestResult {
    let root = tempfile::tempdir()?;
    let binary = root.path().join("unique-relative-collector");
    std::os::unix::fs::symlink("/usr/bin/true", &binary)?;
    let definition = root.path().join("tool.toml");
    std::fs::write(
        &definition,
        "collector = 'unique-relative-collector'\nversion = ['${H2H_COLLECTOR}', '--version']\n",
    )?;
    let loaded = Definition::load("other", &definition)?;
    let Tool::Collector { binary: actual } = loaded.tool else {
        return Err("expected collector definition".into());
    };
    assert_eq!(actual, std::fs::canonicalize(binary)?);
    Ok(())
}

#[tokio::test]
async fn given_wait_future_when_cancelled_then_descendant_is_stopped() -> TestResult {
    tokio::time::timeout(Duration::from_secs(35), async {
        let (child, descendant) = spawn_waiting_group().await?;
        let leader = child.pid();
        tokio::select! {
            biased;
            result = wait_process(child, "collect", Duration::from_secs(300)) => {
                return Err(format!("wait unexpectedly completed: {result:?}").into());
            }
            () = std::future::ready(()) => {}
        }
        assert!(
            descendant_stopped(descendant).await?,
            "cancelled wait left descendant alive"
        );
        assert_reaped(leader);
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await??;
    Ok(())
}

#[tokio::test]
async fn given_unrelated_group_when_clean_tool_exits_then_no_leftovers_are_reported() -> TestResult
{
    tokio::time::timeout(Duration::from_secs(35), async {
        let (mut unrelated, _) = spawn_waiting_group().await?;
        assert!(unrelated.exited()?.is_none());
        let root = tempfile::tempdir()?;
        let result = execute_process(Command::new("true"), root.path(), "collect").await?;
        assert!(!result.left_processes);
        assert!(unrelated.exited()?.is_none());
        let status = unrelated.terminate()?;
        use std::os::unix::process::ExitStatusExt;
        assert_eq!(status.signal(), Some(9));
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await??;
    Ok(())
}

#[test]
fn given_missing_definition_when_loading_then_error_names_tool_and_path() -> TestResult {
    let root = tempfile::tempdir()?;
    let path = root.path().join("missing.toml");
    let error = Definition::load("other", &path)
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        format!(
            "tool other definition {}: No such file or directory (os error 2)",
            path.display()
        )
    );
    Ok(())
}

#[test]
fn given_empty_collector_when_loading_then_definition_is_rejected() -> TestResult {
    let root = tempfile::tempdir()?;
    let path = root.path().join("empty.toml");
    std::fs::write(&path, "collector = ''\nversion = ['true']\n")?;
    let error = Definition::load("other", &path)
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        format!(
            "tool other definition {}: collector must not be empty",
            path.display()
        )
    );
    Ok(())
}

#[test]
fn given_directory_collector_when_loading_then_definition_is_rejected() -> TestResult {
    let root = tempfile::tempdir()?;
    let path = root.path().join("directory.toml");
    std::fs::write(&path, "collector = '.'\nversion = ['true']\n")?;
    let error = Definition::load("other", &path)
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        format!(
            "tool other definition {}: collector {} is not a regular file",
            path.display(),
            root.path().display()
        )
    );
    Ok(())
}

#[test]
fn given_missing_collector_when_loading_then_error_names_tool_definition_and_binary() -> TestResult
{
    let root = tempfile::tempdir()?;
    let path = root.path().join("missing.toml");
    std::fs::write(&path, "collector = 'absent'\nversion = ['true']\n")?;
    let error = Definition::load("other", &path)
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        format!(
            "tool other definition {}: collector {}: No such file or directory (os error 2)",
            path.display(),
            root.path().join("absent").display()
        )
    );
    Ok(())
}

#[test]
fn given_absolute_or_parent_collector_when_loading_then_regular_file_is_canonicalized() -> TestResult
{
    let root = tempfile::tempdir()?;
    let nested = root.path().join("nested");
    std::fs::create_dir(&nested)?;
    let binary = root.path().join("collector");
    std::os::unix::fs::symlink("/usr/bin/true", &binary)?;
    let path = nested.join("tool.toml");
    for collector in [PathBuf::from("../collector"), binary.clone()] {
        std::fs::write(
            &path,
            format!(
                "collector = '{}'\nversion = ['true']\n",
                collector.display()
            ),
        )?;
        let loaded = Definition::load("other", &path)?;
        let Tool::Collector { binary: actual } = loaded.tool else {
            return Err("expected collector definition".into());
        };
        assert_eq!(actual, std::fs::canonicalize(&binary)?);
    }
    Ok(())
}
