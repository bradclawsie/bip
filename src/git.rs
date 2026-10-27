use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use anyhow::{Context, Result, bail, ensure};

use crate::metadata::{IDENTITY_PATH, SIGNERS_PATH};

pub struct Repository {
    pub root: PathBuf,
    pub main: String,
    pub oid_length: usize,
}

pub fn git_command(directory: &Path) -> Command {
    let mut command = Command::new("git");
    command
        .current_dir(directory)
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .env("GIT_NO_LAZY_FETCH", "1")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .args(["-c", "core.fsmonitor=false"]);
    command
}

pub fn output(command: &mut Command, input: Option<&[u8]>) -> Result<Output> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    command.stdin(if input.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    });
    let mut child = command.spawn().context(
        "Could not start required Git/OpenSSH tool; install Git and OpenSSH and check PATH",
    )?;
    if let Some(input) = input {
        child
            .stdin
            .take()
            .context("Missing child input pipe")?
            .write_all(input)?;
    }
    Ok(child.wait_with_output()?)
}

pub fn checked(command: &mut Command, input: Option<&[u8]>) -> Result<Vec<u8>> {
    let result = output(command, input)?;
    ensure!(
        result.status.success(),
        "Git/OpenSSH operation failed: {}",
        String::from_utf8_lossy(&result.stderr).trim()
    );
    Ok(result.stdout)
}

pub fn utf8(bytes: Vec<u8>) -> Result<String> {
    Ok(String::from_utf8(bytes)
        .context("Expected UTF-8 Git/bootstrap data")?
        .trim_end_matches('\n')
        .to_owned())
}

pub fn path_exists(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).with_context(|| format!("Cannot inspect {}", path.display())),
    }
}

impl Repository {
    pub fn discover() -> Result<Self> {
        let cwd = std::env::current_dir()?;
        let bare = output(
            git_command(&cwd).args(["rev-parse", "--is-bare-repository"]),
            None,
        )?;
        ensure!(
            bare.status.success(),
            "An existing Git repository is required; initialize or enter one first"
        );
        ensure!(
            utf8(bare.stdout)? != "true",
            "Bare repositories are unsupported; use a working-tree checkout"
        );
        let root = PathBuf::from(utf8(checked(
            git_command(&cwd).args(["rev-parse", "--show-toplevel"]),
            None,
        )?)?);
        ensure!(
            utf8(checked(
                git_command(&root).args(["rev-parse", "--is-shallow-repository"]),
                None
            )?)? == "false",
            "Shallow repositories are unsupported; obtain complete history first (git fetch --unshallow)"
        );
        let grafts = PathBuf::from(utf8(checked(
            git_command(&root).args([
                "rev-parse",
                "--path-format=absolute",
                "--git-path",
                "info/grafts",
            ]),
            None,
        )?)?);
        ensure!(
            !path_exists(&grafts)? && std::env::var_os("GIT_GRAFT_FILE").is_none(),
            "Legacy Git graft overrides are unsupported; remove the graft override before using bip"
        );
        let main = output(
            git_command(&root).args(["show-ref", "--verify", "--hash", "refs/heads/main"]),
            None,
        )?;
        if !main.status.success() {
            let head = output(
                git_command(&root).args(["symbolic-ref", "--quiet", "HEAD"]),
                None,
            )?;
            if head.status.success() && utf8(head.stdout)? == "refs/heads/main" {
                bail!(
                    "Create your first Git commit on main to anchor bip initialization, then run bip init again"
                );
            }
            bail!(
                "A local main branch is required; create or rename the intended branch to main and check it out"
            );
        }
        let format = utf8(checked(
            git_command(&root).args(["rev-parse", "--show-object-format"]),
            None,
        )?)?;
        let oid_length = match format.as_str() {
            "sha1" => 40,
            "sha256" => 64,
            _ => bail!("Unsupported Git object format: {format}"),
        };
        Ok(Self {
            root,
            main: utf8(main.stdout)?,
            oid_length,
        })
    }

    pub fn command(&self) -> Command {
        git_command(&self.root)
    }

    pub fn read(&self, args: &[&str]) -> Result<Vec<u8>> {
        checked(self.command().args(args), None)
    }

    pub fn text(&self, args: &[&str]) -> Result<String> {
        utf8(self.read(args)?)
    }

    pub fn valid_oid(&self, oid: &str) -> bool {
        oid.len() == self.oid_length && oid.bytes().all(|byte| byte.is_ascii_hexdigit())
    }

    pub fn check_init(&self) -> Result<()> {
        let head = output(
            self.command().args(["symbolic-ref", "--quiet", "HEAD"]),
            None,
        )?;
        ensure!(
            head.status.success() && utf8(head.stdout)? == "refs/heads/main",
            "Check out main before running bip init"
        );
        ensure!(
            self.text(&["rev-parse", "refs/heads/main"])? == self.main,
            "main changed during initialization; retry from the new tip"
        );
        ensure!(
            !path_exists(&self.root.join(".bip"))?,
            "Existing .bip metadata will not be overwritten; inspect it before initializing"
        );
        for path in [IDENTITY_PATH, SIGNERS_PATH] {
            let result = output(
                self.command()
                    .args(["check-ignore", "--no-index", "--quiet", "--", path]),
                None,
            )?;
            match result.status.code() {
                Some(0) => bail!(
                    "Git is configured to ignore bip metadata ({path}). Update the relevant ignore rule or add an exception for .bip, then run bip init again. To find the rule, run from the repository root: git check-ignore -v --no-index -- {path}"
                ),
                Some(1) => (),
                _ => bail!(
                    "Could not check Git ignore rules for {path}: {}",
                    String::from_utf8_lossy(&result.stderr).trim()
                ),
            }
        }
        ensure!(
            self.read(&[
                "status",
                "--porcelain=v1",
                "-z",
                "--untracked-files=all",
                "--ignore-submodules=none"
            ])?
            .is_empty(),
            "A clean index and working tree are required; commit, stash, or otherwise resolve changes before bip init"
        );
        Ok(())
    }
}
