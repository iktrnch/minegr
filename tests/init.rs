use std::cell::{Cell, RefCell};
use std::fs;
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use clap::Parser;
use minegr::artifact::{
    MinecraftVersion, MinecraftVersionKind, PlatformAvailability, ResolvedArtifact,
};
use minegr::cli::{Cli, InitArgs};
use minegr::config::{Platform, load_config};
use minegr::config_path::resolve_creation_path;
use minegr::init::{
    ArtifactProvider, InitError, InitHostValidator, InitPrompter, JavaRuntime,
    SystemInitHostValidator, run_new_init,
};

#[test]
fn flag_driven_init_creates_a_reproducible_instance_with_documented_defaults() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let path =
        resolve_creation_path(Path::new("minegr.toml"), temp.path()).expect("configuration path");
    let args = init_args(&[
        "minegr",
        "init",
        "--minecraft-version",
        "1.21.8",
        "--platform",
        "vanilla",
        "--accept-eula",
        "--yes",
    ]);
    let artifacts = FakeArtifacts::successful();
    let java = FakeJava { major: 21 };
    let prompts = FakePrompter::non_interactive();

    let output = run_new_init(
        &args,
        &path,
        &prompts,
        &artifacts,
        &java,
        &FakeHost::passing(),
    )
    .expect("new instance should initialize");

    let existing = minegr::config_path::resolve_existing_path(path.as_path(), temp.path())
        .expect("created path");
    let config = load_config(&existing).expect("created config loads").config;
    assert_eq!(
        config.minegr.name,
        temp.path().file_name().unwrap().to_string_lossy()
    );
    let uuid = uuid::Uuid::parse_str(config.minegr.uuid.as_deref().expect("generated UUID"))
        .expect("valid UUID");
    assert_eq!(uuid.get_version_num(), 7);
    assert_eq!(config.minecraft.version, "1.21.8");
    assert_eq!(config.minecraft.platform, Platform::Vanilla);
    assert_eq!(config.java.jvm_args, ["-Xms2G", "-Xmx2G"]);
    assert_eq!(config.java.server_args, ["nogui"]);
    assert_eq!(
        config.minecraft.properties["server-port"],
        minegr::config::PropertyValue::Integer(25_565)
    );
    assert_eq!(
        fs::metadata(path.as_path())
            .expect("configuration metadata")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    assert_eq!(
        fs::read(temp.path().join("server.jar")).expect("server.jar"),
        b"artifact"
    );
    assert_eq!(
        fs::read_to_string(temp.path().join("server.properties")).expect("properties"),
        "server-port=25565\n"
    );
    assert_eq!(
        fs::read_to_string(temp.path().join("eula.txt")).expect("EULA"),
        "eula=true\n"
    );
    assert_eq!(
        output.stdout(),
        format!(
            "Configuration: {}\nStart it with: minegr start --config {}\n",
            path.as_str(),
            path.as_str()
        )
    );
    assert!(!prompts.was_used());
}

#[test]
fn flag_driven_init_preserves_custom_name_memory_and_port() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let path = resolve_creation_path(Path::new("minegr.toml"), temp.path()).unwrap();
    let args = init_args(&[
        "minegr",
        "init",
        "--name",
        "custom survival",
        "--minecraft-version",
        "1.21.8",
        "--platform",
        "vanilla",
        "--memory",
        "4G",
        "--port",
        "25570",
        "--accept-eula",
        "--yes",
    ]);

    run_new_init(
        &args,
        &path,
        &FakePrompter::non_interactive(),
        &FakeArtifacts::successful(),
        &FakeJava { major: 21 },
        &FakeHost::passing(),
    )
    .expect("custom flags initialize");

    let config = load_config(
        &minegr::config_path::resolve_existing_path(path.as_path(), temp.path()).unwrap(),
    )
    .unwrap()
    .config;
    assert_eq!(config.minegr.name, "custom survival");
    assert_eq!(config.java.jvm_args, ["-Xms4G", "-Xmx4G"]);
    assert_eq!(
        config.minecraft.properties["server-port"],
        minegr::config::PropertyValue::Integer(25_570)
    );
}

