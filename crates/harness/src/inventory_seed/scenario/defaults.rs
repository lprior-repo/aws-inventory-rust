use crate::inventory_seed::file;

use super::*;

pub(super) fn scenario() -> Result<SeedScenario, SeedError> {
    let content = include_str!("../../../seeds/ec2.toml");
    file::load(content)
}
