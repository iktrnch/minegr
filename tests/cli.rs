use assert_cmd::Command;
use predicates::prelude::*;
use rustix::fs::{FlockOperation, fcntl_lock};
use std::fs::File;
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;

#[test]
fn help_lists_every_public_command() {
    Command::cargo_bin("minegr")
        .expect("binary should build")
        .arg("--help")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("init")
                .and(predicate::str::contains("sync"))
                .and(predicate::str::contains("start"))
                .and(predicate::str::contains("stop"))
                .and(predicate::str::contains("restart"))
                .and(predicate::str::contains("status"))
                .and(predicate::str::contains("logs"))
                .and(predicate::str::contains("console"))
                .and(predicate::str::contains("backup")),
        );
}

#[test]
fn every_public_subcommand_parses_in_the_real_process() {
    for command in [
        "init", "sync", "start", "stop", "restart", "status", "logs", "console", "backup",
    ] {
        Command::cargo_bin("minegr")
            .expect("binary should build")
            .args([command, "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Usage:"))
            .stderr(predicate::str::is_empty());
    }
}

#[test]
fn global_config_is_accepted_after_a_subcommand() {
    Command::cargo_bin("minegr")
        .expect("binary should build")
        .args(["status", "--config", "chosen.toml"])
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with(
            "Failed to load configuration chosen.toml:",
        ));
}

#[test]
fn logs_rejects_out_of_range_last_with_clap_exit_code() {
    Command::cargo_bin("minegr")
        .expect("binary should build")
        .args(["logs", "--last", "10001"])
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("10001 is not in 1..=10000"));
}

#[test]
fn init_rejects_zero_port_with_clap_exit_code() {
    Command::cargo_bin("minegr")
        .expect("binary should build")
        .args(["init", "--port", "0"])
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("0 is not in 1..=65535"));
}

#[test]
fn init_uuid_requires_an_existing_configuration() {
    let temp = tempfile::tempdir().expect("temporary directory");

    Command::cargo_bin("minegr")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["init", "--uuid"])
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with(
            "Failed to load configuration ./minegr.toml:",
        ));
}

#[test]
fn init_uuid_reports_success_and_rewrites_only_uuid_through_the_real_cli() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let port = available_port();
    write_sync_fixture(temp.path(), port);
    let runtime = temp.path().join("runtime");
    std::fs::create_dir(&runtime).unwrap();
    let config_path = temp.path().join("minegr.toml");
    let before = std::fs::read_to_string(&config_path).unwrap();
    let canonical = config_path.canonicalize().unwrap();

    Command::cargo_bin("minegr")
        .expect("binary should build")
        .current_dir(temp.path())
        .env("XDG_RUNTIME_DIR", &runtime)
        .args(["init", "--uuid"])
        .assert()
        .success()
        .stdout(format!(
            "Configuration: {}\nStart it with: minegr start --config {}\n",
            canonical.display(),
            canonical.display()
        ))
        .stderr(predicate::str::is_empty());

    let after = std::fs::read_to_string(config_path).unwrap();
    assert_ne!(after, before);
    let before_without_uuid = before
        .lines()
        .filter(|line| !line.starts_with("uuid = "))
        .collect::<Vec<_>>();
    let after_without_uuid = after
        .lines()
        .filter(|line| !line.starts_with("uuid = "))
        .collect::<Vec<_>>();
    assert_eq!(after_without_uuid, before_without_uuid);
}

#[test]
fn non_interactive_init_reports_required_flags_without_network_access() {
    let temp = tempfile::tempdir().expect("temporary directory");

    Command::cargo_bin("minegr")
        .expect("binary should build")
        .current_dir(temp.path())
        .env("CI", "1")
        .arg("init")
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains("--minecraft-version")
                .and(predicate::str::contains("--platform"))
                .and(predicate::str::contains("--accept-eula"))
                .and(predicate::str::contains("--yes")),
        );
}

