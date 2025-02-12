mod fixtures;

use assert_cmd::prelude::*;
use assert_fs::fixture::TempDir;
use fixtures::{port, tmpdir, Error, FILES};
use reqwest::StatusCode;
use rstest::{rstest, rstest_parametrize};
use select::document::Document;
use select::predicate::Text;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

#[rstest_parametrize(
    cli_auth_arg, client_username, client_password,
    case("testuser:testpassword", "testuser", "testpassword"),
    case(
        "testuser:sha256:9f735e0df9a1ddc702bf0a1a7b83033f9f7153a00c29de82cedadc9957289b05",
        "testuser",
        "testpassword"
    ),
    case(
        "testuser:sha512:e9e633097ab9ceb3e48ec3f70ee2beba41d05d5420efee5da85f97d97005727587fda33ef4ff2322088f4c79e8133cc9cd9f3512f4d3a303cbdb5bc585415a00",
        "testuser",
        "testpassword"
    ),
)]
fn auth_accepts(
    tmpdir: TempDir,
    port: u16,
    cli_auth_arg: &str,
    client_username: &str,
    client_password: &str,
) -> Result<(), Error> {
    let mut child = Command::cargo_bin("miniserve")?
        .arg(tmpdir.path())
        .arg("-p")
        .arg(port.to_string())
        .arg("-a")
        .arg(cli_auth_arg)
        .stdout(Stdio::null())
        .spawn()?;

    sleep(Duration::from_secs(1));

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://localhost:{}", port).as_str())
        .basic_auth(client_username, Some(client_password))
        .send()?;

    let status_code = response.status();
    assert_eq!(status_code, StatusCode::OK);

    let body = response.error_for_status()?;
    let parsed = Document::from_read(body)?;
    for &file in FILES {
        assert!(parsed.find(Text).any(|x| x.text() == file));
    }

    child.kill()?;

    Ok(())
}

#[rstest_parametrize(
    cli_auth_arg, client_username, client_password,
    case("rightuser:rightpassword", "wronguser", "rightpassword"),
    case(
        "rightuser:sha256:314eee236177a721d0e58d3ca4ff01795cdcad1e8478ba8183a2e58d69c648c0",
        "wronguser",
        "rightpassword"
    ),
    case(
        "rightuser:sha512:84ec4056571afeec9f5b59453305877e9a66c3f9a1d91733fde759b370c1d540b9dc58bfc88c5980ad2d020c3a8ee84f21314a180856f5a82ba29ecba29e2cab",
        "wronguser",
        "rightpassword"
    ),
    case("rightuser:rightpassword", "rightuser", "wrongpassword"),
    case(
        "rightuser:sha256:314eee236177a721d0e58d3ca4ff01795cdcad1e8478ba8183a2e58d69c648c0",
        "rightuser",
        "wrongpassword"
    ),
    case(
        "rightuser:sha512:84ec4056571afeec9f5b59453305877e9a66c3f9a1d91733fde759b370c1d540b9dc58bfc88c5980ad2d020c3a8ee84f21314a180856f5a82ba29ecba29e2cab",
        "rightuser",
        "wrongpassword"
    ),
)]
fn auth_rejects(
    tmpdir: TempDir,
    port: u16,
    cli_auth_arg: &str,
    client_username: &str,
    client_password: &str,
) -> Result<(), Error> {
    let mut child = Command::cargo_bin("miniserve")?
        .arg(tmpdir.path())
        .arg("-p")
        .arg(port.to_string())
        .arg("-a")
        .arg(cli_auth_arg)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    sleep(Duration::from_secs(1));

    let client = reqwest::Client::new();
    let status = client
        .get(format!("http://localhost:{}", port).as_str())
        .basic_auth(client_username, Some(client_password))
        .send()?
        .status();

    assert_eq!(status, StatusCode::UNAUTHORIZED);

    child.kill()?;

    Ok(())
}

