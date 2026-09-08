use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::Path;
use std::thread::{self, JoinHandle};

use minegr::artifact::{
    ArtifactService, Endpoints, MinecraftVersion, MinecraftVersionKind, PlatformAvailability,
    ResolvedArtifact,
};
use minegr::config::{MinecraftConfig, Platform};

#[test]
fn vanilla_metadata_resolves_an_exact_server_and_java_requirement() {
    let server = MockServer::start(vec![
        MockResponse::json(
            "/manifest",
            r#"{"versions":[
                {"id":"1.21.8","type":"release","url":"REPLACE/version"},
                {"id":"25w10a","type":"snapshot","url":"REPLACE/snapshot"}
            ]}"#,
        ),
        MockResponse::json(
            "/version",
            r#"{"downloads":{"server":{"url":"REPLACE/server.jar","sha1":"1e5dcbb59b753cb1d46e234d8f6180285b8b86ad"}},"javaVersion":{"majorVersion":21}}"#,
        ),
    ]);
    let service = server.service();

    let versions = service.versions().expect("version manifest");
    assert_eq!(versions[0].id, "1.21.8");
    assert_eq!(versions[0].kind, MinecraftVersionKind::Release);
    assert_eq!(versions[1].kind, MinecraftVersionKind::Snapshot);
    let resolved = service
        .resolve(&versions[0], Platform::Vanilla)
        .expect("vanilla artifact");

    assert_eq!(resolved.version, "1.21.8");
    assert_eq!(
        resolved.checksum.as_deref(),
        Some("sha1:1e5dcbb59b753cb1d46e234d8f6180285b8b86ad")
    );
    assert_eq!(resolved.required_java_major, 21);
    assert_eq!(resolved.download_url, server.url("/server.jar"));
    server.finish();
}

#[test]
fn paper_resolution_selects_the_first_stable_build_and_sends_user_agent() {
    let server = MockServer::start(vec![
        MockResponse::json(
            "/unused",
            r#"{"downloads":{},"javaVersion":{"majorVersion":21}}"#,
        ),
        MockResponse::json_with_user_agent(
            "/paper/versions/1.21.8/builds",
            r#"[
              {"id":101,"channel":"EXPERIMENTAL","downloads":{"server:default":{"url":"ignored"}}},
              {"id":100,"channel":"STABLE","downloads":{"server:default":{"url":"REPLACE/paper.jar","checksums":{"sha256":"abcdef"}}}}
            ]"#,
        ),
    ]);
    let service = server.service();
    let version = MinecraftVersion::new(
        "1.21.8",
        server.url("/unused"),
        MinecraftVersionKind::Release,
    );

    let resolved = service
        .resolve(&version, Platform::Paper)
        .expect("paper artifact");

    assert_eq!(resolved.build, Some(100));
    assert_eq!(resolved.checksum.as_deref(), Some("sha256:abcdef"));
    assert_eq!(resolved.download_url, server.url("/paper.jar"));
    server.finish();
}

#[test]
fn fabric_resolution_pins_newest_stable_loader_and_installer() {
    let server = MockServer::start(vec![
        MockResponse::json(
            "/unused",
            r#"{"downloads":{},"javaVersion":{"majorVersion":21}}"#,
        ),
        MockResponse::json(
            "/fabric/loader/1.21.8",
            r#"[
              {"loader":{"version":"0.17.0","stable":false}},
              {"loader":{"version":"0.16.14","stable":true}}
            ]"#,
        ),
        MockResponse::json(
            "/fabric/installer",
            r#"[
              {"version":"1.1.1","stable":false},
              {"version":"1.0.3","stable":true}
            ]"#,
        ),
    ]);
    let service = server.service();
    let version = MinecraftVersion::new(
        "1.21.8",
        server.url("/unused"),
        MinecraftVersionKind::Release,
    );

    let resolved = service
        .resolve(&version, Platform::Fabric)
        .expect("fabric artifact");

    assert_eq!(resolved.loader.as_deref(), Some("0.16.14"));
    assert_eq!(resolved.installer.as_deref(), Some("1.0.3"));
    assert_eq!(
        resolved.download_url,
        server.url("/fabric/server/1.21.8/0.16.14/1.0.3/server/jar")
    );
    assert_eq!(resolved.checksum, None);
    server.finish();
}

