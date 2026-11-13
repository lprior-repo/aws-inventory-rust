use super::*;

pub(super) struct OwnedChild(Child);

impl Drop for OwnedChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

pub(super) struct OwnedGraph {
    pub(super) address: SocketAddr,
    pub(super) name: Option<String>,
}

impl OwnedGraph {
    pub(super) fn remove(&mut self) -> Result<()> {
        let Some(name) = &self.name else {
            return Ok(());
        };
        let mut socket = TcpStream::connect_timeout(&self.address, Duration::from_secs(2))?;
        socket.set_read_timeout(Some(Duration::from_secs(2)))?;
        socket.set_write_timeout(Some(Duration::from_secs(2)))?;
        write!(
            socket,
            "*2\r\n$12\r\nGRAPH.DELETE\r\n${}\r\n{}\r\n",
            name.len(),
            name
        )?;
        let mut response = [0_u8; 5];
        socket.read_exact(&mut response)?;
        ensure!(
            &response == b"+OK\r\n",
            "graph cleanup did not receive the server acknowledgment"
        );
        self.name = None;
        Ok(())
    }
}

impl Drop for OwnedGraph {
    fn drop(&mut self) {
        let _ = self.remove();
    }
}

pub(super) fn cli(arguments: &[&str], expected_exit: i32) -> Result<Value> {
    let mut stdout = tempfile::tempfile()?;
    let mut stderr = tempfile::tempfile()?;
    let mut child = OwnedChild(
        Command::new(env!("CARGO_BIN_EXE_aws-inventory"))
            .args(arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout.try_clone()?))
            .stderr(Stdio::from(stderr.try_clone()?))
            .spawn()?,
    );
    let mut status = None;
    for _ in 0..1_200 {
        if let Some(finished) = child.0.try_wait()? {
            status = Some(finished);
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let status = status.context("fixture CLI exceeded its 60-second deadline")?;
    ensure!(
        stdout.metadata()?.len() <= MAX_DOCUMENT && stderr.metadata()?.len() <= 65_536,
        "fixture CLI exceeded its output budget"
    );
    stdout.seek(SeekFrom::Start(0))?;
    stderr.seek(SeekFrom::Start(0))?;
    let mut diagnostic = String::new();
    stderr.take(65_536).read_to_string(&mut diagnostic)?;
    ensure!(
        status.code() == Some(expected_exit),
        "fixture CLI exited {status}: {diagnostic}"
    );
    Ok(serde_json::from_reader(stdout.take(MAX_DOCUMENT))?)
}

pub(super) fn case<'a>(snapshot: &'a Value, name: &str) -> Result<&'a Value> {
    snapshot
        .get("cases")
        .and_then(Value::as_array)
        .context("snapshot cases missing")?
        .iter()
        .find(|case| case.get("name").and_then(Value::as_str) == Some(name))
        .context("required snapshot case missing")
}

pub(super) fn ordered(values: &[Value]) -> Result<Vec<String>> {
    let mut encoded = values
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<Vec<_>, _>>()?;
    encoded.sort();
    Ok(encoded)
}

pub(super) fn query(database: &str, sql: &str) -> Result<Vec<Value>> {
    cli(&["query", "--db", database, "--limit", "2048", sql], 0)?
        .as_array()
        .cloned()
        .context("query result is not a row array")
}

pub(super) fn observations(
    database: &str,
    scan_id: &str,
    expected: &Value,
    before: u128,
    after: u128,
) -> Result<()> {
    for (table, column, key) in [
        ("resource_observations", "resource_json", "resources"),
        ("edge_observations", "edge_json", "edges"),
    ] {
        let rows = query(
            database,
            &format!("SELECT {column} FROM {table} WHERE scan_id='{scan_id}'"),
        )?;
        let mut values = Vec::with_capacity(rows.len());
        for row in rows {
            let mut value: Value = serde_json::from_str(
                row.get(column)
                    .and_then(Value::as_str)
                    .context("serialized observation missing")?,
            )?;
            let observed = value
                .as_object_mut()
                .context("observation must be an object")?
                .remove("observed_at")
                .and_then(|time| time.as_u64())
                .context("observation time missing")?;
            ensure!(
                (before..=after).contains(&u128::from(observed)),
                "observation time outside the scan interval"
            );
            values.push(value);
        }
        assert_eq!(
            ordered(&values)?,
            ordered(
                expected
                    .get(key)
                    .and_then(Value::as_array)
                    .context("snapshot observations missing")?
            )?,
            "{key} differ"
        );
    }
    let mut scopes = query(
        database,
        &format!(
            "SELECT scope_key,account_id,region,collector,parent_id,source,last_page,next_token IS NOT NULL AS has_next_token,complete,failed FROM scope_checkpoints WHERE scan_id='{scan_id}'"
        ),
    )?;
    for scope in &mut scopes {
        let object = scope.as_object_mut().context("scope must be an object")?;
        for key in ["has_next_token", "complete", "failed"] {
            let value = object
                .get(key)
                .and_then(Value::as_i64)
                .context("scope flag missing")?;
            ensure!(matches!(value, 0 | 1), "invalid scope flag");
            object.insert(key.into(), json!(value == 1));
        }
    }
    assert_eq!(
        ordered(&scopes)?,
        ordered(
            expected
                .get("scopes")
                .and_then(Value::as_array)
                .context("snapshot scopes missing")?
        )?
    );
    let failures = query(
        database,
        &format!(
            "SELECT scope_key,category,message FROM collection_failures WHERE scan_id='{scan_id}'"
        ),
    )?;
    assert_eq!(
        ordered(&failures)?,
        ordered(
            expected
                .get("failures")
                .and_then(Value::as_array)
                .context("snapshot failures missing")?
        )?
    );
    Ok(())
}
