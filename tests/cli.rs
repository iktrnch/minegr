use assert_cmd::Command;
use predicates::prelude::*;
use std::os::unix::fs::PermissionsExt;

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
