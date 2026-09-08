use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use clap::Parser;
use minegr::artifact::{MinecraftVersion, PlatformAvailability, ResolvedArtifact};
use minegr::cli::{Cli, InitArgs};
use minegr::config::{
    Config, JavaConfig, LoadedConfig, MinecraftConfig, MinegrConfig, Platform, PropertyValue,
    load_config,
};
use minegr::config_path::{ConfigPath, resolve_existing_path};
use minegr::config_write::{WriteMode, write_config};
use minegr::init::{
    ArtifactProvider, InitHostValidator, JavaRuntime, RunningInstanceProbe, run_existing_init,
};
use minegr::managed_files::materialize_managed_bytes;

#[test]
fn existing_init_materializes_only_missing_files_from_pinned_configuration() {
    let fixture = Fixture::new();
    let artifacts = ExistingArtifacts::new();

    let output = run_existing_init(
        &normal_args(),
        &fixture.path,
        &fixture.loaded,
        &artifacts,
        &FakeJava,
        &FakeRunning(false),
        &FakeHost,
    )
    .expect("existing instance materializes");

    assert_eq!(
        fs::read(fixture.root().join("server.jar")).unwrap(),
        b"artifact"
    );
    assert_eq!(
        fs::read_to_string(fixture.root().join("server.properties")).unwrap(),
        "server-port=25565\n"
    );
    assert_eq!(
        fs::read_to_string(fixture.root().join("eula.txt")).unwrap(),
        "eula=true\n"
    );
    assert_eq!(artifacts.resolve_calls.get(), 1);
    assert_eq!(artifacts.download_calls.get(), 1);
    assert!(output.stdout().starts_with("Configuration: "));
}

#[test]
fn existing_init_accepts_identical_files_without_rewriting_them() {
    let fixture = Fixture::new();
    let artifacts = ExistingArtifacts::new();
    run_existing_init(
        &normal_args(),
        &fixture.path,
        &fixture.loaded,
        &artifacts,
        &FakeJava,
        &FakeRunning(false),
        &FakeHost,
    )
    .expect("initial materialization");
    let inodes = ["server.jar", "server.properties", "eula.txt"]
        .map(|name| fs::metadata(fixture.root().join(name)).unwrap().ino());

    run_existing_init(
        &normal_args(),
        &fixture.path,
        &fixture.loaded,
        &artifacts,
        &FakeJava,
        &FakeRunning(false),
        &FakeHost,
    )
    .expect("identical materialization is a no-op");

    assert_eq!(
        ["server.jar", "server.properties", "eula.txt"]
            .map(|name| { fs::metadata(fixture.root().join(name)).unwrap().ino() }),
        inodes
    );
}

#[test]
fn existing_init_preflights_drift_before_creating_any_missing_file() {
    let fixture = Fixture::new();
    fs::write(
        fixture.root().join("server.properties"),
        "server-port=25566\n",
    )
    .unwrap();
    let artifacts = ExistingArtifacts::new();

    let error = run_existing_init(
        &normal_args(),
        &fixture.path,
        &fixture.loaded,
        &artifacts,
        &FakeJava,
        &FakeRunning(false),
        &FakeHost,
    )
    .expect_err("drift must fail");

    assert!(error.to_string().contains("server.properties"));
    assert!(error.to_string().contains("remove it before retrying"));
    assert_eq!(artifacts.resolve_calls.get(), 0);
    assert!(!fixture.root().join("server.jar").exists());
    assert!(!fixture.root().join("eula.txt").exists());
}

