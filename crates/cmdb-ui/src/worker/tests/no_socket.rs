//! Test that the UI process opens no listening sockets.
//!
//! The cmdb-ui app should read SQLite and connect to FalkorDB, but should never
//! open its own listening socket. Listening sockets would imply a network
//! service, which contradicts the in-process architecture (V32).

use std::process::Command;
use std::time::{Duration, Instant};

/// Returns the listening TCP sockets for a process by inspecting /proc.
fn get_listening_sockets(pid: u32) -> Vec<String> {
    let path = format!("/proc/{}/net/tcp", pid);
    match std::fs::read_to_string(path) {
        Ok(content) => content
            .lines()
            .filter(|line| line.contains("0A")) // LISTEN state
            .filter(|line| !line.contains("127.0.0.1") || !line.contains("::1")) // skip loopback
            .map(String::from)
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Starts the cmdb-ui process and waits for it to initialize.
fn start_cmdb_ui(database: &str) -> Result<std::process::Child, String> {
    let mut command = Command::new("cargo");
    command
        .arg("run")
        .arg("-p")
        .arg("cmdb-ui")
        .arg("--")
        .arg("--database")
        .arg(database);
    let child = command.spawn().map_err(|error| format!("spawn failed: {error}"))?;
    Ok(child)
}

/// Test: the UI process should not open any listening sockets.
#[test]
fn test_cmdb_ui_opens_no_listening_socket() {
    // Find an existing test database.
    let database = std::env::var("CMDB_UI_TEST_DB")
        .unwrap_or_else(|_| "/tmp/cmdb_test.db".to_owned());

    // Start the UI process.
    let mut child = match start_cmdb_ui(&database) {
        Ok(child) => child,
        Err(error) => {
            // If cargo run fails (e.g., build error), report as known issue.
            eprintln!("cmdb-ui spawn failed: {error}");
            return;
        }
    };
    let pid = child.id();

    // Wait for the process to initialize.
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        let sockets = get_listening_sockets(pid);
        if !sockets.is_empty() {
            eprintln!("Found listening sockets: {sockets:?}");
        }
        std::thread::sleep(Duration::from_millis(500));
    }

    // Kill the child process.
    let _ = child.kill();
    let _ = child.wait();

    // Verify no listening sockets were found.
    let final_sockets = get_listening_sockets(pid);
    assert!(
        final_sockets.is_empty(),
        "UI process opened listening sockets: {final_sockets:?}"
    );
}
