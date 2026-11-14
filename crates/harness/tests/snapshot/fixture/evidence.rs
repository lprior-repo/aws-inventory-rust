use std::collections::BTreeMap;
use std::path::Path;

use serde_json::{Value, json};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Connection, SqliteConnection};

use super::CASE_COMMANDS;
use super::ROLE;
use super::io::{insert, write_json};

pub(super) fn seed_commands(
    evidence: &Path,
    binary_sha: &str,
    execution: &mut Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut selections = BTreeMap::new();
    selections.insert("native", vec!["--service", "all"]);
    selections.insert(
        "sources",
        vec![
            "--service",
            "config,resource-explorer",
            "--discover-sources",
        ],
    );
    selections.insert(
        "cloud-control",
        vec![
            "--service",
            "cloud-control",
            "--cloud-control",
            "--resource-type",
            "AWS::EKS::Cluster",
        ],
    );
    selections.insert(
        "parents",
        vec![
            "--service",
            "cloud-control",
            "--cloud-control",
            "--resource-type",
            "AWS::EKS::Addon",
        ],
    );
    selections.insert("assumed-role", vec!["--service", "iam", "--role-arn", ROLE]);
    for name in CASE_COMMANDS {
        let selection = selections.get(name).ok_or("missing selection")?;
        let arguments = common_arguments(evidence)
            .into_iter()
            .chain(selection.iter().map(|value| (*value).to_owned()))
            .collect::<Vec<_>>();
        let scan_id = format!("synthetic-{name}");
        let command = json!({
            "name": name, "arguments": arguments, "exit_code": 0, "binary_sha256": binary_sha,
            "result": {"status": "complete", "write_error": null,
                "scans": [{"scan_id": scan_id, "status": "complete", "error": null}]}});
        let sha = write_json(&evidence.join(format!("{name}.json")), &command)?;
        let binding = json!({"file": format!("{name}.json"), "sha256": sha,
            "scan_id": scan_id, "binary_sha256": binary_sha});
        if let Some(scan_commands) = execution
            .get_mut("scan_commands")
            .and_then(Value::as_object_mut)
        {
            scan_commands.insert(name.to_owned(), binding.clone());
        }
        if name == "cloud-control" {
            insert(execution, "cloud_control_command", binding.clone())?;
        }
        if name == "parents" {
            insert(execution, "parent_command", binding)?;
        }
    }
    Ok(())
}

fn common_arguments(evidence: &Path) -> Vec<String> {
    let database = evidence
        .join("inventory.sqlite")
        .to_string_lossy()
        .into_owned();
    [
        "scan",
        "--db",
        database.as_str(),
        "--region",
        "us-east-1",
        "--attempts",
        "2",
        "--max-pages",
        "100",
        "--page-size",
        "1",
        "--concurrency",
        "32",
        "--per-bucket-concurrency",
        "4",
        "--rps",
        "100",
    ]
    .iter()
    .map(|value| (*value).to_owned())
    .collect()
}

pub(super) async fn create_database(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true);
    let mut connection = SqliteConnection::connect_with(&options).await?;
    for statement in [
        "CREATE TABLE scans(scan_id TEXT PRIMARY KEY,status TEXT)",
        "CREATE TABLE resource_observations(scan_id TEXT,resource_json TEXT)",
        "CREATE TABLE edge_observations(scan_id TEXT,edge_json TEXT)",
        "CREATE TABLE scope_checkpoints(scan_id TEXT,scope_key TEXT,account_id TEXT,region TEXT,\
         collector TEXT,parent_id TEXT,source TEXT,last_page INTEGER,next_token TEXT,\
         complete INTEGER,failed INTEGER)",
        "CREATE TABLE collection_failures(scan_id TEXT,scope_key TEXT,category TEXT,message TEXT)",
    ] {
        sqlx::query(statement).execute(&mut connection).await?;
    }
    for name in CASE_COMMANDS {
        insert_scan(&mut connection, &format!("synthetic-{name}"), name).await?;
    }
    connection.close().await?;
    Ok(())
}

async fn insert_scan(
    connection: &mut SqliteConnection,
    scan_id: &str,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let resource = json!({"id": name, "observed_at": 123,
        "properties": {"observed_at": "provider-time", "array": [2, 1]}});
    let edge = json!({"from_id": name, "to_id": "parent", "observed_at": 124,
        "properties": {"kept": true}});
    sqlx::query("INSERT INTO scans VALUES (?, 'complete')")
        .bind(scan_id)
        .execute(&mut *connection)
        .await?;
    sqlx::query("INSERT INTO resource_observations VALUES (?,?)")
        .bind(scan_id)
        .bind(resource.to_string())
        .execute(&mut *connection)
        .await?;
    sqlx::query("INSERT INTO edge_observations VALUES (?,?)")
        .bind(scan_id)
        .bind(edge.to_string())
        .execute(&mut *connection)
        .await?;
    sqlx::query("INSERT INTO scope_checkpoints VALUES (?,?,?,?,?,?,?,?,?,?,?)")
        .bind(scan_id)
        .bind(format!("scope-{name}"))
        .bind("000000000000")
        .bind("us-east-1")
        .bind(name)
        .bind(None::<String>)
        .bind("direct")
        .bind(2_i64)
        .bind(None::<String>)
        .bind(1_i64)
        .bind(0_i64)
        .execute(&mut *connection)
        .await?;
    Ok(())
}