#[test]
fn uuid_regeneration_changes_only_uuid_and_preserves_surrounding_document_text() {
    let fixture = Fixture::new_with_source(
        r#"# keep this comment
[minegr]
config_version = 1
uuid = "018f0000-0000-7000-8000-000000000000" # identity
name = "survival"

[minecraft]
platform = "vanilla"
version = "1.21.8"
checksum = "sha1:1e5dcbb59b753cb1d46e234d8f6180285b8b86ad"
eula = true

[java]
jvm_args = ["-Xms2G", "-Xmx2G"]
server_args = ["nogui"]

[minecraft.properties]
server-port = 25565
"#,
    );
    let original = fixture.loaded.config.clone();
    let artifacts = ExistingArtifacts::new();

    run_existing_init(
        &uuid_args(),
        &fixture.path,
        &fixture.loaded,
        &artifacts,
        &FakeJava,
        &FakeRunning(false),
        &FakeHost,
    )
    .expect("UUID regeneration");

    let updated =
        load_config(&resolve_existing_path(fixture.path.as_path(), fixture.root()).unwrap())
            .unwrap()
            .config;
    assert_ne!(updated.minegr.uuid, original.minegr.uuid);
    let mut expected = original;
    expected.minegr.uuid = updated.minegr.uuid.clone();
    assert_eq!(updated, expected);
    let source = fs::read_to_string(fixture.path.as_path()).unwrap();
    assert!(source.starts_with("# keep this comment\n"));
    assert!(source.contains(" # identity"));
    assert_eq!(artifacts.resolve_calls.get(), 0);
    assert_eq!(artifacts.download_calls.get(), 0);
}

#[test]
fn uuid_regeneration_rejects_a_running_selected_instance_without_writing() {
    let fixture = Fixture::new();
    let before = fs::read(fixture.path.as_path()).unwrap();

    let error = run_existing_init(
        &uuid_args(),
        &fixture.path,
        &fixture.loaded,
        &ExistingArtifacts::new(),
        &FakeJava,
        &FakeRunning(true),
        &FakeHost,
    )
    .expect_err("running instance");

    assert!(error.to_string().contains("running"));
    assert_eq!(fs::read(fixture.path.as_path()).unwrap(), before);
}

#[test]
fn uuid_regeneration_rejects_configuration_replaced_after_loading() {
    let fixture = Fixture::new();
    let loaded = load_config(&fixture.path).unwrap();
    let replacement = CONFIG_REPLACEMENT.replace("replacement", "new path contents");
    let staged = fixture.root().join("replacement.toml");
    fs::write(&staged, &replacement).unwrap();
    fs::rename(&staged, fixture.path.as_path()).unwrap();

    let error = run_existing_init(
        &uuid_args(),
        &fixture.path,
        &loaded,
        &ExistingArtifacts::new(),
        &FakeJava,
        &FakeRunning(false),
        &FakeHost,
    )
    .expect_err("replacement after validation must fail");

    assert!(error.to_string().contains("changed"));
    assert_eq!(
        fs::read_to_string(fixture.path.as_path()).unwrap(),
        replacement
    );
}

#[test]
fn uuid_regeneration_rejects_configuration_edited_in_place_after_loading() {
    let fixture = Fixture::new();
    let loaded = load_config(&fixture.path).unwrap();
    let edited = CONFIG_REPLACEMENT.replace("replacement", "in-place edit");
    fs::write(fixture.path.as_path(), &edited).unwrap();

    let error = run_existing_init(
        &uuid_args(),
        &fixture.path,
        &loaded,
        &ExistingArtifacts::new(),
        &FakeJava,
        &FakeRunning(false),
        &FakeHost,
    )
    .expect_err("in-place edit after loading must fail");

    assert!(error.to_string().contains("changed"));
    assert_eq!(fs::read_to_string(fixture.path.as_path()).unwrap(), edited);
}

fn normal_args() -> InitArgs {
    args(&["minegr", "init"])
}

fn uuid_args() -> InitArgs {
    args(&["minegr", "init", "--uuid"])
}

fn args(values: &[&str]) -> InitArgs {
    match Cli::try_parse_from(values).unwrap().command {
        minegr::cli::Command::Init(arguments) => arguments,
        _ => unreachable!(),
    }
}

struct Fixture {
    _directory: tempfile::TempDir,
    path: ConfigPath,
    loaded: LoadedConfig,
}

