use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use minegr::config::{Config, LoadedConfig, PropertyValue, load_config};
use minegr::config_path::{ConfigPath, resolve_existing_path};
use minegr::sync::{StoppedServerProbe, run_sync};

#[test]
fn sync_replaces_the_complete_property_table_with_documented_types() {
    let fixture = Fixture::new();
    fs::write(
        fixture.root().join("server.properties"),
        "server-port=25570\nwhite-list=true\nlevel-name=overworld\nview-distance=12\nonline-mode=true\n",
    )
    .unwrap();

    let output = run_sync(&fixture.path, &fixture.loaded, &FakeStopped(false))
        .expect("synchronization succeeds");

    let loaded = load_config(&fixture.path).unwrap().config;
    assert_eq!(loaded.minegr, fixture.loaded.config.minegr);
    assert_eq!(loaded.java, fixture.loaded.config.java);
    assert_eq!(
        loaded.minecraft.platform,
        fixture.loaded.config.minecraft.platform
    );
    assert_eq!(
        loaded.minecraft.version,
        fixture.loaded.config.minecraft.version
    );
    assert_eq!(
        loaded.minecraft.checksum,
        fixture.loaded.config.minecraft.checksum
    );
    assert_eq!(loaded.minecraft.eula, fixture.loaded.config.minecraft.eula);
    assert_eq!(
        loaded.minecraft.properties,
        std::collections::BTreeMap::from([
            (
                "level-name".to_owned(),
                PropertyValue::String("overworld".to_owned()),
            ),
            (
                "online-mode".to_owned(),
                PropertyValue::String("true".to_owned()),
            ),
            ("server-port".to_owned(), PropertyValue::Integer(25_570)),
            (
                "view-distance".to_owned(),
                PropertyValue::String("12".to_owned()),
            ),
            ("white-list".to_owned(), PropertyValue::Boolean(true)),
        ])
    );
    assert!(!loaded.minecraft.properties.contains_key("removed"));
    assert_eq!(
        output.stdout(),
        format!("Configuration synchronized: {}\n", fixture.path.as_str())
    );
}

#[test]
fn sync_preserves_text_outside_the_replaced_table() {
    let fixture = Fixture::new();
    fs::write(
        fixture.root().join("server.properties"),
        "server-port=25565\n",
    )
    .unwrap();

    run_sync(&fixture.path, &fixture.loaded, &FakeStopped(false)).unwrap();

    let source = fs::read_to_string(fixture.path.as_path()).unwrap();
    assert!(source.starts_with("# keep top-level comment\n"));
    assert!(source.contains("jvm_args = [\"-Xms2G\", \"-Xmx2G\"] # keep Java comment"));
    assert!(source.contains("[minecraft.properties]\nserver-port = 25565\n"));
}

#[test]
fn sync_parses_every_property_before_rewriting_configuration() {
    let fixture = Fixture::new();
    fs::write(
        fixture.root().join("server.properties"),
        "server-port=25570\nmotd=first\nmotd=second\n",
    )
    .unwrap();
    let before = fs::read(fixture.path.as_path()).unwrap();

    let error = run_sync(&fixture.path, &fixture.loaded, &FakeStopped(false))
        .expect_err("duplicate property fails");

    assert!(error.to_string().contains("duplicate property `motd`"));
    assert_eq!(fs::read(fixture.path.as_path()).unwrap(), before);
}

#[test]
fn sync_rejects_invalid_recognized_values_without_rewriting() {
    for source in [
        "server-port=0\n",
        "server-port=65536\n",
        "white-list=yes\n",
        "level-name=\n",
    ] {
        let fixture = Fixture::new();
        fs::write(fixture.root().join("server.properties"), source).unwrap();
        let before = fs::read(fixture.path.as_path()).unwrap();

        let error = run_sync(&fixture.path, &fixture.loaded, &FakeStopped(false))
            .expect_err("invalid recognized value fails");

        assert!(error.to_string().to_ascii_lowercase().contains("invalid"));
        assert_eq!(fs::read(fixture.path.as_path()).unwrap(), before);
    }
}

#[test]
fn sync_checks_stopped_state_before_reading_or_writing() {
    let fixture = Fixture::new();
    let before = fs::read(fixture.path.as_path()).unwrap();

    let error = run_sync(&fixture.path, &fixture.loaded, &FakeStopped(true))
        .expect_err("running server fails");

    assert!(error.to_string().contains("running"));
    assert!(!fixture.root().join("server.properties").exists());
    assert_eq!(fs::read(fixture.path.as_path()).unwrap(), before);
}

#[test]
fn sync_probe_error_blocks_reading_and_writing() {
    let fixture = Fixture::new();
    let before = fs::read(fixture.path.as_path()).unwrap();

    let error = run_sync(&fixture.path, &fixture.loaded, &FailingStoppedProbe)
        .expect_err("an unavailable stopped-state check must fail closed");

    assert!(
        error
            .to_string()
            .contains("Failed to verify that the server is stopped")
    );
    assert!(error.to_string().contains("probe unavailable"));
    assert!(!fixture.root().join("server.properties").exists());
    assert_eq!(fs::read(fixture.path.as_path()).unwrap(), before);
}

