use std::collections::BTreeMap;

use minegr::config::{Config, JavaConfig, MinecraftConfig, MinegrConfig, Platform, PropertyValue};
use minegr::validation::{
    ConfigFileObservation, FileKind, HostCheck, HostCheckKind, HostCheckStatus, Observation,
    RecognizedProperty, ValidationCode, ValidationLevel, validate_config, validate_host,
};

fn valid_config() -> Config {
    Config {
        minegr: MinegrConfig {
            config_version: 1,
            uuid: Some("018f0000-0000-7000-8000-000000000000".to_owned()),
            name: "survival".to_owned(),
        },
        minecraft: MinecraftConfig {
            platform: Platform::Paper,
            version: "26.2".to_owned(),
            build: Some(42),
            loader: None,
            installer: None,
            checksum: Some("sha256:0123456789abcdef".to_owned()),
            eula: true,
            properties: BTreeMap::from([
                ("server-port".to_owned(), PropertyValue::Integer(25_565)),
                ("white-list".to_owned(), PropertyValue::Boolean(true)),
            ]),
        },
        java: JavaConfig {
            executable: Some("java".to_owned()),
            jvm_args: vec!["-Xms2G".to_owned(), "-Xmx2G".to_owned()],
            server_args: vec!["nogui".to_owned()],
        },
    }
}

#[test]
fn pure_validation_reports_independent_findings_in_documented_order() {
    let mut config = valid_config();
    config.minegr.config_version = 2;
    config.minegr.uuid = Some("not-a-uuid".to_owned());
    config.minecraft.build = None;
    config.minecraft.loader = Some("unexpected".to_owned());
    config.minecraft.checksum = Some("sha256:not-hex".to_owned());
    config.java.jvm_args.push("-jar".to_owned());
    config
        .minecraft
        .properties
        .insert("server-port".to_owned(), PropertyValue::Integer(0));
    config.minecraft.properties.insert(
        "white-list".to_owned(),
        PropertyValue::String("yes".to_owned()),
    );

    let report = validate_config(&config);

    assert_eq!(
        report.codes(),
        vec![
            ValidationCode::ConfigVersion,
            ValidationCode::Uuid,
            ValidationCode::PlatformCoordinates,
            ValidationCode::PlatformCoordinates,
            ValidationCode::Checksum,
            ValidationCode::JavaArguments,
            ValidationCode::ServerPort,
            ValidationCode::WhiteList,
        ]
    );
    assert!(
        report
            .findings()
            .iter()
            .all(|finding| finding.level == ValidationLevel::Error)
    );
}

#[test]
fn valid_configuration_has_no_pure_findings() {
    assert!(validate_config(&valid_config()).findings().is_empty());
}

#[test]
fn recognized_property_registry_is_complete_and_unknown_scalars_remain_forward_compatible() {
    assert_eq!(
        RecognizedProperty::ALL.map(RecognizedProperty::name),
        ["server-port", "white-list", "level-name"]
    );
    let mut config = valid_config();
    config
        .minecraft
        .properties
        .insert("rate-limit".to_owned(), PropertyValue::Float(0.5));

    assert!(validate_config(&config).findings().is_empty());
}

#[test]
fn host_validation_warns_for_permissions_and_does_not_weaken_unavailable_checks() {
    let config = valid_config();
    let pure = validate_config(&config);
    let input = minegr::validation::HostValidationInput {
        config_file: ConfigFileObservation {
            kind: Observation::Known(FileKind::Regular),
            owner_uid: Observation::Known(1000),
            expected_uid: 1000,
            mode: Observation::Known(0o644),
        },
        checks: vec![
            HostCheck::new(
                HostCheckKind::DiskSpace,
                None,
                HostCheckStatus::Unavailable("statvfs failed".to_owned()),
            ),
            HostCheck::new(
                HostCheckKind::JavaCompatibility,
                None,
                HostCheckStatus::Failed("Java 17 is too old".to_owned()),
            ),
        ],
    };

    let report = validate_host(&input, &pure);

    assert_eq!(
        report.codes(),
        vec![
            ValidationCode::ConfigPermissions,
            ValidationCode::JavaCompatibility,
            ValidationCode::DiskSpace,
        ]
    );
    assert_eq!(report.findings()[0].level, ValidationLevel::Warning);
    assert_eq!(report.findings()[1].level, ValidationLevel::Error);
    assert_eq!(report.findings()[2].level, ValidationLevel::Error);
}

