use std::process::Command;
use std::time::{Duration, Instant};

fn get_listening_sockets(pid: u32) -> Vec<String> {
    let path = format!("/proc/{}/net/tcp", pid);
    match std::fs::read_to_string(path) {
        Ok(content) => content
            .lines()
            .filter(|line| line.contains("0A"))
            .filter(|line| !line.contains("127.0.0.1") || !line.contains("::1"))
            .map(String::from)
            .collect(),
        Err(_) => Vec::new(),
    }
}

fn start_cmdb_ui(database: &str) -> Result<std::process::Child, String> {
    let mut command = Command::new("cargo");
    command
        .arg("run")
        .arg("-p")
        .arg("cmdb-ui")
        .arg("--")
        .arg("--database")
        .arg(database);
    let child = command
        .spawn()
        .map_err(|error| format!("spawn failed: {error}"))?;
    Ok(child)
}

#[test]
fn test_cmdb_ui_opens_no_listening_socket() {
    let database =
        std::env::var("CMDB_UI_TEST_DB").unwrap_or_else(|_| "/tmp/cmdb_test.db".to_owned());

    let mut child = match start_cmdb_ui(&database) {
        Ok(child) => child,
        Err(error) => {
            eprintln!("cmdb-ui spawn failed: {error}");
            return;
        }
    };
    let pid = child.id();

    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        let sockets = get_listening_sockets(pid);
        if !sockets.is_empty() {
            eprintln!("Found listening sockets: {sockets:?}");
        }
        std::thread::sleep(Duration::from_millis(500));
    }

    let _ = child.kill();
    let _ = child.wait();

    let final_sockets = get_listening_sockets(pid);
    assert!(
        final_sockets.is_empty(),
        "UI process opened listening sockets: {final_sockets:?}"
    );
}
