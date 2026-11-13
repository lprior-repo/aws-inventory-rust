//! In-process AWS Emulator seam.
//!
//! The embedded facade replaces the HTTP transport inside the AWS SDK client, so
//! a test exercises the real request/response machinery with no socket and no
//! TLS for direct SDK calls. Building it links the complete service router: run
//! these tests explicitly (`cargo test` in this directory), never as part of the
//! main workspace's ordinary suite.

/// Stack for constructing the facade; the router overflows a default thread.
pub const BOOT_STACK_SIZE: usize = 64 * 1024 * 1024;
/// Stack for each async worker that drives SDK calls.
pub const WORKER_STACK_SIZE: usize = 16 * 1024 * 1024;
