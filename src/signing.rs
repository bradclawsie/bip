use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, ensure};

use crate::git::{self, Repository};
use crate::metadata;

pub struct SigningKey {
    pub path: PathBuf,
    pub public: String,
}

impl SigningKey {
    pub fn read(path: &Path) -> Result<Self> {
        let path = path
            .canonicalize()
            .context("Cannot access selected SSH signing key")?;
        let bytes = fs::read(&path).context("Cannot read selected SSH signing key")?;
        let text =
            std::str::from_utf8(&bytes).context("Selected SSH key must be an OpenSSH key file")?;
        let public = if text.trim_start().starts_with("-----BEGIN ") {
            // OpenSSH handles the private format and any passphrase interaction.
            let output = git::checked(
                Command::new("ssh-keygen").args(["-y", "-f"]).arg(&path),
                None,
            )?;
            metadata::public_key(std::str::from_utf8(&output)?)?
        } else {
            ensure!(
                text.lines().filter(|line| !line.trim().is_empty()).count() == 1,
                "Select a file containing exactly one SSH public key"
            );
            metadata::public_key(text)?
        };
        Ok(Self { path, public })
    }
}

pub fn sign(repo: &Repository, key: &SigningKey, tree: &str) -> Result<String> {
    let mut command = repo.command();
    command
        .args([
            "-c",
            "gpg.format=ssh",
            "-c",
            "gpg.ssh.program=ssh-keygen",
            "-c",
        ])
        .arg(format!("user.signingkey={}", key.path.display()))
        .args([
            "commit-tree",
            "-S",
            tree,
            "-p",
            &repo.main,
            "-m",
            metadata::SUBJECT,
        ]);
    git::utf8(git::checked(&mut command, None).context("Could not create SSH-signed initialization commit; check Git/OpenSSH signing support and the selected key or agent")?)
}

pub fn verify(repo: &Repository, oid: &str, signers: &[u8]) -> Result<()> {
    let commit = repo.read(&["cat-file", "commit", oid])?;
    let header_end = commit
        .windows(2)
        .position(|bytes| bytes == b"\n\n")
        .context("Malformed commit")?;
    let signature_header: &[u8] = if repo.oid_length == 64 {
        b"gpgsig-sha256 -----BEGIN SSH SIGNATURE-----"
    } else {
        b"gpgsig -----BEGIN SSH SIGNATURE-----"
    };
    ensure!(
        commit[..header_end]
            .split(|byte| *byte == b'\n')
            .any(|line| line == signature_header),
        "Initialization requires an SSH signature"
    );
    let directory = tempfile::tempdir()?;
    let allowed = directory.path().join("allowed_signers");
    let revoked = directory.path().join("revoked");
    fs::write(&allowed, signers)?;
    fs::write(&revoked, b"")?;
    let mut command = repo.command();
    command
        .args([
            "-c",
            "gpg.format=ssh",
            "-c",
            "gpg.ssh.program=ssh-keygen",
            "-c",
        ])
        .arg(format!("gpg.ssh.allowedSignersFile={}", allowed.display()))
        .arg("-c")
        .arg(format!("gpg.ssh.revocationFile={}", revoked.display()))
        .args(["verify-commit", oid]);
    git::checked(&mut command, None)
        .context("Initialization SSH signature did not verify against the declared first owner")?;
    Ok(())
}
