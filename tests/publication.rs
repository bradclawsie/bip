#![cfg(unix)]

mod support;

use std::fs;

use support::{Fixture, failure, success};

#[test]
fn existing_index_lock_is_preserved() {
    let fixture = Fixture::new();
    fixture.commit();
    let key = fixture.key("owner");
    let before = fixture.snapshot();
    fs::write(fixture.repo.join(".git/index.lock"), "other operation").unwrap();
    failure(
        fixture.bip(&["init", "--signing-key", key.to_str().unwrap()]),
        "Cannot lock Git index",
    );
    assert_eq!(fixture.snapshot(), before);
    assert_eq!(
        fs::read_to_string(fixture.repo.join(".git/index.lock")).unwrap(),
        "other operation"
    );
    assert!(!fixture.repo.join(".bip").exists());
}

#[test]
fn failed_ref_update_restores_index_and_files() {
    let fixture = Fixture::new();
    fixture.commit();
    let key = fixture.key("owner");
    let before = fixture.snapshot();
    let tools = fixture.shim("git", "for arg in \"$@\"; do\n if [ \"$arg\" = update-ref ]; then echo 'simulated ref failure' >&2; exit 1; fi\ndone\nexec \"$BIP_TEST_GIT\" \"$@\"");
    failure(
        fixture
            .with_tools(&tools)
            .args(["init", "--signing-key"])
            .arg(&key)
            .output()
            .unwrap(),
        "Could not publish initialization",
    );
    assert_eq!(fixture.snapshot(), before);
    assert!(!fixture.repo.join(".bip").exists());
    assert!(!fixture.repo.join(".git/index.lock").exists());
}

#[test]
fn concurrent_main_update_is_never_overwritten() {
    let fixture = Fixture::new();
    fixture.commit();
    let key = fixture.key("owner");
    let before = fixture.snapshot();
    let tree = fixture.git(&["rev-parse", "HEAD^{tree}"]);
    let tools = fixture.shim("git", "for arg in \"$@\"; do\n if [ \"$arg\" = update-ref ]; then\n  other=$(\"$BIP_TEST_GIT\" commit-tree \"$BIP_TEST_TREE\" -p \"$BIP_TEST_OLD\" -m 'Concurrent update') || exit 1\n  \"$BIP_TEST_GIT\" update-ref refs/heads/main \"$other\" \"$BIP_TEST_OLD\" || exit 1\n fi\ndone\nexec \"$BIP_TEST_GIT\" \"$@\"");
    failure(
        fixture
            .with_tools(&tools)
            .env("BIP_TEST_TREE", tree)
            .env("BIP_TEST_OLD", &before.0)
            .args(["init", "--signing-key"])
            .arg(&key)
            .output()
            .unwrap(),
        "Could not publish initialization",
    );
    let after = fixture.snapshot();
    assert_ne!(after.0, before.0);
    assert_eq!(after.1, before.1);
    assert_eq!(after.2, before.2);
    assert_eq!(
        fixture.git(&["log", "-1", "--format=%s"]),
        "Concurrent update"
    );
    assert!(!fixture.repo.join(".bip").exists());
}

#[test]
fn path_created_during_preparation_is_preserved() {
    let fixture = Fixture::new();
    fixture.commit();
    let key = fixture.key("owner");
    let before = fixture.snapshot();
    let tools = fixture.shim("git", "for arg in \"$@\"; do\n if [ \"$arg\" = index ]; then printf 'concurrent file' > .bip; fi\ndone\nexec \"$BIP_TEST_GIT\" \"$@\"");
    failure(
        fixture
            .with_tools(&tools)
            .args(["init", "--signing-key"])
            .arg(&key)
            .output()
            .unwrap(),
        "will not be overwritten",
    );
    let after = fixture.snapshot();
    assert_eq!(after.0, before.0);
    assert_eq!(after.1, before.1);
    assert_eq!(
        fs::read_to_string(fixture.repo.join(".bip")).unwrap(),
        "concurrent file"
    );
    assert!(!fixture.repo.join(".git/index.lock").exists());
}