/// Helper functions that register multiple accounts
#[cfg(test)]
fn register_accounts<'a>(command: &'a mut Command) -> &'a mut Command {
    command
        .arg("--auth")
        .arg("usr0:pwd0")
        .arg("--auth")
        .arg("usr1:pwd1")
        .arg("--auth")
        .arg("usr2:sha256:149d2937d1bce53fa683ae652291bd54cc8754444216a9e278b45776b76375af") // pwd2
        .arg("--auth")
        .arg("usr3:sha256:ffc169417b4146cebe09a3e9ffbca33db82e3e593b4d04c0959a89c05b87e15d") // pwd3
        .arg("--auth")
        .arg("usr4:sha512:68050a967d061ac480b414bc8f9a6d368ad0082203edcd23860e94c36178aad1a038e061716707d5479e23081a6d920dc6e9f88e5eb789cdd23e211d718d161a") // pwd4
        .arg("--auth")
        .arg("usr5:sha512:be82a7dccd06122f9e232e9730e67e69e30ec61b268fd9b21a5e5d42db770d45586a1ce47816649a0107e9fadf079d9cf0104f0a3aaa0f67bad80289c3ba25a8") // pwd5
}

#[rstest_parametrize(
    username, password,
    case("usr0", "pwd0"),
    case("usr1", "pwd1"),
    case("usr2", "pwd2"),
    case("usr3", "pwd3"),
    case("usr4", "pwd4"),
    case("usr5", "pwd5"),
)]
fn auth_multiple_accounts_pass(
    tmpdir: TempDir,
    port: u16,
    username: &str,
    password: &str,
) -> Result<(), Error> {
    let mut child = register_accounts(
        &mut Command::cargo_bin("miniserve")?
    )
        .arg("-p")
        .arg(port.to_string())
        .arg(tmpdir.path())
        .stdout(Stdio::null())
        .spawn()?;

    sleep(Duration::from_secs(1));

    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://localhost:{}", port).as_str())
        .basic_auth(username, Some(password))
        .send()?;

    let status = response.status();
    assert_eq!(status, StatusCode::OK);

    let body = response.error_for_status()?;
    let parsed = Document::from_read(body)?;
    for &file in FILES {
        assert!(parsed.find(Text).any(|x| x.text() == file));
    }

    child.kill()?;

    Ok(())
}

#[rstest]
fn auth_multiple_accounts_wrong_username(
    tmpdir: TempDir,
    port: u16
) -> Result<(), Error> {
    let mut child = register_accounts(
        Command::cargo_bin("miniserve")?
            .arg(tmpdir.path())
            .arg("-p")
            .arg(port.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
    )
        .spawn()?;

    sleep(Duration::from_secs(1));

    let client = reqwest::Client::new();

    let status = client
        .get(format!("http://localhost:{}", port).as_str())
        .basic_auth("unregistered user", Some("pwd0"))
        .send()?
        .status();

    assert_eq!(status, StatusCode::UNAUTHORIZED);

    child.kill()?;

    Ok(())
}

#[rstest_parametrize(
    username, password,
    case("usr0", "pwd5"),
    case("usr1", "pwd4"),
    case("usr2", "pwd3"),
    case("usr3", "pwd2"),
    case("usr4", "pwd1"),
    case("usr5", "pwd0"),
)]
fn auth_multiple_accounts_wrong_password(
    tmpdir: TempDir,
    port: u16,
    username: &str,
    password: &str,
) -> Result<(), Error> {
    let mut child = register_accounts(
        Command::cargo_bin("miniserve")?
            .arg(tmpdir.path())
            .arg("-p")
            .arg(port.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
    )
        .spawn()?;

    sleep(Duration::from_secs(1));

    let client = reqwest::Client::new();

    let status = client
        .get(format!("http://localhost:{}", port).as_str())
        .basic_auth(username, Some(password))
        .send()?
        .status();

    assert_eq!(status, StatusCode::UNAUTHORIZED);

    child.kill()?;

    Ok(())
}