#[test]
fn sync_rejects_a_symlinked_server_properties_source() {
    let fixture = Fixture::new();
    let outside = fixture.root().join("outside.properties");
    fs::write(&outside, "server-port=25570\n").unwrap();
    symlink(&outside, fixture.root().join("server.properties")).unwrap();
    let before = fs::read(fixture.path.as_path()).unwrap();

    let error = run_sync(&fixture.path, &fixture.loaded, &FakeStopped(false))
        .expect_err("symlink source fails");

    assert!(error.to_string().contains("not a regular file"));
    assert_eq!(fs::read(fixture.path.as_path()).unwrap(), before);
}

#[test]
fn sync_requires_server_properties_and_leaves_configuration_unchanged() {
    let fixture = Fixture::new();
    let before = fs::read(fixture.path.as_path()).unwrap();

    let error = run_sync(&fixture.path, &fixture.loaded, &FakeStopped(false))
        .expect_err("missing properties fails");

    assert!(error.to_string().contains("does not exist"));
    assert_eq!(fs::read(fixture.path.as_path()).unwrap(), before);
}

#[test]
fn sync_creates_the_properties_table_when_valid_configuration_omits_it() {
    let directory = tempfile::tempdir().unwrap();
    let source = CONFIG
        .split("[minecraft.properties]")
        .next()
        .unwrap()
        .trim_end();
    fs::write(directory.path().join("minegr.toml"), format!("{source}\n")).unwrap();
    fs::write(
        directory.path().join("server.properties"),
        "server-port=25565\n",
    )
    .unwrap();
    let path = resolve_existing_path(Path::new("minegr.toml"), directory.path()).unwrap();
    let loaded = load_config(&path).unwrap();

    run_sync(&path, &loaded, &FakeStopped(false)).expect("missing table is created");

    assert_eq!(
        load_config(&path).unwrap().config.minecraft.properties,
        std::collections::BTreeMap::from([(
            "server-port".to_owned(),
            PropertyValue::Integer(25_565),
        )])
    );
}

#[test]
fn sync_rejects_configuration_replaced_after_validation() {
    let fixture = Fixture::new();
    let loaded = load_config(&fixture.path).unwrap();
    fs::write(
        fixture.root().join("server.properties"),
        "server-port=25570\n",
    )
    .unwrap();
    let replacement = CONFIG.replace("survival", "replacement");
    let staged = fixture.root().join("replacement.toml");
    fs::write(&staged, &replacement).unwrap();
    fs::rename(&staged, fixture.path.as_path()).unwrap();

    let error = run_sync(&fixture.path, &loaded, &FakeStopped(false))
        .expect_err("replacement after validation must fail");

    assert!(error.to_string().contains("changed"));
    assert_eq!(
        fs::read_to_string(fixture.path.as_path()).unwrap(),
        replacement
    );
}

#[test]
fn sync_rejects_configuration_edited_in_place_after_validation() {
    let fixture = Fixture::new();
    let loaded = load_config(&fixture.path).unwrap();
    fs::write(
        fixture.root().join("server.properties"),
        "server-port=25570\n",
    )
    .unwrap();
    let edited = CONFIG.replace("survival", "in-place edit");
    fs::write(fixture.path.as_path(), &edited).unwrap();

    let error = run_sync(&fixture.path, &loaded, &FakeStopped(false))
        .expect_err("in-place edit after validation must fail");

    assert!(error.to_string().contains("changed"));
    assert_eq!(fs::read_to_string(fixture.path.as_path()).unwrap(), edited);
}

struct Fixture {
    _directory: tempfile::TempDir,
    path: ConfigPath,
    loaded: LoadedConfig,
}

impl Fixture {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        fs::write(directory.path().join("minegr.toml"), CONFIG).unwrap();
        let path = resolve_existing_path(Path::new("minegr.toml"), directory.path()).unwrap();
        let loaded = load_config(&path).unwrap();
        Self {
            _directory: directory,
            path,
            loaded,
        }
    }

    fn root(&self) -> &Path {
        self.path.as_path().parent().unwrap()
    }
}

struct FakeStopped(bool);

impl StoppedServerProbe for FakeStopped {
    fn server_running(
        &self,
        _path: &ConfigPath,
        _config: &Config,
        _properties: &std::collections::BTreeMap<String, PropertyValue>,
    ) -> Result<bool, String> {
        Ok(self.0)
    }
}

struct FailingStoppedProbe;

impl StoppedServerProbe for FailingStoppedProbe {
    fn server_running(
        &self,
        _path: &ConfigPath,
        _config: &Config,
        _properties: &std::collections::BTreeMap<String, PropertyValue>,
    ) -> Result<bool, String> {
        Err("probe unavailable".to_owned())
    }
}

const CONFIG: &str = r#"# keep top-level comment
[minegr]
config_version = 1
uuid = "018f0000-0000-7000-8000-000000000000"
name = "survival"

[minecraft]
platform = "vanilla"
version = "1.21.8"
checksum = "sha1:1e5dcbb59b753cb1d46e234d8f6180285b8b86ad"
eula = true

[java]
jvm_args = ["-Xms2G", "-Xmx2G"] # keep Java comment
server_args = ["nogui"]

[minecraft.properties]
server-port = 25565
white-list = false
removed = "gone"
"#;
