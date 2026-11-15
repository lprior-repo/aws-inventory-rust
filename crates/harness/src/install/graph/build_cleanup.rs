use std::ffi::{CStr, CString, OsString};
use std::ops::ControlFlow;
use std::os::fd::{BorrowedFd, OwnedFd};
use std::os::unix::ffi::OsStringExt;

use super::OwnedDirectory;
use crate::HarnessError;

enum CleanupName {
    Root,
    Child(CString),
}

struct CleanupFrame {
    entries: rustix::fs::Dir,
    name: CleanupName,
}

pub(super) fn clear_owned(directory: &OwnedFd) -> Result<(), HarnessError> {
    use ControlFlow::{Break, Continue};
    let mut stack = cleanup_stack(directory)?;
    match (0..100_000).try_for_each(|_| match cleanup_step(&mut stack) {
        Ok(Continue(())) => Continue(()),
        Ok(Break(())) => Break(Ok(())),
        Err(error) => Break(Err(error)),
    }) {
        Break(result) => result,
        Continue(()) => Err(HarnessError::invalid("source cleanup exceeds entry limit")),
    }
}

fn cleanup_stack(directory: &OwnedFd) -> Result<Vec<CleanupFrame>, HarnessError> {
    let mut stack = Vec::new();
    stack.try_reserve_exact(64).map_err(|error| HarnessError::invalid(error.to_string()))?;
    stack.push(CleanupFrame {
        entries: rustix::fs::Dir::read_from(directory).map_err(std::io::Error::from)?,
        name: CleanupName::Root,
    });
    Ok(stack)
}

fn cleanup_step(stack: &mut Vec<CleanupFrame>) -> Result<ControlFlow<()>, HarnessError> {
    let Some(frame) = stack.last_mut() else {
        return Ok(ControlFlow::Break(()));
    };
    match frame.entries.next() {
        Some(entry) => cleanup_entry(stack, entry.map_err(std::io::Error::from)?)?,
        None => cleanup_ascend(stack)?,
    }
    Ok(ControlFlow::Continue(()))
}

fn cleanup_entry(stack: &mut Vec<CleanupFrame>, entry: rustix::fs::DirEntry) -> Result<(), HarnessError> {
    let name = entry.file_name();
    if matches!(name.to_bytes(), b"." | b"..") {
        return Ok(());
    }
    let parent = stack.last().ok_or_else(|| HarnessError::invalid("cleanup parent missing"))?;
    let fd = parent.entries.fd().map_err(std::io::Error::from)?;
    if let Some(child) = cleanup_open(fd, name)? {
        cleanup_descend(stack, child, name.to_owned())?;
    }
    Ok(())
}

fn cleanup_open(fd: BorrowedFd<'_>, name: &CStr) -> Result<Option<OwnedFd>, HarnessError> {
    match rustix::fs::openat(fd, name, super::DIRECTORY_FLAGS, rustix::fs::Mode::empty()) {
        Ok(child) => Ok(Some(child)),
        Err(error) if error == rustix::io::Errno::NOTDIR || error == rustix::io::Errno::LOOP => {
            rustix::fs::unlinkat(fd, name, rustix::fs::AtFlags::empty())
                .map_err(std::io::Error::from)?;
            Ok(None)
        }
        Err(error) => Err(std::io::Error::from(error).into()),
    }
}

fn cleanup_descend(
    stack: &mut Vec<CleanupFrame>,
    child: OwnedFd,
    name: CString,
) -> Result<(), HarnessError> {
    if stack.len() >= 64 {
        return Err(HarnessError::invalid("source cleanup exceeds directory depth limit"));
    }
    stack.push(CleanupFrame {
        entries: rustix::fs::Dir::new(child).map_err(std::io::Error::from)?,
        name: CleanupName::Child(name),
    });
    Ok(())
}

fn cleanup_ascend(stack: &mut Vec<CleanupFrame>) -> Result<(), HarnessError> {
    let frame = stack.pop().ok_or_else(|| HarnessError::invalid("cleanup frame missing"))?;
    if let CleanupName::Child(name) = frame.name {
        let parent = stack.last().ok_or_else(|| HarnessError::invalid("cleanup parent missing"))?;
        let parent_fd = parent.entries.fd().map_err(std::io::Error::from)?.try_clone_to_owned()?;
        let child_fd = frame.entries.fd().map_err(std::io::Error::from)?.try_clone_to_owned()?;
        // Ownership is the opened inode, not the mutable name used to descend.
        // remove_empty claims into a private quarantine and checks that identity
        // before unlinking; a substituted foreign directory is preserved/restored.
        OwnedDirectory::retained(parent_fd, OsString::from_vec(name.into_bytes()), child_fd)
            .remove_empty()?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "build_cleanup_tests.rs"]
mod tests;
