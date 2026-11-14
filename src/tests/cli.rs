use super::*;
use clap::CommandFactory;

/// Uses the conventional config filename when no override is supplied.
#[test]
fn config_defaults_to_minegr_toml() {
    let matches = Cli::command()
        .try_get_matches_from(["minegr", "init"])
        .unwrap();

    assert_eq!(
        matches.get_one::<PathBuf>("config"),
        Some(&PathBuf::from("minegr.toml"))
    );
}

/// Accepts global config overrides before and after every command.
#[test]
fn config_is_global_for_every_command() {
    for command in [
        "daemon", "backup", "console", "init", "logs", "restart", "start", "status", "stop", "sync",
    ] {
        for flag in ["--config", "-c"] {
            for arguments in [
                ["minegr", flag, "instances/survival.toml", command],
                ["minegr", command, flag, "instances/survival.toml"],
            ] {
                let cli = Cli::try_parse_from(arguments).unwrap();
                assert_eq!(cli.config, PathBuf::from("instances/survival.toml"));
            }
        }
    }
}

/// Parses every supported initialization option.
#[test]
fn init_accepts_all_options() {
    let matches = Cli::command()
        .try_get_matches_from([
            "minegr",
            "init",
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
    let error = Cli::try_parse_from(["minegr", "init", "--platform", "forge"])
        .err()
        .expect("unsupported platforms must be rejected");
    assert_eq!(error.kind(), clap::error::ErrorKind::InvalidValue);
}

/// Leaves every init option unset when no flags are supplied.
#[test]
fn init_options_are_optional() {
    let cli = Cli::try_parse_from(["minegr", "init"]).unwrap();
    let Some(Commands::Init(args)) = cli.command else {
        panic!("expected init command");
    };
    assert_eq!(args.name, None);
    assert_eq!(args.minecraft_version, None);
    assert_eq!(args.platform, None);
    assert_eq!(args.accept_eula, None);
}

/// Accepts each of the three supported platforms.
#[test]
fn init_accepts_supported_platforms() {
    for platform in ["vanilla", "paper", "fabric"] {
        let cli = Cli::try_parse_from(["minegr", "init", "--platform", platform]).unwrap();
        let Some(Commands::Init(args)) = cli.command else {
            panic!("expected init command");
        };
        assert_eq!(args.platform.as_deref(), Some(platform));
    }
}
