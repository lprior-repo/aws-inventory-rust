//! Independent assertions over persisted fixture evidence.
//!
//! Expectations are authored semantic fixtures, not the generated descriptor
//! rules and not the output under test. Each module returns the same
//! `{"check": ..., "passed": ...}` records the Python harness emits.

pub mod cloud_control;
