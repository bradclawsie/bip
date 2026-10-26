mod support;

use std::fs;

use support::{Fixture, failure, success};

const LIMIT: &str = "Initialization verified; subsequent ownership changes and governance history were not checked.";

fn initialize(fixture: &Fixture) -> (std::path::PathBuf, String) {
    fixture.commit();
    let key = fixture.key("owner");
    let oid = success(fixture.bip(&["init", "--signing-key", key.to_str().unwrap()]))
        .trim()
        .to_owned();
    (key, oid)
}

#[test]
fn initialization_and_read_only_verification() {
    let fixture = Fixture::new();
    let (key, oid) = initialize(&fixture);
    assert_eq!(oid.len(), 40);
    assert_eq!(fixture.git(&["rev-parse", "main"]), oid);
    assert_eq!(fixture.git(&["log", "-1", "--format=%s"]), "bip! init v1");
    assert!(fixture.git(&["status", "--porcelain"]).is_empty());
    assert_eq!(
        fixture.git(&["diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD"]),
        ".bip/allowed_signers\n.bip/identity.toml"
    );
    let entry = fs::read_to_string(fixture.repo.join(".bip/allowed_signers")).unwrap();
    assert!(
        entry.contains(
            fs::read_to_string(key.with_extension("pub"))
                .unwrap()
                .split_whitespace()
                .nth(1)
                .unwrap()
        )
    );
    let before = fixture.snapshot();
    let output = success(fixture.bip(&["verify"]));
    assert!(
        output.contains(&oid) && output.contains("No expected identity") && output.contains(LIMIT)
    );
    let output = success(fixture.bip(&["verify", "--expected-id", &oid.to_uppercase()]));
    assert!(output.contains("Expected identity matched") && output.contains(LIMIT));
    failure(
        fixture.bip(&["verify", "--expected-id", &"0".repeat(40)]),
        "identity mismatch",
    );
    for invalid in [&oid[..8], "main", "HEAD~1", "--all", &"x".repeat(40)] {
        failure(
            fixture.bip(&["verify", "--expected-id", invalid]),
            "full hexadecimal object ID",
        );
    }
    assert_eq!(fixture.snapshot(), before);
    fixture.git(&["checkout", "-b", "other"]);
    fs::write(
        fixture.repo.join(".bip/allowed_signers"),
        "working tree junk",
    )
    .unwrap();
    let before = fixture.snapshot();
    let config = fs::read(fixture.repo.join(".git/config")).unwrap();
    let output = success(fixture.bip(&["verify", "--expected-id", &oid]));
    assert!(output.contains(LIMIT));
    assert_eq!(fixture.snapshot(), before);
    assert_eq!(fs::read(fixture.repo.join(".git/config")).unwrap(), config);
    assert_eq!(
        fs::read_to_string(fixture.repo.join(".bip/allowed_signers")).unwrap(),
        "working tree junk"
    );
}

#[test]
fn clone_verifies_without_private_key_or_agent() {
    let fixture = Fixture::new();
    let (key, oid) = initialize(&fixture);
    let clone = fixture.directory.path().join("clone");
    fixture.clone_to(&clone);
    fs::remove_file(key).unwrap();
    for directory in [&fixture.repo, &clone] {
        for expected in [false, true] {
            let mut command = fixture.command(env!("CARGO_BIN_EXE_bip"));
            command.current_dir(directory).arg("verify");
            if expected {
                command.args(["--expected-id", &oid]);
            }
            let output = success(command.output().unwrap());
            assert!(output.contains(&oid) && output.contains(LIMIT));
        }
    }
}

#[test]
fn later_owner_changes_do_not_change_bootstrap_but_duplicate_markers_fail() {
    let fixture = Fixture::new();
    let (_, oid) = initialize(&fixture);
    fs::write(
        fixture.repo.join(".bip/allowed_signers"),
        "later owner data",
    )
    .unwrap();
    fixture.git(&["add", ".bip/allowed_signers"]);
    fixture.git(&["commit", "-m", "Later owner change"]);
    let clone = fixture.directory.path().join("clone");
    fixture.clone_to(&clone);
    let output = success(
        fixture
            .command(env!("CARGO_BIN_EXE_bip"))
            .current_dir(&clone)
            .args(["verify", "--expected-id", &oid])
            .output()
            .unwrap(),
    );
    assert!(output.contains(LIMIT));
    fixture.git(&["commit", "--allow-empty", "-m", "bip! init v1"]);
    failure(
        fixture.bip(&["verify", "--expected-id", &oid]),
        "Ambiguous initialization",
    );
    let duplicate_clone = fixture.directory.path().join("duplicate-clone");
    fixture.clone_to(&duplicate_clone);
    failure(
        fixture
            .command(env!("CARGO_BIN_EXE_bip"))
            .current_dir(duplicate_clone)
            .args(["verify", "--expected-id", &oid])
            .output()
            .unwrap(),
        "Ambiguous initialization",
    );
}