#[test]
fn valid_configuration_reaches_the_selected_command_boundary() {
    let temp = tempfile::tempdir().expect("temporary directory");
    std::fs::write(
        temp.path().join("minegr.toml"),
        r#"
[minegr]
config_version = 1
uuid = "018f0000-0000-7000-8000-000000000000"
name = "survival"

[minecraft]
platform = "vanilla"
version = "1.21.8"
checksum = "sha1:0123456789abcdef"
eula = true

[java]
jvm_args = ["-Xms2G", "-Xmx2G"]
server_args = ["nogui"]

[minecraft.properties]
server-port = 25565
"#,
    )
    .expect("fixture should be writable");

    Command::cargo_bin("minegr")
        .expect("binary should build")
        .current_dir(temp.path())
        .arg("status")
        .assert()
        .code(4)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "Command `status` is not implemented yet",
        ));
}

#[test]
fn broad_configuration_permissions_warn_on_real_stderr() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let path = temp.path().join("minegr.toml");
    std::fs::write(
        &path,
        r#"
[minegr]
config_version = 1
uuid = "018f0000-0000-7000-8000-000000000000"
name = "survival"

[minecraft]
platform = "vanilla"
version = "1.21.8"
checksum = "sha1:0123456789abcdef"
eula = true

[java]
jvm_args = ["-Xms2G", "-Xmx2G"]
server_args = ["nogui"]
"#,
    )
    .expect("fixture should be writable");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644))
        .expect("fixture permissions");

    Command::cargo_bin("minegr")
        .expect("binary should build")
        .current_dir(temp.path())
        .arg("status")
        .assert()
        .code(4)
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains("configuration permissions 0644 are broader than owner-only")
                .and(predicate::str::contains(
                    "Command `status` is not implemented yet",
                )),
        );
}

#[test]
fn sync_reports_exact_success_output_through_the_real_cli() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let port = available_port();
    write_sync_fixture(temp.path(), port);
    let runtime = temp.path().join("runtime");
    std::fs::create_dir(&runtime).unwrap();
    std::fs::write(
        temp.path().join("server.properties"),
        format!("server-port={port}\nwhite-list=true\n"),
    )
    .unwrap();
    let canonical = temp.path().join("minegr.toml").canonicalize().unwrap();

    Command::cargo_bin("minegr")
        .expect("binary should build")
        .current_dir(temp.path())
        .env("XDG_RUNTIME_DIR", &runtime)
        .arg("sync")
        .assert()
        .success()
        .stdout(format!(
            "Configuration synchronized: {}\n",
            canonical.display()
        ))
        .stderr(predicate::str::is_empty());
}

#[test]
fn sync_uses_unavailable_exit_code_when_the_configured_port_is_occupied() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    write_sync_fixture(temp.path(), port);
    let runtime = temp.path().join("runtime");
    std::fs::create_dir(&runtime).unwrap();
    std::fs::write(
        temp.path().join("server.properties"),
        format!("server-port={port}\n"),
    )
    .unwrap();
    let before = std::fs::read(temp.path().join("minegr.toml")).unwrap();

    Command::cargo_bin("minegr")
        .expect("binary should build")
        .current_dir(temp.path())
        .env("XDG_RUNTIME_DIR", &runtime)
        .arg("sync")
        .assert()
        .code(3)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("server appears to be running"));

    assert_eq!(
        std::fs::read(temp.path().join("minegr.toml")).unwrap(),
        before
    );
    drop(listener);
}

#[test]
fn sync_rejects_an_accepting_owner_matched_daemon_socket() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let port = available_port();
    write_sync_fixture(temp.path(), port);
    std::fs::write(
        temp.path().join("server.properties"),
        format!("server-port={port}\n"),
    )
    .unwrap();
    let runtime = temp.path().join("runtime");
    let sockets = runtime.join("minegr");
    std::fs::create_dir_all(&sockets).unwrap();
    let listener =
        UnixListener::bind(sockets.join("018f0000-0000-7000-8000-000000000000.sock")).unwrap();
    let accepting = std::thread::spawn(move || listener.accept().map(|_| ()).unwrap());

    Command::cargo_bin("minegr")
        .expect("binary should build")
        .current_dir(temp.path())
        .env("XDG_RUNTIME_DIR", &runtime)
        .arg("sync")
        .assert()
        .code(3)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("server appears to be running"));

    accepting.join().expect("daemon listener thread");
}

