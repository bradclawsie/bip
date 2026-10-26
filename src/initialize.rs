use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, ensure};

use crate::bootstrap;
use crate::git::{self, Repository};
use crate::metadata::{IDENTITY_PATH, Identity, SIGNERS_PATH};
use crate::signing::{self, SigningKey};

pub fn initialize(repo: &Repository, path: &Path) -> Result<String> {
    repo.check_init()?;
    ensure!(
        bootstrap::candidates(repo)?.is_empty(),
        "Existing bip initialization marker; repeated initialization is forbidden"
    );
    let key = SigningKey::read(path)?;
    let identity = Identity::new(&repo.main)?.encode()?;
    let signers = crate::metadata::signer_entry(&key.public).into_bytes();
    let directory = tempfile::tempdir()?;
    let index = directory.path().join("index");
    git::checked(
        repo.command()
            .env("GIT_INDEX_FILE", &index)
            .args(["read-tree", &repo.main]),
        None,
    )?;
    for (path, contents) in [(IDENTITY_PATH, &identity), (SIGNERS_PATH, &signers)] {
        let blob = git::utf8(git::checked(
            repo.command().args(["hash-object", "-w", "--stdin"]),
            Some(contents),
        )?)?;
        git::checked(
            repo.command().env("GIT_INDEX_FILE", &index).args([
                "update-index",
                "--add",
                "--cacheinfo",
                &format!("100644,{blob},{path}"),
            ]),
            None,
        )?;
    }
    let tree = git::utf8(git::checked(
        repo.command()
            .env("GIT_INDEX_FILE", &index)
            .arg("write-tree"),
        None,
    )?)?;
    let oid = signing::sign(repo, &key, &tree)?;
    let committed_signers = bootstrap::validate(repo, &oid)?;
    signing::verify(repo, &oid, &committed_signers)?;
    repo.check_init()?;
    publish(repo, &oid, &fs::read(index)?, &identity, &signers)?;
    Ok(oid)
}

// Own only the lock and metadata created by this invocation. Never reset a checkout.
struct Publication<'a> {
    repo: &'a Repository,
    index_path: PathBuf,
    lock_path: PathBuf,
    old_index: Option<Vec<u8>>,
    new_index: Vec<u8>,
    index_installed: bool,
    lock_owned: bool,
    directory_created: bool,
    files: Vec<(PathBuf, Vec<u8>)>,
}

impl Publication<'_> {
    fn cleanup(&mut self) -> Result<()> {
        let mut failures = Vec::new();
        if self.index_installed {
            let restore = (|| -> Result<()> {
                let mut lock = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&self.lock_path)?;
                self.lock_owned = true;
                ensure!(
                    fs::read(&self.index_path)? == self.new_index,
                    "Index changed after publication; preserving it"
                );
                if let Some(old) = &self.old_index {
                    lock.write_all(old)?;
                    drop(lock);
                    fs::rename(&self.lock_path, &self.index_path)?;
                    self.lock_owned = false;
                } else {
                    fs::remove_file(&self.index_path)?;
                }
                Ok(())
            })();
            if let Err(error) = restore {
                failures.push(format!("index: {error:#}"));
            }
        }
        if self.lock_owned {
            match fs::remove_file(&self.lock_path) {
                Ok(()) => self.lock_owned = false,
                Err(error) => failures.push(format!("{}: {error}", self.lock_path.display())),
            }
        }
        for (path, contents) in self.files.iter().rev() {
            let remove = (|| -> Result<()> {
                ensure!(
                    fs::symlink_metadata(path)?.file_type().is_file()
                        && fs::read(path)? == *contents,
                    "File changed; preserving it"
                );
                fs::remove_file(path)?;
                Ok(())
            })();
            if let Err(error) = remove {
                failures.push(format!("{}: {error:#}", path.display()));
            }
        }
        if self.directory_created
            && let Err(error) = fs::remove_dir(self.repo.root.join(".bip"))
        {
            failures.push(format!(".bip directory: {error}"));
        }
        ensure!(
            failures.is_empty(),
            "Incomplete cleanup: {}. Inspect git status, the index, and .bip; preserve your changes before removing leftover bootstrap files or locks",
            failures.join("; ")
        );
        Ok(())
    }
}

fn publish(
    repo: &Repository,
    oid: &str,
    new_index: &[u8],
    identity: &[u8],
    signers: &[u8],
) -> Result<()> {
    let index_path = PathBuf::from(repo.text(&[
        "rev-parse",
        "--path-format=absolute",
        "--git-path",
        "index",
    ])?);
    let mut lock_name = index_path.as_os_str().to_os_string();
    lock_name.push(".lock");
    let lock_path = PathBuf::from(lock_name);
    let mut lock = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock_path)
        .context(
            "Cannot lock Git index; finish other Git operations and inspect any stale index.lock",
        )?;
    let mut publication = Publication {
        repo,
        old_index: None,
        index_path,
        lock_path,
        new_index: new_index.to_vec(),
        index_installed: false,
        lock_owned: true,
        directory_created: false,
        files: Vec::new(),
    };
    let result = (|| -> Result<()> {
        repo.check_init()?;
        publication.old_index = match fs::read(&publication.index_path) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        };
        lock.write_all(new_index)?;
        lock.sync_all()?;
        fs::create_dir(repo.root.join(".bip"))?;
        publication.directory_created = true;
        for (name, contents) in [(IDENTITY_PATH, identity), (SIGNERS_PATH, signers)] {
            let path = repo.root.join(name);
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)?;
            // Record ownership before writing so a short write is also cleaned up.
            publication.files.push((path, Vec::new()));
            let write = file.write_all(contents);
            let entry = publication
                .files
                .last_mut()
                .context("Missing publication file")?;
            entry.1 = fs::read(&entry.0)?;
            write?;
        }
        git::checked(
            repo.command().env("GIT_INDEX_FILE", &publication.lock_path).args(["diff", "--quiet", "--no-ext-diff", "--ignore-submodules=none", "--"]),
            None,
        ).context("Working tree does not match the prepared bootstrap; inspect Git attributes, filters, or concurrent edits")?;
        // Git diff can refresh index stat data. Keep that work on our private
        // index and record the resulting bytes before installing it.
        drop(lock);
        publication.new_index = fs::read(&publication.lock_path)?;
        fs::rename(&publication.lock_path, &publication.index_path)?;
        publication.lock_owned = false;
        publication.index_installed = true;
        let hooks = tempfile::tempdir()?;
        git::checked(
            repo.command()
                .arg("-c")
                .arg(format!("core.hooksPath={}", hooks.path().display()))
                .args([
                    "update-ref",
                    "-m",
                    "bip initialization",
                    "refs/heads/main",
                    oid,
                    &repo.main,
                ]),
            None,
        )
        .context("Could not publish initialization; main may have changed")?;
        Ok(())
    })();
    if let Err(error) = result {
        if let Err(cleanup) = publication.cleanup() {
            return Err(anyhow!("{error:#}; {cleanup:#}"));
        }
        return Err(error);
    }
    Ok(())
}
