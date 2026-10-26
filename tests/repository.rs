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