#[test]
fn interactive_init_collects_missing_fields_and_materializes_the_instance() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let path = resolve_creation_path(Path::new("minegr.toml"), temp.path()).unwrap();
    let args = init_args(&["minegr", "init"]);
    let prompts = FakePrompter::interactive_with_confirmation(true);

    run_new_init(
        &args,
        &path,
        &prompts,
        &FakeArtifacts::successful(),
        &FakeJava { major: 21 },
        &FakeHost::passing(),
    )
    .expect("interactive answers initialize");

    assert!(prompts.was_used());
    assert!(path.as_path().exists());
    assert!(temp.path().join("server.jar").exists());
    assert!(temp.path().join("server.properties").exists());
    assert!(temp.path().join("eula.txt").exists());
}

#[test]
fn non_interactive_init_names_every_required_missing_flag_before_network_access() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let path = resolve_creation_path(Path::new("minegr.toml"), temp.path()).unwrap();
    let args = init_args(&["minegr", "init"]);
    let artifacts = FakeArtifacts::successful();

    let error = run_new_init(
        &args,
        &path,
        &FakePrompter::non_interactive(),
        &artifacts,
        &FakeJava { major: 21 },
        &FakeHost::passing(),
    )
    .expect_err("missing non-interactive answers");

    let message = error.to_string();
    assert!(message.contains("--minecraft-version"));
    assert!(message.contains("--platform"));
    assert!(message.contains("--accept-eula"));
    assert!(message.contains("--yes"));
    assert_eq!(artifacts.calls.get(), 0);
    assert!(!path.as_path().exists());
}

#[test]
fn yes_never_implies_eula_acceptance() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let path = resolve_creation_path(Path::new("minegr.toml"), temp.path()).unwrap();
    let args = init_args(&[
        "minegr",
        "init",
        "--minecraft-version",
        "1.21.8",
        "--platform",
        "vanilla",
        "--yes",
    ]);

    let error = run_new_init(
        &args,
        &path,
        &FakePrompter::non_interactive(),
        &FakeArtifacts::successful(),
        &FakeJava { major: 21 },
        &FakeHost::passing(),
    )
    .expect_err("EULA acceptance remains required");

    assert!(error.to_string().contains("--accept-eula"));
    assert!(!path.as_path().exists());
}

#[test]
fn denied_final_confirmation_creates_nothing() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let path = resolve_creation_path(Path::new("minegr.toml"), temp.path()).unwrap();
    let args = init_args(&[
        "minegr",
        "init",
        "--minecraft-version",
        "1.21.8",
        "--platform",
        "vanilla",
        "--accept-eula",
    ]);
    let prompts = FakePrompter::interactive_with_confirmation(false);

    let error = run_new_init(
        &args,
        &path,
        &prompts,
        &FakeArtifacts::successful(),
        &FakeJava { major: 21 },
        &FakeHost::passing(),
    )
    .expect_err("creation denied");

    assert!(matches!(error, InitError::CreationDenied));
    assert!(
        prompts
            .summary
            .borrow()
            .contains("Minecraft: vanilla 1.21.8")
    );
    assert!(!path.as_path().exists());
}

#[test]
fn incompatible_java_is_rejected_before_configuration_creation() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let path = resolve_creation_path(Path::new("minegr.toml"), temp.path()).unwrap();
    let args = complete_args();

    let error = run_new_init(
        &args,
        &path,
        &FakePrompter::non_interactive(),
        &FakeArtifacts::successful(),
        &FakeJava { major: 17 },
        &FakeHost::passing(),
    )
    .expect_err("Java is too old");

    assert!(error.to_string().contains("requires Java 21"));
    assert!(!path.as_path().exists());
}

#[test]
fn invalid_memory_flag_fails_without_prompting_or_network_access() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let path = resolve_creation_path(Path::new("minegr.toml"), temp.path()).unwrap();
    let args = init_args(&[
        "minegr",
        "init",
        "--minecraft-version",
        "1.21.8",
        "--platform",
        "vanilla",
        "--memory",
        "0G",
        "--accept-eula",
        "--yes",
    ]);
    let prompts = FakePrompter::non_interactive();
    let artifacts = FakeArtifacts::successful();

    let error = run_new_init(
        &args,
        &path,
        &prompts,
        &artifacts,
        &FakeJava { major: 21 },
        &FakeHost::passing(),
    )
    .expect_err("invalid memory");

    assert!(error.to_string().contains("--memory"));
    assert!(!prompts.was_used());
    assert_eq!(artifacts.calls.get(), 0);
}