#[test]
fn wrong_signing_key_and_local_trust_overrides() {
    let fixture = Fixture::new();
    let (_, oid) = initialize(&fixture);
    fixture.git(&["config", "gpg.format", "openpgp"]);
    fixture.git(&["config", "gpg.ssh.program", "/nonexistent/custom-verifier"]);
    fixture.git(&[
        "config",
        "gpg.ssh.allowedSignersFile",
        "/nonexistent/signers",
    ]);
    fixture.git(&["config", "gpg.ssh.revocationFile", "/nonexistent/revoked"]);
    assert!(success(fixture.bip(&["verify"])).contains(LIMIT));
    let other = fixture.key("other");
    let tree = fixture.git(&["rev-parse", "HEAD^{tree}"]);
    let parent = fixture.git(&["rev-parse", "HEAD^"]);
    let bad = fixture.git(&[
        "-c",
        "gpg.format=ssh",
        "-c",
        "gpg.ssh.program=ssh-keygen",
        "-c",
        &format!("user.signingkey={}", other.display()),
        "commit-tree",
        "-S",
        &tree,
        "-p",
        &parent,
        "-m",
        "bip! init v1",
    ]);
    fixture.git(&["update-ref", "refs/heads/main", &bad]);
    failure(fixture.bip(&["verify"]), "did not verify");
    fixture.git(&["replace", &bad, &oid]);
    failure(fixture.bip(&["verify"]), "did not verify");
    fs::write(fixture.repo.join(".git/info/grafts"), "").unwrap();
    failure(fixture.bip(&["verify"]), "graft overrides");
}

#[test]
fn signing_and_reinitialization_failures_preserve_checkout() {
    let fixture = Fixture::new();
    fixture.commit();
    let key = fixture.key("owner");
    let before = fixture.snapshot();
    failure(
        fixture.bip(&["init", "--signing-key", "missing"]),
        "Cannot access selected",
    );
    let saved_key = key.with_extension("saved");
    fs::rename(&key, &saved_key).unwrap();
    failure(
        fixture.bip(&[
            "init",
            "--signing-key",
            key.with_extension("pub").to_str().unwrap(),
        ]),
        "Could not create SSH-signed",
    );
    fs::rename(saved_key, &key).unwrap();
    assert_eq!(fixture.snapshot(), before);
    assert!(!fixture.repo.join(".bip").exists());
    success(fixture.bip(&["init", "--signing-key", key.to_str().unwrap()]));
    let before = fixture.snapshot();
    failure(
        fixture.bip(&["init", "--signing-key", key.to_str().unwrap()]),
        "will not be overwritten",
    );
    assert_eq!(fixture.snapshot(), before);
    fixture.git(&["rm", "-r", ".bip"]);
    fixture.git(&["commit", "-m", "Remove metadata"]);
    failure(
        fixture.bip(&["init", "--signing-key", key.to_str().unwrap()]),
        "repeated initialization",
    );
}

#[test]
fn sha256_repository_identity() {
    let fixture = Fixture::new();
    // Reinitialize an empty repository with the alternate Git object format.
    fs::remove_dir_all(fixture.repo.join(".git")).unwrap();
    fixture.git(&["init", "--object-format=sha256", "-b", "main"]);
    fixture.git(&["config", "user.name", "Test Owner"]);
    fixture.git(&["config", "user.email", "owner@example.test"]);
    let (_, oid) = initialize(&fixture);
    assert_eq!(oid.len(), 64);
    assert!(success(fixture.bip(&["verify", "--expected-id", &oid])).contains(LIMIT));
    failure(
        fixture.bip(&["verify", "--expected-id", &oid[..40]]),
        "full hexadecimal",
    );
}