#[test]
fn host_validation_rejects_non_regular_and_foreign_owned_configuration() {
    let pure = validate_config(&valid_config());
    let input = minegr::validation::HostValidationInput {
        config_file: ConfigFileObservation {
            kind: Observation::Known(FileKind::NonRegular),
            owner_uid: Observation::Known(2000),
            expected_uid: 1000,
            mode: Observation::Known(0o600),
        },
        checks: Vec::new(),
    };

    let report = validate_host(&input, &pure);

    assert_eq!(
        report.codes(),
        vec![ValidationCode::ConfigFileType, ValidationCode::ConfigOwner,]
    );
    assert!(
        report
            .findings()
            .iter()
            .all(|finding| finding.level == ValidationLevel::Error)
    );
}

#[test]
fn failed_prerequisite_skips_only_the_dependent_host_check() {
    let mut config = valid_config();
    config
        .minecraft
        .properties
        .insert("server-port".to_owned(), PropertyValue::Integer(0));
    let pure = validate_config(&config);
    let input = minegr::validation::HostValidationInput {
        config_file: ConfigFileObservation {
            kind: Observation::Known(FileKind::Regular),
            owner_uid: Observation::Known(1000),
            expected_uid: 1000,
            mode: Observation::Known(0o600),
        },
        checks: vec![
            HostCheck::new(
                HostCheckKind::Port,
                None,
                HostCheckStatus::Failed("port is occupied".to_owned()),
            )
            .requiring(ValidationCode::ServerPort),
            HostCheck::new(
                HostCheckKind::Memory,
                None,
                HostCheckStatus::Failed("insufficient memory".to_owned()),
            ),
        ],
    };

    let report = validate_host(&input, &pure);

    assert_eq!(
        report.codes(),
        vec![ValidationCode::Memory, ValidationCode::Port]
    );
    assert_eq!(report.findings()[0].level, ValidationLevel::Error);
    assert_eq!(report.findings()[1].level, ValidationLevel::Skipped);
    assert!(report.findings()[1].message.contains("server-port"));
}

#[test]
fn failed_host_prerequisite_skips_its_dependent_host_check() {
    let pure = validate_config(&valid_config());
    let input = minegr::validation::HostValidationInput {
        config_file: ConfigFileObservation {
            kind: Observation::Known(FileKind::Regular),
            owner_uid: Observation::Known(1000),
            expected_uid: 1000,
            mode: Observation::Known(0o600),
        },
        checks: vec![
            HostCheck::new(
                HostCheckKind::ManagedArtifact,
                Some("server.jar".to_owned()),
                HostCheckStatus::Failed("unsafe path".to_owned()),
            ),
            HostCheck::new(
                HostCheckKind::ArtifactChecksum,
                Some("server.jar".to_owned()),
                HostCheckStatus::Failed("checksum differs".to_owned()),
            )
            .requiring_host(
                HostCheckKind::ManagedArtifact,
                Some("server.jar".to_owned()),
            ),
        ],
    };

    let report = validate_host(&input, &pure);

    assert_eq!(
        report.codes(),
        vec![
            ValidationCode::ManagedArtifact,
            ValidationCode::ArtifactChecksum,
        ]
    );
    assert_eq!(report.findings()[0].level, ValidationLevel::Error);
    assert_eq!(report.findings()[1].level, ValidationLevel::Skipped);
    assert!(report.findings()[1].message.contains("managed-artifact"));
}

