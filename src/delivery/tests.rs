use super::*;
use clap::Parser;
use proptest::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};

fn options(directory: &Path) -> DeliveryArgs {
    DeliveryArgs {
        output_dir: Some(directory.to_owned()),
        ..DeliveryArgs::default()
    }
}

mod configuration;
mod failure;
mod publication;
