use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::Path;

use minegr::config::{
    Config, JavaConfig, MinecraftConfig, MinegrConfig, Platform, PropertyValue, load_config,
};
use minegr::config_path::{ConfigPath, resolve_creation_path, resolve_existing_path};
use minegr::config_write::{
    AtomicWriteOps, PublishError, TargetKind, TargetMetadata, WriteMode, write_config,
    write_config_with,
};

fn config(name: &str) -> Config {
    Config {
        minegr: MinegrConfig {
            config_version: 1,
            uuid: Some("018f0000-0000-7000-8000-000000000000".to_owned()),
            name: name.to_owned(),
        },
        minecraft: MinecraftConfig {
            platform: Platform::Vanilla,
            version: "1.21.8".to_owned(),
            build: None,
            loader: None,
            installer: None,
            checksum: Some("sha1:0123456789abcdef".to_owned()),
            eula: true,
            properties: BTreeMap::from([(
                "server-port".to_owned(),
                PropertyValue::Integer(25_565),
            )]),
        },
        java: JavaConfig {
            executable: None,
            jvm_args: vec!["-Xms2G".to_owned(), "-Xmx2G".to_owned()],
            server_args: vec!["nogui".to_owned()],
        },
    }
}

#[test]
fn create_publishes_owner_only_configuration_that_round_trips() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let path = resolve_creation_path(Path::new("minegr.toml"), temp.path())
        .expect("creation path should resolve");
    let expected = config("survival");

    write_config(&path, &expected, WriteMode::Create).expect("configuration should be created");

    let metadata = fs::metadata(path.as_path()).expect("published configuration metadata");
    assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
    let existing = resolve_existing_path(path.as_path(), temp.path()).expect("published path");
    assert_eq!(
        load_config(&existing).expect("configuration loads").config,
        expected
    );
    assert_eq!(temporary_files(temp.path()), 0);
}

#[test]
fn create_never_replaces_an_existing_target() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let target = temp.path().join("minegr.toml");
    fs::write(&target, "original").expect("fixture should be writable");
    let path = resolve_existing_path(&target, temp.path()).expect("existing path should resolve");

    write_config(&path, &config("replacement"), WriteMode::Create)
        .expect_err("create must not replace a target");

    assert_eq!(
        fs::read_to_string(target).expect("target remains"),
        "original"
    );
    assert_eq!(temporary_files(temp.path()), 0);
}

#[test]
fn replace_atomically_publishes_the_new_owner_only_configuration() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let path = resolve_creation_path(Path::new("minegr.toml"), temp.path())
        .expect("creation path should resolve");
    write_config(&path, &config("old"), WriteMode::Create).expect("initial configuration");
    let path = resolve_existing_path(path.as_path(), temp.path()).expect("existing path");
    let expected = config("new");

    write_config(&path, &expected, WriteMode::Replace).expect("replacement should succeed");

    assert_eq!(
        load_config(&path).expect("replacement loads").config,
        expected
    );
    assert_eq!(
        fs::metadata(path.as_path())
            .expect("replacement metadata")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    assert_eq!(temporary_files(temp.path()), 0);
}

#[test]
fn replace_rejects_a_symlink_target() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let outside = temp.path().join("outside");
    fs::write(&outside, "outside").expect("fixture should be writable");
    let selected = temp.path().join("minegr.toml");
    symlink(&outside, &selected).expect("symlink fixture");
    let path = resolve_creation_path(&selected, temp.path()).expect("symlink resolves");

    write_config(&path, &config("replacement"), WriteMode::Replace)
        .expect_err("symlink target must be rejected");

    assert_eq!(
        fs::read_to_string(outside).expect("outside remains"),
        "outside"
    );
}

fn temporary_files(directory: &Path) -> usize {
    fs::read_dir(directory)
        .expect("directory should be readable")
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().ends_with(".tmp"))
        .count()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FailurePoint {
    None,
    Permissions,
    Write,
    FileSync,
    Rename,
    AfterExchange,
    ParentSync,
}

#[derive(Default)]
struct FakeFile {
    fail_write: bool,
}

