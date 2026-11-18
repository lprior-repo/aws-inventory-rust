use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use rustix::fd::OwnedFd;
use rustix::fs::{AtFlags, Mode, OFlags, RenameFlags};

use super::{build, complete, executable, regular_file};
use crate::HarnessError;

pub(super) const DIRECTORY_FLAGS: OFlags = OFlags::RDONLY
    .union(OFlags::DIRECTORY)
    .union(OFlags::NOFOLLOW)
    .union(OFlags::NONBLOCK)
    .union(OFlags::CLOEXEC);

pub(super) fn descriptor_path(file: &impl std::os::fd::AsRawFd) -> PathBuf {
    PathBuf::from(format!(
        "/proc/{}/fd/{}",
        std::process::id(),
        file.as_raw_fd()
    ))
}

pub(super) struct OwnedDirectory {
    parent: OwnedFd,
    name: OsString,
    directory: OwnedFd,
}

#[derive(Clone, Copy)]
enum Removal {
    Empty,
    Tree,
}

impl OwnedDirectory {
    pub(super) fn retained(parent: OwnedFd, name: OsString, directory: OwnedFd) -> Self {
        Self {
            parent,
            name,
            directory,
        }
    }

    pub(super) fn claim(path: &Path) -> Result<Self, HarnessError> {
        let (parent, name) = Self::location(path)?;
        let private = Self::private(parent.try_clone()?)?;
        private.verify()?;
        let result = rustix::fs::renameat_with(
            &private.parent,
            &private.name,
            &parent,
            &name,
            RenameFlags::NOREPLACE,
        )
        .map_err(std::io::Error::from);
        match result {
            Ok(()) => private.relocate(parent, name),
            Err(error) => complete(Err(error.into()), private.remove_private_empty()),
        }
    }

    fn relocate(self, parent: OwnedFd, name: OsString) -> Result<Self, HarnessError> {
        let owned = Self {
            parent,
            name,
            directory: self.directory,
        };
        owned.verify()?;
        Ok(owned)
    }

    #[cfg(test)]
    pub(super) fn capture(path: &Path) -> Result<Self, HarnessError> {
        let (parent, name) = Self::location(path)?;
        Self::open(parent, name)
    }

    fn location(path: &Path) -> Result<(OwnedFd, OsString), HarnessError> {
        let parent = path
            .parent()
            .ok_or_else(|| HarnessError::invalid("directory has no parent"))?;
        let name = path
            .file_name()
            .ok_or_else(|| HarnessError::invalid("directory has no name"))?;
        let parent = rustix::fs::open(parent, DIRECTORY_FLAGS, Mode::empty())
            .map_err(std::io::Error::from)?;
        Ok((parent, name.to_owned()))
    }

    fn open(parent: OwnedFd, name: OsString) -> Result<Self, HarnessError> {
        let directory = rustix::fs::openat(&parent, &name, DIRECTORY_FLAGS, Mode::empty())
            .map_err(std::io::Error::from)?;
        Ok(Self {
            parent,
            name,
            directory,
        })
    }

    fn private(parent: OwnedFd) -> Result<Self, HarnessError> {
        let mut nonce = [0_u8; 16];
        let count =
            rustix::rand::getrandom(nonce.as_mut_slice(), rustix::rand::GetRandomFlags::empty())
                .map_err(std::io::Error::from)?;
        if count != nonce.len() {
            return Err(HarnessError::invalid(
                "incomplete cleanup namespace entropy",
            ));
        }
        let name = OsString::from(format!(".graph-owned-{:x}", base16ct::HexDisplay(&nonce)));
        rustix::fs::mkdirat(&parent, &name, Mode::RWXU).map_err(std::io::Error::from)?;
        Self::open(parent, name)
    }

    pub(super) fn anchor(&self) -> PathBuf {
        descriptor_path(&self.directory)
    }

    pub(super) fn parent(&self) -> std::os::fd::BorrowedFd<'_> {
        std::os::fd::AsFd::as_fd(&self.parent)
    }

    pub(super) fn open_binary(&self, name: &str) -> Result<std::fs::File, HarnessError> {
        let src = rustix::fs::openat(&self.directory, "src", DIRECTORY_FLAGS, Mode::empty())
            .map_err(std::io::Error::from)?;
        let flags = OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC;
        let file = std::fs::File::from(
            rustix::fs::openat(src, name, flags, Mode::empty()).map_err(std::io::Error::from)?,
        );
        let metadata = file.metadata()?;
        regular_file(&metadata)?;
        executable(&file)?;
        Ok(file)
    }

    pub(super) fn verify(&self) -> Result<(), HarnessError> {
        self.verify_in(&self.parent, &self.name)
    }

    fn verify_in(&self, parent: &OwnedFd, name: &OsStr) -> Result<(), HarnessError> {
        let current = rustix::fs::statat(parent, name, AtFlags::SYMLINK_NOFOLLOW)
            .map_err(std::io::Error::from)?;
        let owned = rustix::fs::fstat(&self.directory).map_err(std::io::Error::from)?;
        if current.st_dev == owned.st_dev && current.st_ino == owned.st_ino {
            return Ok(());
        }
        Err(HarnessError::invalid("owned directory identity changed"))
    }

    pub(super) fn remove_empty(self) -> Result<(), HarnessError> {
        self.remove(Removal::Empty)
    }

    pub(super) fn remove_tree(self) -> Result<(), HarnessError> {
        self.remove(Removal::Tree)
    }

    fn remove(self, mode: Removal) -> Result<(), HarnessError> {
        if let Err(error) = self.verify() {
            return complete(Err(error), self.clear(mode));
        }
        let quarantine = Self::private(self.parent.try_clone()?)?;
        let result = self.claim_and_remove(&quarantine, mode);
        complete(result, quarantine.remove_private_empty())
    }

    fn claim_and_remove(&self, quarantine: &Self, mode: Removal) -> Result<(), HarnessError> {
        rustix::fs::renameat_with(
            &self.parent,
            &self.name,
            &quarantine.directory,
            "owned",
            RenameFlags::NOREPLACE,
        )
        .map_err(std::io::Error::from)?;
        if let Err(error) = self.verify_in(&quarantine.directory, OsStr::new("owned")) {
            return complete(Err(error), self.restore(quarantine));
        }
        self.clear(mode)?;
        self.verify_in(&quarantine.directory, OsStr::new("owned"))?;
        rustix::fs::unlinkat(&quarantine.directory, "owned", AtFlags::REMOVEDIR)
            .map_err(std::io::Error::from)?;
        Ok(())
    }

    fn restore(&self, quarantine: &Self) -> Result<(), HarnessError> {
        rustix::fs::renameat_with(
            &quarantine.directory,
            "owned",
            &self.parent,
            &self.name,
            RenameFlags::NOREPLACE,
        )
        .map_err(std::io::Error::from)?;
        Ok(())
    }

    fn clear(&self, mode: Removal) -> Result<(), HarnessError> {
        match mode {
            Removal::Empty => Ok(()),
            Removal::Tree => build::clear_owned(&self.directory),
        }
    }

    fn remove_private_empty(&self) -> Result<(), HarnessError> {
        self.verify()?;
        rustix::fs::unlinkat(&self.parent, &self.name, AtFlags::REMOVEDIR)
            .map_err(std::io::Error::from)?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "ownership_tests.rs"]
mod tests;
