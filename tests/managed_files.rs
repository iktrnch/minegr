use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::{MetadataExt, symlink};

use minegr::config::PropertyValue;
use minegr::managed_files::{
    ManagedFileError, materialize_text_files, parse_server_properties, render_server_properties,
};

#[test]
fn server_properties_render_deterministically_from_every_scalar() {
    let properties = BTreeMap::from([
        (
            "difficulty".to_owned(),
            PropertyValue::String("hard".to_owned()),
        ),
        (
            "motd".to_owned(),
            PropertyValue::String("Minegr = ready\\now".to_owned()),
        ),
        ("rate-limit".to_owned(), PropertyValue::Float(0.5)),
        ("server-port".to_owned(), PropertyValue::Integer(25_565)),
        ("white-list".to_owned(), PropertyValue::Boolean(true)),
    ]);

    assert_eq!(
        render_server_properties(&properties),
        "difficulty=hard\nmotd=Minegr = ready\\\\now\nrate-limit=0.5\nserver-port=25565\nwhite-list=true\n"
    );
}

#[test]
fn server_properties_parse_complete_java_property_lines() {
    let parsed = parse_server_properties(
        "# generated\n! retained comment\nserver-port:25565\nwhite-list = true\nlong-value=first\\\n  second\nescaped\\ key=value\\nline\n",
    )
    .expect("properties should parse");

    assert_eq!(parsed["server-port"], "25565");
    assert_eq!(parsed["white-list"], "true");
    assert_eq!(parsed["long-value"], "firstsecond");
    assert_eq!(parsed["escaped key"], "value\nline");
}

#[test]
fn server_properties_reject_duplicates_and_malformed_escapes() {
    let duplicate = parse_server_properties("motd=first\nmotd=second\n")
        .expect_err("duplicate property should fail");
    assert!(duplicate.to_string().contains("duplicate property `motd`"));

    let malformed = parse_server_properties("motd=bad\\u12xz\n")
        .expect_err("invalid unicode escape should fail");
    assert!(malformed.to_string().contains("line 1"));
}

#[test]
fn materialization_creates_expected_text_files_and_accepts_identical_files() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let properties = BTreeMap::from([("server-port".to_owned(), PropertyValue::Integer(25_565))]);

    materialize_text_files(temp.path(), &properties, true).expect("first materialization");
    let properties_path = temp.path().join("server.properties");
    let eula_path = temp.path().join("eula.txt");
    let properties_inode = fs::metadata(&properties_path).expect("metadata").ino();
    let eula_inode = fs::metadata(&eula_path).expect("metadata").ino();

    assert_eq!(
        fs::read_to_string(&properties_path).expect("server.properties"),
        "server-port=25565\n"
    );
    assert_eq!(
        fs::read_to_string(&eula_path).expect("eula.txt"),
        "eula=true\n"
    );
    materialize_text_files(temp.path(), &properties, true).expect("identical files are accepted");
    assert_eq!(
        fs::metadata(properties_path).expect("metadata").ino(),
        properties_inode
    );
    assert_eq!(fs::metadata(eula_path).expect("metadata").ino(), eula_inode);
    assert_eq!(temporary_files(temp.path()), 0);
}

#[test]
fn materialization_preflights_all_text_files_before_creating_any() {
    let temp = tempfile::tempdir().expect("temporary directory");
    fs::write(temp.path().join("server.properties"), "server-port=25566\n")
        .expect("conflicting fixture");
    let properties = BTreeMap::from([("server-port".to_owned(), PropertyValue::Integer(25_565))]);

    let error = materialize_text_files(temp.path(), &properties, true)
        .expect_err("differing file should fail");

    assert!(matches!(error, ManagedFileError::Differs { name } if name == "server.properties"));
    assert!(error.to_string().contains("remove it before retrying"));
    assert!(!temp.path().join("eula.txt").exists());
    assert_eq!(temporary_files(temp.path()), 0);
}

#[test]
fn materialization_rejects_symlinked_managed_paths() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let outside = temp.path().join("outside");
    fs::write(&outside, "outside").expect("outside fixture");
    symlink(&outside, temp.path().join("eula.txt")).expect("symlink fixture");

    let error = materialize_text_files(temp.path(), &BTreeMap::new(), true)
        .expect_err("symlink must be rejected");

    assert!(error.to_string().contains("eula.txt"));
    assert!(error.to_string().contains("not a regular file"));
    assert_eq!(
        fs::read_to_string(outside).expect("outside remains"),
        "outside"
    );
}

fn temporary_files(directory: &std::path::Path) -> usize {
    fs::read_dir(directory)
        .expect("directory should be readable")
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().ends_with(".tmp"))
        .count()
}
