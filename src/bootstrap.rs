use anyhow::{Context, Result, bail, ensure};

use crate::git::Repository;
use crate::metadata::{self, IDENTITY_PATH, Identity, SIGNERS_PATH, SUBJECT};

fn message(commit: &[u8]) -> Result<&[u8]> {
    let start = commit
        .windows(2)
        .position(|bytes| bytes == b"\n\n")
        .context("Malformed commit headers")?
        + 2;
    Ok(&commit[start..])
}

fn subject(commit: &[u8]) -> Result<&[u8]> {
    Ok(message(commit)?
        .split(|byte| *byte == b'\n')
        .next()
        .unwrap_or_default())
}

pub fn candidates(repo: &Repository) -> Result<Vec<String>> {
    let history = repo.text(&["rev-list", "--first-parent", &repo.main])?;
    let mut found = Vec::new();
    for oid in history.lines() {
        let commit = repo.read(&["cat-file", "commit", oid])?;
        if subject(&commit)?.starts_with(metadata::MARKER) {
            found.push(oid.to_owned());
        }
    }
    Ok(found)
}

pub fn discover(repo: &Repository) -> Result<String> {
    let found = candidates(repo)?;
    match found.as_slice() {
        [] => bail!("No bip initialization was found on the first-parent history of main"),
        [oid] => Ok(oid.clone()),
        _ => bail!("Ambiguous initialization: multiple bip initialization markers on main"),
    }
}

pub fn validate(repo: &Repository, oid: &str) -> Result<Vec<u8>> {
    let commit = repo.read(&["cat-file", "commit", oid])?;
    let subject = subject(&commit)?;
    if subject != SUBJECT.as_bytes() {
        if subject.starts_with(b"bip! init v") {
            bail!("Unsupported initialization marker version");
        }
        bail!("Malformed initialization marker; expected {SUBJECT}");
    }
    let header_end = commit
        .windows(2)
        .position(|bytes| bytes == b"\n\n")
        .context("Malformed commit")?;
    let parents: Vec<_> = commit[..header_end]
        .split(|byte| *byte == b'\n')
        .filter_map(|line| line.strip_prefix(b"parent "))
        .collect();
    ensure!(
        parents.len() == 1,
        "Initialization must have exactly one parent commit"
    );
    let parent = std::str::from_utf8(parents[0])?;
    ensure!(repo.valid_oid(parent), "Invalid initialization parent hash");
    ensure!(
        repo.read(&["ls-tree", "-z", parent, "--", ".bip"])?
            .is_empty(),
        "Initialization parent already contains .bip metadata"
    );
    let changed = repo.read(&[
        "diff-tree",
        "--no-commit-id",
        "--name-only",
        "--no-renames",
        "--no-ext-diff",
        "--ignore-submodules=none",
        "-r",
        "-z",
        parent,
        oid,
    ])?;
    let mut names: Vec<_> = changed
        .split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
        .collect();
    names.sort_unstable();
    ensure!(
        names == [SIGNERS_PATH.as_bytes(), IDENTITY_PATH.as_bytes()],
        "Initialization must change only the two bootstrap files"
    );
    for path in [IDENTITY_PATH, SIGNERS_PATH] {
        let entry = repo.read(&["ls-tree", "-z", oid, "--", path])?;
        ensure!(
            entry.starts_with(b"100644 blob "),
            "Bootstrap metadata must be regular non-executable files: {path}"
        );
    }
    let identity = Identity::decode(
        &repo.read(&["show", &format!("{oid}:{IDENTITY_PATH}")])?,
        repo.oid_length,
    )?;
    ensure!(
        identity.parent_commit.eq_ignore_ascii_case(parent),
        "Recorded parent does not match initialization parent"
    );
    let signers = repo.read(&["show", &format!("{oid}:{SIGNERS_PATH}")])?;
    metadata::validate_signers(&signers)?;
    Ok(signers)
}
