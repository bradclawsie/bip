mod support;

use std::fs;
use support::{Fixture, failure, success};

#[test]
fn repository_requirements() {
    let fixture = Fixture::new();
    failure(
        fixture.bip(&["init", "--signing-key", "unused"]),
        "first Git commit",
    );
    fixture.commit();
    fixture.git(&["branch", "-m", "other"]);
    fixture.git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    failure(fixture.bip(&["verify"]), "local main branch");
    fixture.git(&["branch", "main"]);
    failure(
        fixture.bip(&["init", "--signing-key", "unused"]),
        "Check out main",
    );
    fixture.git(&["checkout", "--detach"]);
    failure(
        fixture.bip(&["init", "--signing-key", "unused"]),
        "Check out main",
    );
    fixture.git(&["checkout", "main"]);
    let subdir = fixture.repo.join("nested");
    fs::create_dir(&subdir).unwrap();
    // A subdirectory resolves the same repository and reaches later checks.
    failure(
        fixture
            .command(env!("CARGO_BIN_EXE_bip"))
            .current_dir(subdir)
            .args(["init", "--signing-key", "unused"])
            .output()
            .unwrap(),
        "Cannot access selected SSH signing key",
    );
    failure(
        fixture
            .command(env!("CARGO_BIN_EXE_bip"))
            .current_dir(&fixture.home)
            .arg("verify")
            .output()
            .unwrap(),
        "existing Git repository",
    );

    let bare = fixture.directory.path().join("bare");
    success(
        fixture
            .command("git")
            .args(["clone", "--bare"])
            .arg(&fixture.repo)
            .arg(&bare)
            .output()
            .unwrap(),
    );
    failure(
        fixture
            .command(env!("CARGO_BIN_EXE_bip"))
            .current_dir(bare)
            .arg("verify")
            .output()
            .unwrap(),
        "Bare repositories",
    );
    let shallow = fixture.directory.path().join("shallow");
    success(
        fixture
            .command("git")
            .args(["clone", "--no-local", "--depth=1"])
            .arg(&fixture.repo)
            .arg(&shallow)
            .output()
            .unwrap(),
    );
    failure(
        fixture
            .command(env!("CARGO_BIN_EXE_bip"))
            .current_dir(shallow)
            .arg("verify")
            .output()
            .unwrap(),
        "complete history",
    );
}

#[test]
fn initialization_preserves_dirty_and_existing_state() {
    let fixture = Fixture::new();
    fixture.commit();
    fs::write(fixture.repo.join("file"), "original").unwrap();
    let before = fixture.snapshot();
    failure(
        fixture.bip(&["init", "--signing-key", "unused"]),
        "clean index",
    );
    assert_eq!(fixture.snapshot(), before);
    fixture.git(&["add", "file"]);
    let before = fixture.snapshot();
    failure(
        fixture.bip(&["init", "--signing-key", "unused"]),
        "clean index",
    );
    assert_eq!(fixture.snapshot(), before);
    fixture.git(&["commit", "-m", "Track file"]);
    fs::write(fixture.repo.join("file"), "edited").unwrap();
    let before = fixture.snapshot();
    failure(
        fixture.bip(&["init", "--signing-key", "unused"]),
        "clean index",
    );
    assert_eq!(fixture.snapshot(), before);
    fixture.git(&["checkout", "--", "file"]);
    fs::write(fixture.repo.join(".gitignore"), ".bip\n").unwrap();
    fixture.git(&["add", ".gitignore"]);
    fixture.git(&["commit", "-m", "Ignore metadata"]);
    fs::create_dir(fixture.repo.join(".bip")).unwrap();
    failure(
        fixture.bip(&["init", "--signing-key", "unused"]),
        "will not be overwritten",
    );
    assert!(fixture.repo.join(".bip").is_dir());
}

#[cfg(unix)]
#[test]
fn dangling_metadata_symlink_is_not_overwritten() {
    let fixture = Fixture::new();
    fixture.commit();
    std::os::unix::fs::symlink("missing", fixture.repo.join(".bip")).unwrap();
    failure(
        fixture.bip(&["init", "--signing-key", "unused"]),
        "will not be overwritten",
    );
    assert_eq!(
        fs::read_link(fixture.repo.join(".bip")).unwrap().to_str(),
        Some("missing")
    );
}

#[test]
fn ignored_bootstrap_paths_prevent_initialization_without_changes() {
    for (source, pattern, affected) in [
        ("repository", ".bip/\n", ".bip/identity.toml"),
        (
            "repository",
            ".bip/allowed_signers\n",
            ".bip/allowed_signers",
        ),
        ("repository", "*.toml\n", ".bip/identity.toml"),
        ("local", ".bip\n", ".bip/identity.toml"),
        ("global", ".bip/\n", ".bip/identity.toml"),
    ] {
        let fixture = Fixture::new();
        fixture.commit();
        match source {
            "repository" => {
                fs::write(fixture.repo.join(".gitignore"), pattern).unwrap();
                fixture.git(&["add", ".gitignore"]);
                fixture.git(&["commit", "-m", "Add ignore rule"]);
            }
            "local" => fs::write(fixture.repo.join(".git/info/exclude"), pattern).unwrap(),
            "global" => {
                let path = fixture.home.join("global-ignore");
                fs::write(&path, pattern).unwrap();
                fixture.git(&["config", "core.excludesFile", path.to_str().unwrap()]);
            }
            _ => unreachable!(),
        }
        let before = fixture.snapshot();
        let output = fixture.bip(&["init", "--signing-key", "unused"]);
        assert!(!output.status.success());
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(
            error.contains("Git is configured to ignore bip metadata"),
            "{source}: {error}"
        );
        assert!(error.contains(affected), "{source}: {error}");
        assert!(
            error.contains("git check-ignore -v --no-index"),
            "{source}: {error}"
        );
        assert!(output.stdout.is_empty());
        assert_eq!(fixture.snapshot(), before);
        assert!(!fixture.repo.join(".bip").exists());
    }
}

#[test]
fn explicit_ignore_exceptions_allow_bootstrap() {
    let fixture = Fixture::new();
    fixture.commit();
    fs::write(
        fixture.repo.join(".gitignore"),
        "*\n!.gitignore\n!.bip/\n!.bip/identity.toml\n!.bip/allowed_signers\n",
    )
    .unwrap();
    fixture.git(&["add", ".gitignore"]);
    fixture.git(&["commit", "-m", "Ignore with bootstrap exceptions"]);
    let key = fixture.key("owner");
    let oid = success(fixture.bip(&["init", "--signing-key", key.to_str().unwrap()]));
    assert!(
        success(fixture.bip(&["verify", "--expected-id", oid.trim()]))
            .contains("Expected identity matched")
    );
}

#[cfg(unix)]
#[test]
fn ignore_check_errors_abort_initialization() {
    let fixture = Fixture::new();
    fixture.commit();
    let before = fixture.snapshot();
    let tools = fixture.shim(
        "git",
        "for arg in \"$@\"; do\n if [ \"$arg\" = check-ignore ]; then echo 'simulated ignore failure' >&2; exit 2; fi\ndone\nexec \"$BIP_TEST_GIT\" \"$@\"",
    );
    let output = fixture
        .with_tools(&tools)
        .args(["init", "--signing-key", "unused"])
        .output()
        .unwrap();
    failure(output, "Could not check Git ignore rules");
    assert_eq!(fixture.snapshot(), before);
    assert!(!fixture.repo.join(".bip").exists());
}