#[test]
fn sync_rejects_an_active_default_world_session_lock() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let port = available_port();
    write_sync_fixture(temp.path(), port);
    std::fs::write(
        temp.path().join("server.properties"),
        format!("server-port={port}\n"),
    )
    .unwrap();
    std::fs::create_dir(temp.path().join("world")).unwrap();
    let lock = File::options()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(temp.path().join("world/session.lock"))
        .unwrap();
    fcntl_lock(&lock, FlockOperation::NonBlockingLockExclusive).unwrap();
    let runtime = temp.path().join("runtime");
    std::fs::create_dir(&runtime).unwrap();

    Command::cargo_bin("minegr")
        .expect("binary should build")
        .current_dir(temp.path())
        .env("XDG_RUNTIME_DIR", &runtime)
        .arg("sync")
        .assert()
        .code(3)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("server appears to be running"));

    fcntl_lock(&lock, FlockOperation::Unlock).unwrap();
}

#[test]
fn sync_checks_drifted_properties_for_unmanaged_server_evidence() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let old_port = available_port();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let running_port = listener.local_addr().unwrap().port();
    write_sync_fixture(temp.path(), old_port);
    let config_path = temp.path().join("minegr.toml");
    let source = std::fs::read_to_string(&config_path).unwrap().replace(
        &format!("server-port = {old_port}"),
        &format!("server-port = {old_port}\nlevel-name = \"old-world\""),
    );
    std::fs::write(&config_path, source).unwrap();
    std::fs::write(
        temp.path().join("server.properties"),
        format!("server-port={running_port}\nlevel-name=new-world\n"),
    )
    .unwrap();
    let runtime = temp.path().join("runtime");
    std::fs::create_dir(&runtime).unwrap();

    Command::cargo_bin("minegr")
        .expect("binary should build")
        .current_dir(temp.path())
        .env("XDG_RUNTIME_DIR", &runtime)
        .arg("sync")
        .assert()
        .code(3)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("server appears to be running"));

    drop(listener);
}

#[test]
fn sync_checks_the_drifted_world_session_lock_before_rewriting() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let old_port = available_port();
    let new_port = loop {
        let candidate = available_port();
        if candidate != old_port {
            break candidate;
        }
    };
    write_sync_fixture(temp.path(), old_port);
    let config_path = temp.path().join("minegr.toml");
    let source = std::fs::read_to_string(&config_path).unwrap().replace(
        &format!("server-port = {old_port}"),
        &format!("server-port = {old_port}\nlevel-name = \"old-world\""),
    );
    std::fs::write(&config_path, source).unwrap();
    std::fs::write(
        temp.path().join("server.properties"),
        format!("server-port={new_port}\nlevel-name=new-world\n"),
    )
    .unwrap();
    std::fs::create_dir(temp.path().join("new-world")).unwrap();
    let lock = File::options()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(temp.path().join("new-world/session.lock"))
        .unwrap();
    fcntl_lock(&lock, FlockOperation::NonBlockingLockExclusive).unwrap();
    let runtime = temp.path().join("runtime");
    std::fs::create_dir(&runtime).unwrap();
    let before = std::fs::read(&config_path).unwrap();

    Command::cargo_bin("minegr")
        .expect("binary should build")
        .current_dir(temp.path())
        .env("XDG_RUNTIME_DIR", &runtime)
        .arg("sync")
        .assert()
        .code(3)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("server appears to be running"));

    assert_eq!(std::fs::read(config_path).unwrap(), before);
    fcntl_lock(&lock, FlockOperation::Unlock).unwrap();
}

fn available_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn write_sync_fixture(root: &std::path::Path, port: u16) {
    let path = root.join("minegr.toml");
    std::fs::write(
        &path,
        format!(
            r#"[minegr]
config_version = 1
uuid = "018f0000-0000-7000-8000-000000000000"
name = "survival"

[minecraft]
platform = "vanilla"
version = "1.21.8"
checksum = "sha1:0123456789abcdef"
eula = true

[java]
jvm_args = ["-Xms2G", "-Xmx2G"]
server_args = ["nogui"]

[minecraft.properties]
server-port = {port}
"#
        ),
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
}
