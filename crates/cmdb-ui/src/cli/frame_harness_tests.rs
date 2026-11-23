use super::{Command, parse};
use crate::frame_harness::FrameConfig;

#[test]
fn given_database_and_duration_when_parsing_then_harness_configuration_reaches_worker()
-> Result<(), String> {
    let args = ["--frame-harness", "20", "--database", "inventory.sqlite"].map(str::to_owned);
    let Command::Run(config) = parse(&args)? else {
        return Err("expected run configuration".to_owned());
    };
    assert_eq!(config.frame_harness, Some(FrameConfig::parse("20")?));
    assert_eq!(
        config.database,
        Some(std::path::PathBuf::from("inventory.sqlite"))
    );
    Ok(())
}

#[test]
fn given_harness_without_database_when_parsing_then_missing_target_is_explicit() {
    let args = ["--frame-harness", "20"].map(str::to_owned);
    assert_eq!(
        parse(&args),
        Err("--frame-harness requires --database PATH".to_owned())
    );
}