#[test]
fn incomplete_file_cleanup_is_explicit_and_preserves_changes() {
    let fixture = Fixture::new();
    fixture.commit();
    let key = fixture.key("owner");
    let before = fixture.snapshot();
    let tools = fixture.shim("git", "for arg in \"$@\"; do\n if [ \"$arg\" = update-ref ]; then\n  printf 'user edit' > .bip/identity.toml\n  printf 'user file' > .bip/keep\n  echo 'simulated ref failure' >&2; exit 1\n fi\ndone\nexec \"$BIP_TEST_GIT\" \"$@\"");
    failure(
        fixture
            .with_tools(&tools)
            .args(["init", "--signing-key"])
            .arg(&key)
            .output()
            .unwrap(),
        "Incomplete cleanup",
    );
    assert_eq!(fixture.git(&["rev-parse", "main"]), before.0);
    assert_eq!(fs::read(fixture.repo.join(".git/index")).unwrap(), before.1);
    assert_eq!(
        fs::read_to_string(fixture.repo.join(".bip/identity.toml")).unwrap(),
        "user edit"
    );
    assert_eq!(
        fs::read_to_string(fixture.repo.join(".bip/keep")).unwrap(),
        "user file"
    );
    assert!(!fixture.repo.join(".bip/allowed_signers").exists());
    assert!(!fixture.repo.join(".git/index.lock").exists());
}

#[test]
fn concurrent_index_change_is_preserved_on_cleanup_failure() {
    let fixture = Fixture::new();
    fixture.commit();
    let key = fixture.key("owner");
    let tools = fixture.shim("git", "for arg in \"$@\"; do\n if [ \"$arg\" = update-ref ]; then printf 'user index' > .git/index; exit 1; fi\ndone\nexec \"$BIP_TEST_GIT\" \"$@\"");
    failure(
        fixture
            .with_tools(&tools)
            .args(["init", "--signing-key"])
            .arg(&key)
            .output()
            .unwrap(),
        "Incomplete cleanup",
    );
    assert_eq!(
        fs::read_to_string(fixture.repo.join(".git/index")).unwrap(),
        "user index"
    );
    assert!(!fixture.repo.join(".git/index.lock").exists());
}

#[test]
fn initialization_does_not_invoke_repository_hooks() {
    use std::os::unix::fs::PermissionsExt;
    let fixture = Fixture::new();
    fixture.commit();
    let key = fixture.key("owner");
    for name in ["pre-commit", "commit-msg", "reference-transaction"] {
        let hook = fixture.repo.join(".git/hooks").join(name);
        fs::write(&hook, "#!/bin/sh\nexit 1\n").unwrap();
        fs::set_permissions(hook, fs::Permissions::from_mode(0o755)).unwrap();
    }
    success(fixture.bip(&["init", "--signing-key", key.to_str().unwrap()]));
    assert!(fixture.git(&["status", "--porcelain"]).is_empty());
}

#[test]
fn uncreatable_metadata_directory_leaves_repository_unchanged() {
    use std::os::unix::fs::PermissionsExt;
    let fixture = Fixture::new();
    fixture.commit();
    let key = fixture.key("owner");
    let before = fixture.snapshot();
    let permissions = fs::metadata(&fixture.repo).unwrap().permissions();
    fs::set_permissions(&fixture.repo, fs::Permissions::from_mode(0o555)).unwrap();
    let result = fixture.bip(&["init", "--signing-key", key.to_str().unwrap()]);
    fs::set_permissions(&fixture.repo, permissions).unwrap();
    failure(result, "Permission denied");
    assert_eq!(fixture.snapshot(), before);
    assert!(!fixture.repo.join(".bip").exists());
    assert!(!fixture.repo.join(".git/index.lock").exists());
}

#[test]
fn incompatible_clean_filter_does_not_publish_dirty_initialization() {
    let fixture = Fixture::new();
    fixture.commit();
    let key = fixture.key("owner");
    fs::write(
        fixture.repo.join(".gitattributes"),
        ".bip/** filter=replace\n",
    )
    .unwrap();
    fixture.git(&["add", ".gitattributes"]);
    fixture.git(&["commit", "-m", "Attribute policy"]);
    fixture.git(&["config", "filter.replace.clean", "printf 'replacement\\n'"]);
    let before = fixture.snapshot();
    failure(
        fixture.bip(&["init", "--signing-key", key.to_str().unwrap()]),
        "does not match the prepared bootstrap",
    );
    assert_eq!(fixture.snapshot(), before);
    assert!(!fixture.repo.join(".bip").exists());
}