#[test]
fn host_prerequisites_match_the_exact_subject() {
    let pure = validate_config(&valid_config());
    let input = minegr::validation::HostValidationInput {
        config_file: ConfigFileObservation {
            kind: Observation::Known(FileKind::Regular),
            owner_uid: Observation::Known(1000),
            expected_uid: 1000,
            mode: Observation::Known(0o600),
        },
        checks: vec![
            HostCheck::new(
                HostCheckKind::ManagedArtifact,
                Some("server.properties".to_owned()),
                HostCheckStatus::Failed("unsafe path".to_owned()),
            ),
            HostCheck::new(
                HostCheckKind::ManagedArtifact,
                Some("server.jar".to_owned()),
                HostCheckStatus::Passed,
            ),
            HostCheck::new(
                HostCheckKind::ArtifactChecksum,
                Some("server.jar".to_owned()),
                HostCheckStatus::Failed("checksum differs".to_owned()),
            )
            .requiring_host(
                HostCheckKind::ManagedArtifact,
                Some("server.jar".to_owned()),
            ),
        ],
    };

    let report = validate_host(&input, &pure);

    assert_eq!(report.findings().len(), 2);
    assert!(
        report
            .findings()
            .iter()
            .all(|finding| finding.level == ValidationLevel::Error)
    );
}

#[test]
fn host_prerequisites_are_independent_of_report_order() {
    let pure = validate_config(&valid_config());
    let input = minegr::validation::HostValidationInput {
        config_file: ConfigFileObservation {
            kind: Observation::Known(FileKind::Regular),
            owner_uid: Observation::Known(1000),
            expected_uid: 1000,
            mode: Observation::Known(0o600),
        },
        checks: vec![
            HostCheck::new(
                HostCheckKind::JavaCompatibility,
                None,
                HostCheckStatus::Failed("Java is incompatible".to_owned()),
            )
            .requiring_host(HostCheckKind::ServerRoot, None),
            HostCheck::new(
                HostCheckKind::ServerRoot,
                None,
                HostCheckStatus::Failed("root is unsafe".to_owned()),
            ),
        ],
    };

    let report = validate_host(&input, &pure);

    assert_eq!(report.findings()[0].level, ValidationLevel::Skipped);
    assert_eq!(report.findings()[1].level, ValidationLevel::Error);
    assert!(report.findings()[0].message.contains("server-root"));
}

#[test]
fn combined_validation_report_retains_host_errors() {
    let mut report = validate_config(&valid_config());
    let host = validate_host(
        &minegr::validation::HostValidationInput {
            config_file: ConfigFileObservation {
                kind: Observation::Known(FileKind::Regular),
                owner_uid: Observation::Known(1000),
                expected_uid: 1000,
                mode: Observation::Known(0o600),
            },
            checks: vec![HostCheck::new(
                HostCheckKind::DiskSpace,
                None,
                HostCheckStatus::Failed("insufficient disk space".to_owned()),
            )],
        },
        &report,
    );

    report.append(host);

    assert!(report.has_errors());
    assert_eq!(report.codes(), vec![ValidationCode::DiskSpace]);
}

#[test]
fn host_checks_are_sorted_by_kind_then_subject() {
    let config = valid_config();
    let pure = validate_config(&config);
    let input = minegr::validation::HostValidationInput {
        config_file: ConfigFileObservation {
            kind: Observation::Known(FileKind::Regular),
            owner_uid: Observation::Known(1000),
            expected_uid: 1000,
            mode: Observation::Known(0o600),
        },
        checks: vec![
            HostCheck::new(
                HostCheckKind::ManagedArtifact,
                Some("server.properties".to_owned()),
                HostCheckStatus::Failed("differs".to_owned()),
            ),
            HostCheck::new(
                HostCheckKind::ManagedArtifact,
                Some("server.jar".to_owned()),
                HostCheckStatus::Failed("differs".to_owned()),
            ),
            HostCheck::new(
                HostCheckKind::JavaCompatibility,
                None,
                HostCheckStatus::Passed,
            ),
        ],
    };

    let report = validate_host(&input, &pure);

    assert!(report.findings()[0].message.contains("server.jar"));
    assert!(report.findings()[1].message.contains("server.properties"));
}