#[test]
fn platform_availability_uses_the_documented_catalogs() {
    let server = MockServer::start(vec![
        MockResponse::json(
            "/version",
            r#"{"downloads":{"server":{"url":"REPLACE/server.jar","sha1":"unused"}},"javaVersion":{"majorVersion":21}}"#,
        ),
        MockResponse::json_with_user_agent(
            "/paper",
            r#"{"versions":{"1.21":["1.21.8","1.21.7"]}}"#,
        ),
        MockResponse::json_with_user_agent(
            "/paper/versions/1.21.8/builds",
            r#"[{"id":100,"channel":"STABLE","downloads":{"server:default":{"url":"REPLACE/paper.jar"}}}]"#,
        ),
        MockResponse::json("/fabric/game", r#"[{"version":"1.21.8","stable":true}]"#),
        MockResponse::json(
            "/fabric/loader/1.21.8",
            r#"[{"loader":{"version":"0.16.14","stable":true}}]"#,
        ),
        MockResponse::json(
            "/fabric/installer",
            r#"[{"version":"1.0.3","stable":true}]"#,
        ),
    ]);
    let service = server.service();
    let version = MinecraftVersion::new(
        "1.21.8",
        server.url("/version"),
        MinecraftVersionKind::Release,
    );

    assert_eq!(
        service.available_platforms(&version).expect("availability"),
        PlatformAvailability {
            vanilla: true,
            paper: true,
            fabric: true,
        }
    );
    server.finish();
}

#[test]
fn platform_availability_excludes_paper_without_a_stable_server_build() {
    let server = MockServer::start(vec![
        MockResponse::json(
            "/version",
            r#"{"downloads":{"server":{"url":"REPLACE/server.jar","sha1":"unused"}},"javaVersion":{"majorVersion":21}}"#,
        ),
        MockResponse::json_with_user_agent("/paper", r#"{"versions":{"1.21":["1.21.8"]}}"#),
        MockResponse::json_with_user_agent(
            "/paper/versions/1.21.8/builds",
            r#"[{"id":101,"channel":"EXPERIMENTAL","downloads":{"server:default":{"url":"ignored"}}}]"#,
        ),
        MockResponse::json("/fabric/game", "[]"),
    ]);
    let version = MinecraftVersion::new(
        "1.21.8",
        server.url("/version"),
        MinecraftVersionKind::Release,
    );

    let availability = server
        .service()
        .available_platforms(&version)
        .expect("availability");

    assert!(!availability.paper);
    server.finish();
}

#[test]
fn platform_availability_excludes_fabric_without_a_stable_installer() {
    let server = MockServer::start(vec![
        MockResponse::json(
            "/version",
            r#"{"downloads":{"server":{"url":"REPLACE/server.jar","sha1":"unused"}},"javaVersion":{"majorVersion":21}}"#,
        ),
        MockResponse::json_with_user_agent("/paper", r#"{"versions":{}}"#),
        MockResponse::json("/fabric/game", r#"[{"version":"1.21.8","stable":true}]"#),
        MockResponse::json(
            "/fabric/loader/1.21.8",
            r#"[{"loader":{"version":"0.16.14","stable":true}}]"#,
        ),
        MockResponse::json(
            "/fabric/installer",
            r#"[{"version":"1.1.1","stable":false}]"#,
        ),
    ]);
    let version = MinecraftVersion::new(
        "1.21.8",
        server.url("/version"),
        MinecraftVersionKind::Release,
    );

    let availability = server
        .service()
        .available_platforms(&version)
        .expect("availability");

    assert!(!availability.fabric);
    server.finish();
}

#[test]
fn verified_download_is_published_atomically_and_checksum_failure_publishes_nothing() {
    let good_server = MockServer::start(vec![MockResponse::bytes("/artifact.jar", b"artifact")]);
    let good = ResolvedArtifact {
        platform: Platform::Vanilla,
        version: "1.21.8".to_owned(),
        build: None,
        loader: None,
        installer: None,
        checksum: Some("sha1:1e5dcbb59b753cb1d46e234d8f6180285b8b86ad".to_owned()),
        download_url: good_server.url("/artifact.jar"),
        required_java_major: 21,
    };
    let temp = tempfile::tempdir().expect("temporary directory");

    good_server
        .service()
        .download(&good, temp.path())
        .expect("verified artifact");

    assert_eq!(
        fs::read(temp.path().join("server.jar")).expect("published jar"),
        b"artifact"
    );
    assert_eq!(temporary_files(temp.path()), 0);
    good_server.finish();

    let bad_server = MockServer::start(vec![MockResponse::bytes("/artifact.jar", b"corrupt")]);
    let bad = ResolvedArtifact {
        download_url: bad_server.url("/artifact.jar"),
        ..good
    };
    let bad_root = tempfile::tempdir().expect("temporary directory");

    let error = bad_server
        .service()
        .download(&bad, bad_root.path())
        .expect_err("checksum mismatch");

    assert!(error.to_string().contains("checksum mismatch"));
    assert!(!bad_root.path().join("server.jar").exists());
    assert_eq!(temporary_files(bad_root.path()), 0);
    bad_server.finish();
}

#[test]
fn unavailable_and_malformed_upstream_metadata_fail_without_fallbacks() {
    let unavailable = MockServer::start(vec![MockResponse::status("/manifest", 503, b"later")]);
    let error = unavailable
        .service()
        .versions()
        .expect_err("unavailable manifest");
    assert!(error.to_string().contains("HTTP 503"));
    unavailable.finish();

    let malformed = MockServer::start(vec![MockResponse::json("/manifest", "not-json")]);
    let error = malformed
        .service()
        .versions()
        .expect_err("malformed manifest");
    assert!(error.to_string().contains("invalid upstream metadata"));
    malformed.finish();
}

#[test]
fn paper_resolution_rejects_a_version_without_a_stable_build() {
    let server = MockServer::start(vec![
        MockResponse::json(
            "/unused",
            r#"{"downloads":{},"javaVersion":{"majorVersion":21}}"#,
        ),
        MockResponse::json_with_user_agent(
            "/paper/versions/1.21.8/builds",
            r#"[{"id":101,"channel":"EXPERIMENTAL","downloads":{}}]"#,
        ),
    ]);
    let service = server.service();
    let version = MinecraftVersion::new(
        "1.21.8",
        server.url("/unused"),
        MinecraftVersionKind::Release,
    );

    let error = service
        .resolve(&version, Platform::Paper)
        .expect_err("no stable fallback is allowed");

    assert!(error.to_string().contains("no eligible paper artifact"));
    server.finish();
}

#[test]
fn existing_paper_configuration_resolves_its_pinned_build_not_the_newest_one() {
    let server = MockServer::start(vec![
        MockResponse::json(
            "/manifest",
            r#"{"versions":[{"id":"1.21.8","type":"release","url":"REPLACE/version"}]}"#,
        ),
        MockResponse::json(
            "/version",
            r#"{"downloads":{},"javaVersion":{"majorVersion":21}}"#,
        ),
        MockResponse::json_with_user_agent(
            "/paper/versions/1.21.8/builds",
            r#"[
              {"id":102,"channel":"STABLE","downloads":{"server:default":{"url":"REPLACE/new.jar","checksums":{"sha256":"new"}}}},
              {"id":100,"channel":"STABLE","downloads":{"server:default":{"url":"REPLACE/pinned.jar","checksums":{"sha256":"pinned"}}}}
            ]"#,
        ),
    ]);
    let config = MinecraftConfig {
        platform: Platform::Paper,
        version: "1.21.8".to_owned(),
        build: Some(100),
        loader: None,
        installer: None,
        checksum: Some("sha256:configured".to_owned()),
        eula: true,
        properties: Default::default(),
    };

    let resolved = server
        .service()
        .resolve_pinned(&config)
        .expect("pinned paper artifact");

    assert_eq!(resolved.build, Some(100));
    assert_eq!(resolved.download_url, server.url("/pinned.jar"));
    assert_eq!(resolved.checksum.as_deref(), Some("sha256:configured"));
    server.finish();
}

fn temporary_files(directory: &Path) -> usize {
    fs::read_dir(directory)
        .expect("directory should be readable")
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().ends_with(".tmp"))
        .count()
}

struct MockResponse {
    path: String,
    body: Vec<u8>,
    content_type: &'static str,
    require_user_agent: bool,
    status: u16,
}

impl MockResponse {
    fn json(path: &str, body: &str) -> Self {
        Self::response(path, body.as_bytes(), "application/json", false)
    }

    fn json_with_user_agent(path: &str, body: &str) -> Self {
        Self::response(path, body.as_bytes(), "application/json", true)
    }

    fn bytes(path: &str, body: &[u8]) -> Self {
        Self::response(path, body, "application/java-archive", false)
    }

    fn response(
        path: &str,
        body: &[u8],
        content_type: &'static str,
        require_user_agent: bool,
    ) -> Self {
        Self {
            path: path.to_owned(),
            body: body.to_vec(),
            content_type,
            require_user_agent,
            status: 200,
        }
    }

    fn status(path: &str, status: u16, body: &[u8]) -> Self {
        Self {
            path: path.to_owned(),
            body: body.to_vec(),
            content_type: "text/plain",
            require_user_agent: false,
            status,
        }
    }
}

struct MockServer {
    address: SocketAddr,
    handle: JoinHandle<()>,
}

impl MockServer {
    fn start(responses: Vec<MockResponse>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("mock listener");
        let address = listener.local_addr().expect("listener address");
        let handle = thread::spawn(move || {
            for response in responses {
                let (mut stream, _) = listener.accept().expect("expected mock request");
                let request = read_request(&mut stream);
                let expected_path = &response.path;
                let body = String::from_utf8_lossy(&response.body)
                    .replace("REPLACE", &format!("http://{address}"))
                    .into_bytes();
                assert!(
                    request.starts_with(&format!("GET {expected_path} HTTP/1.1\r\n")),
                    "unexpected request: {request}"
                );
                if response.require_user_agent {
                    let lower = request.to_ascii_lowercase();
                    assert!(
                        lower.contains(
                            "user-agent: minegr/0.1.0 (https://github.com/iktrnch/minegr)"
                        )
                    );
                }
                write!(
                    stream,
                    "HTTP/1.1 {} Test\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    response.status,
                    response.content_type,
                    body.len()
                )
                .expect("mock response headers");
                stream.write_all(&body).expect("mock response body");
            }
        });
        Self { address, handle }
    }

    fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.address, path)
    }

    fn service(&self) -> ArtifactService {
        let root = format!("http://{}", self.address);
        ArtifactService::new(Endpoints {
            mojang_manifest: format!("{root}/manifest"),
            paper_project: format!("{root}/paper"),
            paper_builds: format!("{root}/paper/versions/{{version}}/builds"),
            fabric_games: format!("{root}/fabric/game"),
            fabric_loaders: format!("{root}/fabric/loader/{{version}}"),
            fabric_installers: format!("{root}/fabric/installer"),
            fabric_server: format!(
                "{root}/fabric/server/{{version}}/{{loader}}/{{installer}}/server/jar"
            ),
        })
        .expect("mock service")
    }

    fn finish(self) {
        self.handle.join().expect("mock server completed");
    }
}

fn read_request(stream: &mut TcpStream) -> String {
    let mut reader = BufReader::new(stream);
    let mut request = String::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).expect("request line");
        if line == "\r\n" || line.is_empty() {
            break;
        }
        request.push_str(&line);
    }
    request
}
