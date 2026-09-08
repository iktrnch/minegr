use std::fs;
use std::io::{self, Cursor};
use std::path::{Path, PathBuf};

use minegr::config::{
    ConfigLoadOps, LoadMetadata, Platform, PropertyValue, load_config, load_config_with,
};
use minegr::config_path::{resolve_creation_path, resolve_existing_path};

const EXAMPLE: &str = r#"
[minegr]
config_version = 1
uuid = "018f0000-0000-7000-8000-000000000000"
name = "survival"

[minecraft]
platform = "paper"
version = "26.2"
build = 42
checksum = "sha256:0123456789abcdef"
eula = true

[java]
executable = "java"
jvm_args = ["-Xms2G", "-Xmx2G"]
server_args = ["nogui"]

[minecraft.properties]
server-port = 25565
white-list = true
motd = "A server"
rate-limit = 0.5
"#;

fn write_config(directory: &Path, contents: &str) -> PathBuf {
    let path = directory.join("minegr.toml");
    fs::write(&path, contents).expect("fixture should be writable");
    path
}

#[test]
fn existing_relative_path_is_canonicalized_without_parent_search() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let nested = temp.path().join("nested");
    fs::create_dir(&nested).expect("nested directory");
    write_config(&nested, EXAMPLE);

    let resolved = resolve_existing_path(Path::new("nested/minegr.toml"), temp.path())
        .expect("existing path should resolve");

    assert_eq!(
        resolved.as_path(),
        nested
            .canonicalize()
            .expect("fixture directory should canonicalize")
            .join("minegr.toml")
    );
}

#[test]
fn creation_path_canonicalizes_existing_parent() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let nested = temp.path().join("nested");
    fs::create_dir(&nested).expect("nested directory");

    let resolved = resolve_creation_path(Path::new("nested/custom.toml"), temp.path())
        .expect("creation path should resolve");

    assert_eq!(
        resolved.as_path(),
        nested
            .canonicalize()
            .expect("fixture directory should canonicalize")
            .join("custom.toml")
    );
}

#[test]
fn documented_configuration_deserializes_to_typed_values() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let path = write_config(temp.path(), EXAMPLE);
    let path = resolve_existing_path(&path, temp.path()).expect("path should resolve");

    let loaded = load_config(&path).expect("documented configuration should load");

    assert_eq!(loaded.config.minegr.name, "survival");
    assert_eq!(loaded.config.minecraft.platform, Platform::Paper);
    assert_eq!(loaded.config.minecraft.build, Some(42));
    assert_eq!(
        loaded.config.minecraft.properties["server-port"],
        PropertyValue::Integer(25_565)
    );
    assert_eq!(
        loaded.config.minecraft.properties["white-list"],
        PropertyValue::Boolean(true)
    );
    assert_eq!(
        loaded.config.minecraft.properties["rate-limit"],
        PropertyValue::Float(0.5)
    );
}

#[test]
fn unknown_keys_outside_minecraft_properties_are_rejected() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let path = write_config(
        temp.path(),
        &EXAMPLE.replace(
            "server_args = [\"nogui\"]",
            "server_args = [\"nogui\"]\nunexpected = true",
        ),
    );
    let path = resolve_existing_path(&path, temp.path()).expect("path should resolve");

    let error = load_config(&path).expect_err("unknown key should fail");

    assert!(error.to_string().contains("unknown field `unexpected`"));
}

#[test]
fn unsupported_config_version_is_rejected() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let path = write_config(
        temp.path(),
        &EXAMPLE.replace("config_version = 1", "config_version = 2"),
    );
    let path = resolve_existing_path(&path, temp.path()).expect("path should resolve");

    let error = load_config(&path).expect_err("unsupported version should fail");

    assert!(error.to_string().contains("unsupported config_version 2"));
}

#[cfg(unix)]
#[test]
fn symlink_configuration_is_rejected_as_non_regular() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().expect("temporary directory");
    let target = temp.path().join("target.toml");
    fs::write(&target, EXAMPLE).expect("fixture should be writable");
    let selected = temp.path().join("minegr.toml");
    symlink(&target, &selected).expect("symlink fixture");
    let path = resolve_existing_path(&selected, temp.path()).expect("path should resolve");

    let error = load_config(&path).expect_err("symlink configuration must be rejected");

    assert!(error.to_string().contains("not a regular file"));
}

#[cfg(unix)]
#[test]
fn non_utf8_creation_path_is_rejected() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let temp = tempfile::tempdir().expect("temporary directory");
    let selected = PathBuf::from(OsString::from_vec(vec![b'b', b'a', b'd', 0xff]));

    let error = resolve_creation_path(&selected, temp.path()).expect_err("path is not UTF-8");

    assert!(
        error
            .to_string()
            .starts_with("Configuration path is not valid UTF-8:")
    );
}

struct RaceLoadOps {
    opened: LoadMetadata,
    selected: LoadMetadata,
}

impl ConfigLoadOps for RaceLoadOps {
    type File = Cursor<Vec<u8>>;

    fn open(&mut self, _path: &Path) -> io::Result<Self::File> {
        Ok(Cursor::new(EXAMPLE.as_bytes().to_vec()))
    }

    fn metadata(&mut self, _file: &Self::File) -> io::Result<LoadMetadata> {
        Ok(self.opened)
    }

    fn inspect_selected(&mut self, _path: &Path) -> io::Result<LoadMetadata> {
        Ok(self.selected)
    }

    fn current_uid(&self) -> u32 {
        1000
    }
}

fn regular_metadata(owner_uid: u32, inode: u64) -> LoadMetadata {
    LoadMetadata {
        regular: true,
        owner_uid,
        mode: 0o100600,
        device_id: 1,
        inode,
    }
}

#[test]
fn loader_authorizes_metadata_from_the_open_file() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let path = resolve_creation_path(Path::new("minegr.toml"), temp.path())
        .expect("fixture path should resolve");
    let mut operations = RaceLoadOps {
        opened: regular_metadata(2000, 2),
        selected: regular_metadata(1000, 1),
    };

    let error = load_config_with(&mut operations, &path)
        .expect_err("foreign opened file must not inherit selected-path authorization");

    assert!(error.to_string().contains("owned by user 2000"));
}

#[test]
fn loader_rejects_a_target_changed_between_resolution_and_open() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let path = resolve_creation_path(Path::new("minegr.toml"), temp.path())
        .expect("fixture path should resolve");
    let mut operations = RaceLoadOps {
        opened: regular_metadata(1000, 2),
        selected: regular_metadata(1000, 1),
    };

    let error =
        load_config_with(&mut operations, &path).expect_err("identity mismatch must fail closed");

    assert!(
        error
            .to_string()
            .contains("changed while it was being loaded")
    );
}
