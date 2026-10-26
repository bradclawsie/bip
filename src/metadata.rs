use std::fs;
use std::process::Command;

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, UtcOffset, format_description::well_known::Rfc3339};

use crate::git;

pub const IDENTITY_PATH: &str = ".bip/identity.toml";
pub const SIGNERS_PATH: &str = ".bip/allowed_signers";
pub const SUBJECT: &str = "bip! init v1";
pub const MARKER: &[u8] = b"bip! init";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    pub format_version: u32,
    pub initialized_at: String,
    pub parent_commit: String,
}

impl Identity {
    pub fn new(parent: &str) -> Result<Self> {
        Ok(Self {
            format_version: 1,
            initialized_at: OffsetDateTime::now_utc().format(&Rfc3339)?,
            parent_commit: parent.to_owned(),
        })
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        Ok(toml::to_string(self)?.into_bytes())
    }

    pub fn decode(bytes: &[u8], oid_length: usize) -> Result<Self> {
        let identity: Self = toml::from_str(std::str::from_utf8(bytes)?)
            .context("Invalid bootstrap identity metadata")?;
        ensure!(
            identity.format_version == 1,
            "Unsupported bootstrap metadata version: {}",
            identity.format_version
        );
        let timestamp = OffsetDateTime::parse(&identity.initialized_at, &Rfc3339)
            .context("Invalid UTC initialization timestamp")?;
        ensure!(
            timestamp.offset() == UtcOffset::UTC,
            "Initialization timestamp must be UTC"
        );
        ensure!(
            identity.parent_commit.len() == oid_length
                && identity
                    .parent_commit
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit()),
            "Invalid full parent commit hash"
        );
        Ok(identity)
    }
}

pub fn public_key(text: &str) -> Result<String> {
    let mut fields = text.split_whitespace();
    let kind = fields.next().context("Missing SSH public key type")?;
    let data = fields.next().context("Missing SSH public key data")?;
    ensure!(
        !kind.contains("-cert-"),
        "SSH certificates are unsupported in bootstrap version 1"
    );
    let key = format!("{kind} {data}");
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("owner.pub");
    fs::write(&path, format!("{key}\n"))?;
    git::checked(
        Command::new("ssh-keygen").args(["-l", "-f"]).arg(&path),
        None,
    )
    .context("Invalid SSH public key or unavailable ssh-keygen")?;
    Ok(key)
}

pub fn signer_entry(key: &str) -> String {
    format!("bip-owner namespaces=\"git\" {key}\n")
}

pub fn validate_signers(bytes: &[u8]) -> Result<()> {
    let text = std::str::from_utf8(bytes).context("Allowed signers must be UTF-8")?;
    let entries: Vec<_> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    ensure!(
        entries.len() == 1,
        "Bootstrap allowed_signers must contain exactly one owner entry"
    );
    let fields: Vec<_> = entries[0].split_whitespace().collect();
    ensure!(
        fields.len() >= 4 && fields[0] == "bip-owner" && fields[1] == "namespaces=\"git\"",
        "Bootstrap owner entry must use principal bip-owner and only namespaces=\"git\""
    );
    public_key(&format!("{} {}", fields[2], fields[3]))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Identity, public_key, signer_entry, validate_signers};
    use crate::git;
    use std::fs;
    use std::process::Command;

    #[test]
    fn identity_format_validation() {
        let identity = Identity::new(&"a".repeat(40)).unwrap();
        let valid = String::from_utf8(identity.encode().unwrap()).unwrap();
        Identity::decode(valid.as_bytes(), 40).unwrap();
        for invalid in [
            valid.replace("format_version = 1", "format_version = 2"),
            valid.replace("format_version = 1", "format_version = \"1\""),
            format!("{valid}extra = 1\n"),
            valid.replace(&identity.initialized_at, "yesterday"),
            valid.replace(&identity.initialized_at, "2026-01-01T00:00:00+01:00"),
            valid.replace(&identity.parent_commit, "abcd"),
            valid.replace(&identity.parent_commit, &"x".repeat(40)),
        ] {
            assert!(
                Identity::decode(invalid.as_bytes(), 40).is_err(),
                "{invalid}"
            );
        }
        assert!(Identity::decode(valid.as_bytes(), 64).is_err());
    }

    #[test]
    fn signer_format_validation() {
        let directory = tempfile::tempdir().unwrap();
        let key_path = directory.path().join("key");
        git::checked(
            Command::new("ssh-keygen")
                .args(["-q", "-t", "ed25519", "-N", "", "-f"])
                .arg(&key_path),
            None,
        )
        .unwrap();
        let key = public_key(&fs::read_to_string(key_path.with_extension("pub")).unwrap()).unwrap();
        let valid = signer_entry(&key);
        validate_signers(format!("# owner\n\n{valid}\n").as_bytes()).unwrap();
        for invalid in [
            String::new(),
            format!("{valid}{valid}"),
            valid.replace("bip-owner", "*"),
            valid.replace("namespaces=\"git\"", "namespaces=\"other\""),
            valid.replace("namespaces=\"git\"", "cert-authority namespaces=\"git\""),
            valid.replace("ssh-ed25519", "ssh-ed25519-cert-v01@openssh.com"),
            "bip-owner namespaces=\"git\" ssh-ed25519 invalid\n".to_owned(),
        ] {
            assert!(validate_signers(invalid.as_bytes()).is_err(), "{invalid}");
        }
    }
}
