#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

pub struct Fixture {
    pub directory: TempDir,
    pub repo: PathBuf,
    pub home: PathBuf,
}

impl Fixture {
    pub fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let home = directory.path().join("home");
        let repo = directory.path().join("repo");
        fs::create_dir(&home).unwrap();
        fs::create_dir(&repo).unwrap();
        let fixture = Self {
            directory,
            repo,
            home,
        };
        fixture.git(&["init", "-b", "main"]);
        fixture.git(&["config", "user.name", "Test Owner"]);
        fixture.git(&["config", "user.email", "owner@example.test"]);
        fixture
    }

    pub fn command(&self, program: &str) -> Command {
        let mut command = Command::new(program);
        for (key, _) in std::env::vars_os() {
            if key.to_string_lossy().starts_with("GIT_")
                || key == "SSH_AUTH_SOCK"
                || key == "SSH_AGENT_PID"
            {
                command.env_remove(key);
            }
        }
        command
            .current_dir(&self.repo)
            .env("HOME", &self.home)
            .env("XDG_CONFIG_HOME", &self.home)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_OPTIONAL_LOCKS", "0")
            .env("GIT_CONFIG_GLOBAL", self.home.join("empty-config"))
            .env("GIT_TERMINAL_PROMPT", "0");
        command
    }

    pub fn git(&self, args: &[&str]) -> String {
        let output = self.command("git").args(args).output().unwrap();
        success(output).trim().to_owned()
    }

    pub fn commit(&self) {
        self.git(&[
            "-c",
            "commit.gpgsign=false",
            "commit",
            "--allow-empty",
            "-m",
            "Existing project",
        ]);
    }

    pub fn bip(&self, args: &[&str]) -> Output {
        self.command(env!("CARGO_BIN_EXE_bip"))
            .args(args)
            .output()
            .unwrap()
    }

    pub fn key(&self, name: &str) -> PathBuf {
        let key = self.directory.path().join(name);
        success(
            self.command("ssh-keygen")
                .args(["-q", "-t", "ed25519", "-N", "", "-f"])
                .arg(&key)
                .output()
                .unwrap(),
        );
        key
    }

    pub fn snapshot(&self) -> (String, Vec<u8>, String) {
        let index = self.git(&["rev-parse", "--git-path", "index"]);
        (
            self.git(&["rev-parse", "main"]),
            fs::read(self.repo.join(index)).unwrap_or_default(),
            self.git(&["status", "--porcelain=v1", "--untracked-files=all"]),
        )
    }

    pub fn clone_to(&self, destination: &Path) {
        success(
            self.command("git")
                .args(["clone", "--no-local"])
                .arg(&self.repo)
                .arg(destination)
                .output()
                .unwrap(),
        );
    }

    #[cfg(unix)]
    pub fn shim(&self, program: &str, script: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let directory = self.directory.path().join("tools");
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join(program);
        fs::write(&path, format!("#!/bin/sh\n{script}\n")).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        directory
    }

    #[cfg(unix)]
    pub fn with_tools(&self, directory: &Path) -> Command {
        let mut command = self.command(env!("CARGO_BIN_EXE_bip"));
        let mut paths = vec![directory.to_owned()];
        paths.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap()));
        command.env("PATH", std::env::join_paths(paths).unwrap());
        for (tool, variable) in [
            ("git", "BIP_TEST_GIT"),
            ("ssh-keygen", "BIP_TEST_SSH_KEYGEN"),
        ] {
            let output = self
                .command("sh")
                .args(["-c", &format!("command -v {tool}")])
                .output()
                .unwrap();
            command.env(variable, success(output).trim());
        }
        command
    }
}

pub fn success(output: Output) -> String {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

pub fn failure(output: Output, expected: &str) {
    assert!(
        !output.status.success(),
        "Unexpected success: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(expected),
        "Expected {expected:?}, got {stderr}"
    );
    assert!(output.stdout.is_empty());
}
