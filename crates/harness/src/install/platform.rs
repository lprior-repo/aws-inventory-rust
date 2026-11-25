use crate::HarnessError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Platform {
    LinuxX86_64,
    LinuxAarch64,
    MacosAarch64,
}

impl Platform {
    pub fn select(os: &str, arch: &str) -> Result<Self, HarnessError> {
        match (os, arch) {
            ("linux", "x86_64") => Ok(Self::LinuxX86_64),
            ("linux", "aarch64") => Ok(Self::LinuxAarch64),
            ("macos", "aarch64") => Ok(Self::MacosAarch64),
            _ => Err(HarnessError::invalid(format!(
                "unsupported platform {os}/{arch}; supported: linux/x86_64, linux/aarch64, macos/aarch64"
            ))),
        }
    }

    pub fn current() -> Result<Self, HarnessError> {
        Self::select(std::env::consts::OS, std::env::consts::ARCH)
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::LinuxX86_64 => "linux/x86_64",
            Self::LinuxAarch64 => "linux/aarch64",
            Self::MacosAarch64 => "macos/aarch64",
        }
    }

    pub fn unsupported(self, component: &str) -> HarnessError {
        HarnessError::invalid(format!(
            "{component} has no pinned artifact for {}; supported Linux targets only",
            self.name()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::Platform;

    #[test]
    fn supported_targets_are_selected() {
        assert_eq!(
            Platform::select("linux", "x86_64").ok(),
            Some(Platform::LinuxX86_64)
        );
        assert_eq!(
            Platform::select("linux", "aarch64").ok(),
            Some(Platform::LinuxAarch64)
        );
        assert_eq!(
            Platform::select("macos", "aarch64").ok(),
            Some(Platform::MacosAarch64)
        );
    }

    #[test]
    fn unsupported_targets_name_the_platform() {
        for (os, arch) in [
            ("linux", "riscv64"),
            ("macos", "x86_64"),
            ("windows", "x86_64"),
        ] {
            let error = Platform::select(os, arch).err().map(|e| e.to_string());
            let message = error.unwrap_or_default();
            assert!(message.contains(&format!("{os}/{arch}")), "{message}");
        }
    }

    #[test]
    fn unsupported_component_names_the_platform() {
        let message = Platform::MacosAarch64
            .unsupported("AWS Emulator")
            .to_string();
        assert!(message.contains("AWS Emulator"), "{message}");
        assert!(message.contains("macos/aarch64"), "{message}");
    }
}
