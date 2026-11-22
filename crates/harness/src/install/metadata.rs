use std::path::{Path, PathBuf};

use serde_json::Value;

use super::platform::Platform;
use crate::HarnessError;

const LINUX_AARCH64_PACKAGE_URL: &str = "https://downloads.aws.com/aws-emulator/packages/aws-emulator/LTS/aws-emulator_1.0.348-1_arm64.deb";
const LINUX_AARCH64_PACKAGE_SHA256: &str =
    "895490c425be74a6c1d449e7b89f4fccda16aa06833f61d3aa3e142ee1cbff15";

pub(super) struct Metadata {
    pub(super) repository: String,
    pub(super) commit: String,
    pub(super) package_url: String,
    pub(super) package_sha256: String,
    pub(super) package_file: &'static str,
    pub(super) signing_key_url: String,
    pub(super) signing_fingerprint: String,
}

impl Metadata {
    pub(super) fn read(path: &Path, platform: Platform) -> Result<Self, HarnessError> {
        let (document, _) = crate::snapshot::read_json(path)?;
        let text = |key: &str| -> Result<String, HarnessError> {
            document
                .get(key)
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| HarnessError::invalid(format!("upstream.json lacks {key}")))
        };
        let (package_url, package_sha256, package_file) = match platform {
            Platform::LinuxX86_64 => (
                text("package_url")?,
                text("package_sha256")?,
                "aws-emulator_amd64.deb",
            ),
            Platform::LinuxAarch64 => (
                LINUX_AARCH64_PACKAGE_URL.to_owned(),
                LINUX_AARCH64_PACKAGE_SHA256.to_owned(),
                "aws-emulator_arm64.deb",
            ),
            Platform::MacosAarch64 => return Err(platform.unsupported("AWS Emulator")),
        };
        Ok(Self {
            repository: text("repository")?,
            commit: text("commit")?,
            package_url,
            package_sha256,
            package_file,
            signing_key_url: text("package_signing_key_url")?,
            signing_fingerprint: text("package_signing_key_fingerprint")?,
        })
    }

    pub(super) fn signature_file(&self, destination: &Path) -> PathBuf {
        destination.join(format!("{}.sig", self.package_file))
    }
}
