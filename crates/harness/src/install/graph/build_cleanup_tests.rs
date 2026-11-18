use std::fs;
use std::os::unix::fs::MetadataExt;
use std::sync::{Arc, Barrier};

use super::super::installer_tests::TestResult;
use super::{cleanup_descend, cleanup_open, cleanup_stack, cleanup_step};

#[test]
fn given_open_child_when_renamed_before_ascend_then_empty_foreign_replacement_survives()
-> TestResult {
    let directory = tempfile::tempdir()?;
    fs::create_dir(directory.path().join("child"))?;
    fs::write(directory.path().join("child/owned"), b"owned")?;
    let root = rustix::fs::open(
        directory.path(),
        super::super::DIRECTORY_FLAGS,
        rustix::fs::Mode::empty(),
    )?;
    let mut stack = cleanup_stack(&root)?;
    let opened = cleanup_open(root.as_fd(), c"child")?.ok_or("child was not opened")?;
    let original = rustix::fs::fstat(&opened)?;
    cleanup_descend(&mut stack, opened, c"child".to_owned())?;

    let barrier = Arc::new(Barrier::new(2));
    let worker_barrier = Arc::clone(&barrier);
    let rename_parent = root.try_clone()?;
    let worker = std::thread::spawn(move || -> Result<_, std::io::Error> {
        worker_barrier.wait();
        let result = (|| {
            rustix::fs::renameat(&rename_parent, "child", &rename_parent, "saved")?;
            rustix::fs::mkdirat(&rename_parent, "child", rustix::fs::Mode::RWXU)?;
            rustix::fs::statat(
                &rename_parent,
                "child",
                rustix::fs::AtFlags::SYMLINK_NOFOLLOW,
            )
        })()
        .map_err(std::io::Error::from);
        worker_barrier.wait();
        result
    });
    barrier.wait();
    barrier.wait();
    let foreign = worker.join().map_err(|_| "rename worker panicked")??;
    assert_ne!(
        (original.st_dev, original.st_ino),
        (foreign.st_dev, foreign.st_ino)
    );

    let result = (0..32).try_for_each(|_| cleanup_step(&mut stack).map(|_| ()));
    assert!(result.is_err(), "changed child identity must fail cleanup");
    let surviving = fs::symlink_metadata(directory.path().join("child"))?;
    assert_eq!(
        (surviving.dev(), surviving.ino()),
        (foreign.st_dev, foreign.st_ino)
    );
    assert_eq!(fs::read_dir(directory.path().join("child"))?.count(), 0);
    assert!(!directory.path().join("saved/owned").exists());
    assert!(directory.path().join("saved").is_dir());
    Ok(())
}

#[test]
fn given_retained_nested_tree_when_cleared_then_only_root_remains() -> TestResult {
    let directory = tempfile::tempdir()?;
    fs::create_dir_all(directory.path().join("first/second"))?;
    fs::write(directory.path().join("first/second/owned"), b"owned")?;
    fs::write(directory.path().join("top"), b"top")?;
    let root = rustix::fs::open(
        directory.path(),
        super::super::DIRECTORY_FLAGS,
        rustix::fs::Mode::empty(),
    )?;
    super::clear_owned(&root)?;
    assert_eq!(fs::read_dir(directory.path())?.count(), 0);
    Ok(())
}

use std::os::fd::AsFd;