#[test]
fn materialization_failure_retains_the_valid_configuration() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let path = resolve_creation_path(Path::new("minegr.toml"), temp.path()).unwrap();
    let artifacts = FakeArtifacts {
        fail_download: true,
        ..FakeArtifacts::successful()
    };

    let error = run_new_init(
        &complete_args(),
        &path,
        &FakePrompter::non_interactive(),
        &artifacts,
        &FakeJava { major: 21 },
        &FakeHost::passing(),
    )
    .expect_err("download failure");

    assert!(error.to_string().contains("download failed"));
    let existing = minegr::config_path::resolve_existing_path(path.as_path(), temp.path()).unwrap();
    assert!(load_config(&existing).is_ok());
    assert!(!temp.path().join("server.jar").exists());
}

#[test]
fn init_host_validation_failure_prevents_configuration_creation() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let path = resolve_creation_path(Path::new("minegr.toml"), temp.path()).unwrap();

    let error = run_new_init(
        &complete_args(),
        &path,
        &FakePrompter::non_interactive(),
        &FakeArtifacts::successful(),
        &FakeJava { major: 21 },
        &FakeHost::failing("configured memory exceeds available memory"),
    )
    .expect_err("host validation must block creation");

    assert!(error.to_string().contains("available memory"));
    assert!(!path.as_path().exists());
}

#[test]
fn system_init_host_validation_rejects_an_occupied_configured_port() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let config = minegr::config::Config {
        minegr: minegr::config::MinegrConfig {
            config_version: 1,
            uuid: Some("018f0000-0000-7000-8000-000000000000".to_owned()),
            name: "occupied".to_owned(),
        },
        minecraft: minegr::config::MinecraftConfig {
            platform: Platform::Vanilla,
            version: "1.21.8".to_owned(),
            build: None,
            loader: None,
            installer: None,
            checksum: Some("sha1:0123456789abcdef".to_owned()),
            eula: true,
            properties: std::collections::BTreeMap::from([(
                "server-port".to_owned(),
                minegr::config::PropertyValue::Integer(i64::from(port)),
            )]),
        },
        java: minegr::config::JavaConfig {
            executable: None,
            jvm_args: vec!["-Xms1M".to_owned(), "-Xmx1M".to_owned()],
            server_args: vec!["nogui".to_owned()],
        },
    };

    let error = SystemInitHostValidator
        .validate(temp.path(), &config)
        .expect_err("occupied port must fail init host validation");

    assert!(error.contains("port is already occupied"));
    drop(listener);
}

#[test]
fn system_host_validation_reports_memory_and_port_together() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let mut config = valid_host_config(port);
    config.java.jvm_args = vec!["-Xmx999999T".to_owned()];

    let error = SystemInitHostValidator
        .validate(temp.path(), &config)
        .expect_err("independent unsafe host facts must all report");

    assert!(error.contains("memory"));
    assert!(error.contains("port"));
}

#[test]
fn system_host_validation_rejects_a_non_directory_server_root() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("not-a-root");
    fs::write(&root, "file").unwrap();

    let error = SystemInitHostValidator
        .validate(&root, &valid_host_config(25_565))
        .expect_err("server root must be a directory");

    assert!(error.contains("server root is not a directory"));
}

fn valid_host_config(port: u16) -> minegr::config::Config {
    minegr::config::Config {
        minegr: minegr::config::MinegrConfig {
            config_version: 1,
            uuid: Some("018f0000-0000-7000-8000-000000000000".to_owned()),
            name: "host-validation".to_owned(),
        },
        minecraft: minegr::config::MinecraftConfig {
            platform: Platform::Vanilla,
            version: "1.21.8".to_owned(),
            build: None,
            loader: None,
            installer: None,
            checksum: Some("sha1:0123456789abcdef".to_owned()),
            eula: true,
            properties: std::collections::BTreeMap::from([(
                "server-port".to_owned(),
                minegr::config::PropertyValue::Integer(i64::from(port)),
            )]),
        },
        java: minegr::config::JavaConfig {
            executable: None,
            jvm_args: vec!["-Xmx1M".to_owned()],
            server_args: vec!["nogui".to_owned()],
        },
    }
}

fn init_args(arguments: &[&str]) -> InitArgs {
    let cli = Cli::try_parse_from(arguments).expect("valid init arguments");
    match cli.command {
        minegr::cli::Command::Init(arguments) => arguments,
        _ => panic!("expected init command"),
    }
}

fn complete_args() -> InitArgs {
    init_args(&[
        "minegr",
        "init",
        "--minecraft-version",
        "1.21.8",
        "--platform",
        "vanilla",
        "--accept-eula",
        "--yes",
    ])
}

