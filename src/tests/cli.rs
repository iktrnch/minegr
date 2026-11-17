use super::*;
use clap::CommandFactory;

/// Uses the current directory when a command has no positional directory.
#[test]
fn config_dir_defaults_to_current_directory_for_every_command() {
    for command in [
        "daemon", "backup", "console", "init", "logs", "restart", "start", "status", "stop", "sync",
    ] {
        let cli = Cli::try_parse_from(["moxum", command]).unwrap();
        assert_eq!(cli.config_dir, PathBuf::from("."));
    }
}

/// Accepts the global positional directory before or after every command.
#[test]
fn every_command_accepts_a_config_directory() {
    for command in [
        "daemon", "backup", "console", "init", "logs", "restart", "start", "status", "stop", "sync",
    ] {
        for directory in ["./server", "/tmp/moxum server", "instances/survival/"] {
            for arguments in [["moxum", command, directory], ["moxum", directory, command]] {
                let cli = Cli::try_parse_from(arguments).unwrap();
                assert_eq!(cli.config_dir, PathBuf::from(directory));
            }
        }
    }
}

/// Parses every supported initialization option.
#[test]
fn init_accepts_all_options() {
    let matches = Cli::command()
        .try_get_matches_from([
            "moxum",
            "init",
            "./server",
            "--name",
            "Survival",
            "--minecraft-version",
            "1.20.1",
            "--platform",
            "fabric",
            "--accept-eula",
        ])
        .unwrap();
    let init = matches.subcommand_matches("init").unwrap();

    assert_eq!(
        init.get_one::<PathBuf>("config_dir"),
        Some(&PathBuf::from("./server"))
    );
    assert_eq!(
        init.get_one::<String>("name").map(String::as_str),
        Some("Survival")
    );
    assert_eq!(
        init.get_one::<String>("minecraft_version")
            .map(String::as_str),
        Some("1.20.1")
    );
    assert_eq!(
        init.get_one::<String>("platform").map(String::as_str),
        Some("fabric")
    );
    assert!(init.get_flag("accept_eula"));
}

/// Rejects platforms outside the supported set.
#[test]
fn init_rejects_unknown_platforms() {
    let error = Cli::try_parse_from(["moxum", "init", "--platform", "forge"])
        .err()
        .expect("unsupported platforms must be rejected");
    assert_eq!(error.kind(), clap::error::ErrorKind::InvalidValue);
}

/// Leaves every init option unset when no flags are supplied.
#[test]
fn init_options_are_optional() {
    let cli = Cli::try_parse_from(["moxum", "init"]).unwrap();
    let Some(Commands::Init(args)) = cli.command else {
        panic!("expected init command");
    };
    assert_eq!(cli.config_dir, PathBuf::from("."));
    assert_eq!(args.name, None);
    assert_eq!(args.minecraft_version, None);
    assert_eq!(args.platform, None);
    assert_eq!(args.accept_eula, None);
}

/// Accepts each of the three supported platforms.
#[test]
fn init_accepts_supported_platforms() {
    for platform in ["vanilla", "paper", "fabric"] {
        let cli = Cli::try_parse_from(["moxum", "init", "--platform", platform]).unwrap();
        let Some(Commands::Init(args)) = cli.command else {
            panic!("expected init command");
        };
        assert_eq!(args.platform.as_deref(), Some(platform));
    }
}

/// Accepts the directory before or after flags with explicit values.
#[test]
fn init_directory_can_appear_before_or_after_flags() {
    for arguments in [
        ["moxum", "init", "./server", "--name", "Survival"],
        ["moxum", "init", "--name", "Survival", "./server"],
    ] {
        let cli = Cli::try_parse_from(arguments).unwrap();
        let Some(Commands::Init(args)) = cli.command else {
            panic!("expected init command");
        };
        assert_eq!(cli.config_dir, PathBuf::from("./server"));
        assert_eq!(args.name.as_deref(), Some("Survival"));
    }
}

/// Rejects the removed config-file flags before and after every command.
#[test]
fn config_flags_are_no_longer_accepted() {
    for command in [
        "daemon", "backup", "console", "init", "logs", "restart", "start", "status", "stop", "sync",
    ] {
        for flag in ["--config", "-c"] {
            for arguments in [
                ["moxum", flag, "moxum.toml", command],
                ["moxum", command, flag, "moxum.toml"],
            ] {
                let error = Cli::try_parse_from(arguments)
                    .err()
                    .expect("config-file flags must be rejected");
                assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);
            }
        }
    }
}