impl Fixture {
    fn new() -> Self {
        let config = config();
        let directory = tempfile::tempdir().unwrap();
        let creation =
            minegr::config_path::resolve_creation_path(Path::new("minegr.toml"), directory.path())
                .unwrap();
        write_config(&creation, &config, WriteMode::Create).unwrap();
        let path = resolve_existing_path(creation.as_path(), directory.path()).unwrap();
        let loaded = load_config(&path).unwrap();
        Self {
            _directory: directory,
            path,
            loaded,
        }
    }

    fn new_with_source(source: &str) -> Self {
        let directory = tempfile::tempdir().unwrap();
        fs::write(directory.path().join("minegr.toml"), source).unwrap();
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

fn config() -> Config {
    Config {
        minegr: MinegrConfig {
            config_version: 1,
            uuid: Some("018f0000-0000-7000-8000-000000000000".to_owned()),
            name: "survival".to_owned(),
        },
        minecraft: MinecraftConfig {
            platform: Platform::Vanilla,
            version: "1.21.8".to_owned(),
            build: None,
            loader: None,
            installer: None,
            checksum: Some("sha1:1e5dcbb59b753cb1d46e234d8f6180285b8b86ad".to_owned()),
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

struct ExistingArtifacts {
    resolve_calls: std::cell::Cell<usize>,
    download_calls: std::cell::Cell<usize>,
}

impl ExistingArtifacts {
    fn new() -> Self {
        Self {
            resolve_calls: std::cell::Cell::new(0),
            download_calls: std::cell::Cell::new(0),
        }
    }
}

impl ArtifactProvider for ExistingArtifacts {
    fn versions(&self) -> Result<Vec<MinecraftVersion>, String> {
        unreachable!("existing init resolves pinned coordinates directly")
    }

    fn available_platforms(
        &self,
        _version: &MinecraftVersion,
    ) -> Result<PlatformAvailability, String> {
        unreachable!("existing init does not reselect a platform")
    }

    fn resolve(
        &self,
        _version: &MinecraftVersion,
        _platform: Platform,
    ) -> Result<ResolvedArtifact, String> {
        unreachable!("existing init does not select a new artifact")
    }

    fn resolve_pinned(&self, config: &MinecraftConfig) -> Result<ResolvedArtifact, String> {
        self.resolve_calls.set(self.resolve_calls.get() + 1);
        Ok(ResolvedArtifact {
            platform: config.platform,
            version: config.version.clone(),
            build: config.build,
            loader: config.loader.clone(),
            installer: config.installer.clone(),
            checksum: config.checksum.clone(),
            download_url: "https://example.invalid/server.jar".to_owned(),
            required_java_major: 21,
        })
    }

    fn download(&self, _artifact: &ResolvedArtifact, root: &Path) -> Result<(), String> {
        self.download_calls.set(self.download_calls.get() + 1);
        materialize_managed_bytes(root, "server.jar", b"artifact")
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
}

struct FakeJava;

impl JavaRuntime for FakeJava {
    fn major_version(&self, _executable: Option<&str>) -> Result<u32, String> {
        Ok(21)
    }
}

struct FakeHost;

impl InitHostValidator for FakeHost {
    fn validate(&self, _server_root: &Path, _config: &Config) -> Result<(), String> {
        Ok(())
    }
}

struct FakeRunning(bool);

impl RunningInstanceProbe for FakeRunning {
    fn selected_instance_running(
        &self,
        _path: &ConfigPath,
        _config: &Config,
    ) -> Result<bool, String> {
        Ok(self.0)
    }
}

const CONFIG_REPLACEMENT: &str = r#"[minegr]
config_version = 1
uuid = "018f0000-0000-7000-8000-000000000001"
name = "replacement"

[minecraft]
platform = "vanilla"
version = "1.21.8"
checksum = "sha1:1e5dcbb59b753cb1d46e234d8f6180285b8b86ad"
eula = true

[java]
jvm_args = ["-Xms2G", "-Xmx2G"]
server_args = ["nogui"]

[minecraft.properties]
server-port = 25565
"#;