struct FakeArtifacts {
    calls: Cell<usize>,
    fail_download: bool,
}

impl FakeArtifacts {
    fn successful() -> Self {
        Self {
            calls: Cell::new(0),
            fail_download: false,
        }
    }

    fn touched(&self) {
        self.calls.set(self.calls.get() + 1);
    }
}

impl ArtifactProvider for FakeArtifacts {
    fn versions(&self) -> Result<Vec<MinecraftVersion>, String> {
        self.touched();
        Ok(vec![MinecraftVersion::new(
            "1.21.8",
            "https://example.invalid/version",
            MinecraftVersionKind::Release,
        )])
    }

    fn available_platforms(
        &self,
        _version: &MinecraftVersion,
    ) -> Result<PlatformAvailability, String> {
        self.touched();
        Ok(PlatformAvailability {
            vanilla: true,
            paper: false,
            fabric: false,
        })
    }

    fn resolve(
        &self,
        version: &MinecraftVersion,
        platform: Platform,
    ) -> Result<ResolvedArtifact, String> {
        self.touched();
        Ok(ResolvedArtifact {
            platform,
            version: version.id.clone(),
            build: None,
            loader: None,
            installer: None,
            checksum: Some("sha1:1e5dcbb59b753cb1d46e234d8f6180285b8b86ad".to_owned()),
            download_url: "https://example.invalid/server.jar".to_owned(),
            required_java_major: 21,
        })
    }

    fn resolve_pinned(
        &self,
        config: &minegr::config::MinecraftConfig,
    ) -> Result<ResolvedArtifact, String> {
        self.touched();
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

    fn download(&self, _artifact: &ResolvedArtifact, server_root: &Path) -> Result<(), String> {
        self.touched();
        if self.fail_download {
            Err("download failed".to_owned())
        } else {
            minegr::managed_files::materialize_managed_bytes(server_root, "server.jar", b"artifact")
                .map(|_| ())
                .map_err(|error| error.to_string())
        }
    }
}

struct FakeJava {
    major: u32,
}

struct FakeHost {
    failure: Option<&'static str>,
}

impl FakeHost {
    fn passing() -> Self {
        Self { failure: None }
    }

    fn failing(message: &'static str) -> Self {
        Self {
            failure: Some(message),
        }
    }
}

impl InitHostValidator for FakeHost {
    fn validate(
        &self,
        _server_root: &Path,
        _config: &minegr::config::Config,
    ) -> Result<(), String> {
        match self.failure {
            Some(message) => Err(message.to_owned()),
            None => Ok(()),
        }
    }
}

impl JavaRuntime for FakeJava {
    fn major_version(&self, _executable: Option<&str>) -> Result<u32, String> {
        Ok(self.major)
    }
}

struct FakePrompter {
    interactive: bool,
    confirmation: bool,
    used: Cell<bool>,
    summary: RefCell<String>,
}

impl FakePrompter {
    fn non_interactive() -> Self {
        Self {
            interactive: false,
            confirmation: false,
            used: Cell::new(false),
            summary: RefCell::new(String::new()),
        }
    }

    fn interactive_with_confirmation(confirmation: bool) -> Self {
        Self {
            interactive: true,
            confirmation,
            used: Cell::new(false),
            summary: RefCell::new(String::new()),
        }
    }

    fn was_used(&self) -> bool {
        self.used.get()
    }
}

impl InitPrompter for FakePrompter {
    fn is_interactive(&self) -> bool {
        self.interactive
    }

    fn name(&self, default: &str) -> Result<String, String> {
        self.used.set(true);
        Ok(default.to_owned())
    }

    fn version(&self, versions: &[MinecraftVersion]) -> Result<usize, String> {
        self.used.set(true);
        assert!(!versions.is_empty());
        Ok(0)
    }

    fn platform(&self, platforms: &[Platform]) -> Result<usize, String> {
        self.used.set(true);
        assert!(!platforms.is_empty());
        Ok(0)
    }

    fn accept_eula(&self) -> Result<bool, String> {
        self.used.set(true);
        Ok(true)
    }

    fn show_summary(&self, summary: &str) {
        self.summary.replace(summary.to_owned());
    }

    fn confirm_creation(&self) -> Result<bool, String> {
        self.used.set(true);
        Ok(self.confirmation)
    }
}