impl Write for FakeFile {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.fail_write {
            Err(io::Error::other("injected write failure"))
        } else {
            Ok(bytes.len())
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct FakeOps {
    failure: FailurePoint,
    target: Option<TargetMetadata>,
    target_at_publish: Option<Option<TargetMetadata>>,
    fail_cleanup: bool,
    fail_parent_sync: bool,
    parent_synced: bool,
    displaced_retained: bool,
    temp_exists: bool,
    target_published: bool,
}

impl FakeOps {
    fn failing_at(failure: FailurePoint) -> Self {
        Self {
            failure,
            target: None,
            target_at_publish: None,
            fail_cleanup: false,
            fail_parent_sync: false,
            parent_synced: false,
            displaced_retained: false,
            temp_exists: false,
            target_published: false,
        }
    }
}

impl AtomicWriteOps for FakeOps {
    type File = FakeFile;

    fn inspect_target(&mut self, _path: &Path) -> io::Result<Option<TargetMetadata>> {
        Ok(self.target)
    }

    fn create_temp(&mut self, _path: &Path) -> io::Result<Self::File> {
        self.temp_exists = true;
        Ok(FakeFile {
            fail_write: self.failure == FailurePoint::Write,
        })
    }

    fn set_temp_permissions(&mut self, _file: &mut Self::File) -> io::Result<()> {
        if self.failure == FailurePoint::Permissions {
            Err(io::Error::other("injected permission failure"))
        } else {
            Ok(())
        }
    }

    fn sync_file(&mut self, _file: &mut Self::File) -> io::Result<()> {
        if self.failure == FailurePoint::FileSync {
            Err(io::Error::other("injected file sync failure"))
        } else {
            Ok(())
        }
    }

    fn publish(
        &mut self,
        _from: &Path,
        _to: &Path,
        mode: WriteMode,
        expected: Option<TargetMetadata>,
    ) -> Result<(), PublishError> {
        if self.failure == FailurePoint::Rename {
            Err(PublishError::BeforePublication(io::Error::other(
                "injected rename failure",
            )))
        } else if self.failure == FailurePoint::AfterExchange {
            self.target_published = true;
            self.displaced_retained = true;
            Err(PublishError::AfterExchange(io::Error::other(
                "injected post-exchange failure",
            )))
        } else if mode == WriteMode::Replace
            && self.target_at_publish.unwrap_or(self.target) != expected
        {
            self.target_published = false;
            self.displaced_retained = false;
            Err(PublishError::RolledBack(io::Error::other(
                "target changed during publication",
            )))
        } else {
            self.target_published = true;
            if mode == WriteMode::Replace {
                self.displaced_retained = true;
            } else {
                self.temp_exists = false;
            }
            Ok(())
        }
    }

    fn remove_temp(&mut self, _path: &Path) -> io::Result<()> {
        if self.fail_cleanup {
            return Err(io::Error::other("injected cleanup failure"));
        }
        self.temp_exists = false;
        self.displaced_retained = false;
        Ok(())
    }

    fn sync_parent(&mut self, _path: &Path) -> io::Result<()> {
        self.parent_synced = true;
        if self.failure == FailurePoint::ParentSync || self.fail_parent_sync {
            Err(io::Error::other("injected parent sync failure"))
        } else {
            Ok(())
        }
    }

    fn current_uid(&self) -> u32 {
        1000
    }
}

fn fake_path() -> ConfigPath {
    let temp = tempfile::tempdir().expect("temporary directory");
    resolve_creation_path(Path::new("minegr.toml"), temp.path())
        .expect("creation path should resolve")
}

#[test]
fn pre_rename_failures_remove_the_temporary_file() {
    for failure in [
        FailurePoint::Permissions,
        FailurePoint::Write,
        FailurePoint::FileSync,
        FailurePoint::Rename,
    ] {
        let mut operations = FakeOps::failing_at(failure);

        write_config_with(
            &mut operations,
            &fake_path(),
            &config("survival"),
            WriteMode::Create,
        )
        .expect_err("injected operation should fail");

        assert!(
            !operations.temp_exists,
            "temporary file remained after {failure:?}"
        );
        assert!(!operations.target_published);
    }
}

#[test]
fn parent_sync_failure_reports_failure_after_atomic_publication() {
    let mut operations = FakeOps::failing_at(FailurePoint::ParentSync);

    write_config_with(
        &mut operations,
        &fake_path(),
        &config("survival"),
        WriteMode::Create,
    )
    .expect_err("injected parent sync should fail");

    assert!(!operations.temp_exists);
    assert!(operations.target_published);
}

#[test]
fn replace_rejects_foreign_owned_regular_target_before_writing() {
    let mut operations = FakeOps::failing_at(FailurePoint::None);
    operations.target = Some(TargetMetadata {
        kind: TargetKind::Regular,
        owner_uid: 2000,
        device_id: 1,
        inode: 1,
    });

    write_config_with(
        &mut operations,
        &fake_path(),
        &config("survival"),
        WriteMode::Replace,
    )
    .expect_err("foreign target should be rejected");

    assert!(!operations.temp_exists);
    assert!(!operations.target_published);
}

#[test]
fn replace_does_not_overwrite_a_target_changed_before_publication() {
    let original = TargetMetadata {
        kind: TargetKind::Regular,
        owner_uid: 1000,
        device_id: 1,
        inode: 1,
    };
    let replacement = TargetMetadata {
        kind: TargetKind::NonRegular,
        owner_uid: 2000,
        device_id: 1,
        inode: 2,
    };
    let mut operations = FakeOps::failing_at(FailurePoint::None);
    operations.target = Some(original);
    operations.target_at_publish = Some(Some(replacement));

    write_config_with(
        &mut operations,
        &fake_path(),
        &config("survival"),
        WriteMode::Replace,
    )
    .expect_err("changed target must not be overwritten");

    assert!(operations.temp_exists);
    assert!(!operations.target_published);
    assert!(!operations.displaced_retained);
    assert!(operations.parent_synced);
}

#[test]
fn replace_does_not_turn_into_creation_when_target_disappears() {
    let original = TargetMetadata {
        kind: TargetKind::Regular,
        owner_uid: 1000,
        device_id: 1,
        inode: 1,
    };
    let mut operations = FakeOps::failing_at(FailurePoint::None);
    operations.target = Some(original);
    operations.target_at_publish = Some(None);

    write_config_with(
        &mut operations,
        &fake_path(),
        &config("survival"),
        WriteMode::Replace,
    )
    .expect_err("missing target must not turn replacement into creation");

    assert!(operations.temp_exists);
    assert!(!operations.target_published);
    assert!(!operations.displaced_retained);
    assert!(operations.parent_synced);
}

#[test]
fn successful_rollback_reports_unpublished_replacement_not_displaced_target() {
    let original = TargetMetadata {
        kind: TargetKind::Regular,
        owner_uid: 1000,
        device_id: 1,
        inode: 1,
    };
    let changed = TargetMetadata {
        kind: TargetKind::Regular,
        owner_uid: 1000,
        device_id: 1,
        inode: 2,
    };
    let mut operations = FakeOps::failing_at(FailurePoint::None);
    operations.target = Some(original);
    operations.target_at_publish = Some(Some(changed));

    let error = write_config_with(
        &mut operations,
        &fake_path(),
        &config("survival"),
        WriteMode::Replace,
    )
    .expect_err("changed target must roll back");

    assert!(!operations.target_published);
    assert!(operations.temp_exists);
    assert!(!operations.displaced_retained);
    assert!(operations.parent_synced);
    assert!(error.to_string().contains("original target restored"));
    assert!(
        error
            .to_string()
            .contains("unpublished replacement retained")
    );
    assert!(!error.to_string().contains("displaced entry retained"));
}

#[test]
fn displaced_removal_failure_still_syncs_parent_and_reports_recovery_path() {
    let mut operations = FakeOps::failing_at(FailurePoint::None);
    operations.target = Some(TargetMetadata {
        kind: TargetKind::Regular,
        owner_uid: 1000,
        device_id: 1,
        inode: 1,
    });
    operations.fail_cleanup = true;

    let error = write_config_with(
        &mut operations,
        &fake_path(),
        &config("survival"),
        WriteMode::Replace,
    )
    .expect_err("displaced target removal failure must be reported");

    assert!(operations.target_published);
    assert!(operations.temp_exists);
    assert!(operations.displaced_retained);
    assert!(operations.parent_synced);
    assert!(error.to_string().contains("replaced file at"));
    assert!(error.to_string().contains(".tmp"));
}

#[test]
fn post_exchange_failure_retains_displaced_entry_and_syncs_parent() {
    let mut operations = FakeOps::failing_at(FailurePoint::AfterExchange);
    operations.target = Some(TargetMetadata {
        kind: TargetKind::Regular,
        owner_uid: 1000,
        device_id: 1,
        inode: 1,
    });

    let error = write_config_with(
        &mut operations,
        &fake_path(),
        &config("survival"),
        WriteMode::Replace,
    )
    .expect_err("post-exchange verification failure must be reported");

    assert!(operations.target_published);
    assert!(operations.temp_exists);
    assert!(operations.displaced_retained);
    assert!(operations.parent_synced);
    assert!(error.to_string().contains("displaced entry retained"));
}

#[test]
fn post_exchange_failure_reports_parent_sync_failure_without_cleanup() {
    let mut operations = FakeOps::failing_at(FailurePoint::AfterExchange);
    operations.fail_parent_sync = true;
    operations.target = Some(TargetMetadata {
        kind: TargetKind::Regular,
        owner_uid: 1000,
        device_id: 1,
        inode: 1,
    });

    let error = write_config_with(
        &mut operations,
        &fake_path(),
        &config("survival"),
        WriteMode::Replace,
    )
    .expect_err("both post-exchange failures must be reported");

    assert!(operations.temp_exists);
    assert!(operations.displaced_retained);
    assert!(operations.parent_synced);
    assert!(
        error
            .to_string()
            .contains("directory synchronization also failed")
    );
}

#[test]
fn permission_and_cleanup_failures_are_both_reported() {
    let mut operations = FakeOps::failing_at(FailurePoint::Permissions);
    operations.fail_cleanup = true;

    let error = write_config_with(
        &mut operations,
        &fake_path(),
        &config("survival"),
        WriteMode::Create,
    )
    .expect_err("permission and cleanup failures must be reported");

    assert!(operations.temp_exists);
    assert!(error.to_string().contains("permission failure"));
    assert!(error.to_string().contains("cleanup failure"));
    assert!(error.to_string().contains(".tmp"));
}
