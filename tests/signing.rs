#![cfg(unix)]

mod support;

use std::fs;
use std::process::{Child, Stdio};
use std::time::Duration;

use support::{Fixture, failure, success};

struct Agent(Child);

impl Drop for Agent {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn public_key_uses_isolated_agent_without_adjacent_private_key() {
    let fixture = Fixture::new();
    fixture.commit();
    let key = fixture.key("owner");
    let socket = fixture.directory.path().join("agent.socket");
    let mut agent = Agent(
        fixture
            .command("ssh-agent")
            .args(["-D", "-a"])
            .arg(&socket)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    for _ in 0..100 {
        if socket.exists() {
            break;
        }
        if let Some(status) = agent.0.try_wait().unwrap() {
            let mut stderr = String::new();
            std::io::Read::read_to_string(&mut agent.0.stderr.take().unwrap(), &mut stderr)
                .unwrap();
            panic!("SSH agent failed ({status}): {stderr}");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(socket.exists(), "SSH agent did not start");
    success(
        fixture
            .command("ssh-add")
            .env("SSH_AUTH_SOCK", &socket)
            .arg(&key)
            .output()
            .unwrap(),
    );
    fs::remove_file(&key).unwrap();
    let oid = success(
        fixture
            .command(env!("CARGO_BIN_EXE_bip"))
            .env("SSH_AUTH_SOCK", &socket)
            .args(["init", "--signing-key"])
            .arg(key.with_extension("pub"))
            .output()
            .unwrap(),
    );
    assert!(
        success(fixture.bip(&["verify", "--expected-id", oid.trim()]))
            .contains("Initialization verified")
    );
}

#[test]
fn signing_failures_and_missing_tools_are_actionable() {
    let fixture = Fixture::new();
    fixture.commit();
    let key = fixture.key("owner");
    let before = fixture.snapshot();
    let tools = fixture.shim(
        "ssh-keygen",
        "echo 'required ssh-keygen unavailable' >&2\nexit 127",
    );
    failure(
        fixture
            .with_tools(&tools)
            .args(["init", "--signing-key"])
            .arg(&key)
            .output()
            .unwrap(),
        "ssh-keygen unavailable",
    );
    assert_eq!(fixture.snapshot(), before);
    fixture.shim("ssh-keygen", "if [ \"$1\" = '-Y' ] && [ \"$2\" = 'sign' ]; then echo 'simulated signing failure' >&2; exit 1; fi\nexec \"$BIP_TEST_SSH_KEYGEN\" \"$@\"");
    failure(
        fixture
            .with_tools(&tools)
            .args(["init", "--signing-key"])
            .arg(&key)
            .output()
            .unwrap(),
        "Could not create SSH-signed",
    );
    assert_eq!(fixture.snapshot(), before);
    assert!(!fixture.repo.join(".bip").exists());
    let empty = fixture.directory.path().join("empty-path");
    fs::create_dir(&empty).unwrap();
    failure(
        fixture
            .command(env!("CARGO_BIN_EXE_bip"))
            .env("PATH", empty)
            .arg("verify")
            .output()
            .unwrap(),
        "install Git and OpenSSH",
    );
}
