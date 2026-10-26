mod support;

use std::fs;
use support::{Fixture, failure};

fn metadata(fixture: &Fixture) {
    let key = fixture.key("owner");
    fs::create_dir(fixture.repo.join(".bip")).unwrap();
    let parent = fixture.git(&["rev-parse", "HEAD"]);
    fs::write(fixture.repo.join(".bip/identity.toml"), format!("format_version = 1\ninitialized_at = \"2026-10-26T12:00:00Z\"\nparent_commit = \"{parent}\"\n")).unwrap();
    fs::write(
        fixture.repo.join(".bip/allowed_signers"),
        format!(
            "bip-owner namespaces=\"git\" {}",
            fs::read_to_string(key.with_extension("pub")).unwrap()
        ),
    )
    .unwrap();
}

#[test]
fn discovery_uses_only_first_parent_and_rejects_ambiguity() {
    let fixture = Fixture::new();
    fixture.commit();
    failure(fixture.bip(&["verify"]), "No bip initialization");
    fixture.git(&["checkout", "-b", "side"]);
    fixture.git(&["commit", "--allow-empty", "-m", "bip! init v1"]);
    fixture.git(&["checkout", "main"]);
    fixture.git(&["merge", "--no-ff", "side", "-m", "Merge side"]);
    failure(fixture.bip(&["verify"]), "No bip initialization");
    fixture.git(&["commit", "--allow-empty", "-m", "bip! init v2"]);
    let first = fixture.git(&["rev-parse", "HEAD"]);
    failure(
        fixture.bip(&["verify"]),
        "Unsupported initialization marker",
    );
    fixture.git(&["commit", "--allow-empty", "-m", "bip! init v1"]);
    failure(
        fixture.bip(&["verify", "--expected-id", &first]),
        "Ambiguous initialization",
    );
    failure(
        fixture.bip(&["init", "--signing-key", "unused"]),
        "repeated initialization",
    );
}

#[test]
fn malformed_marker_is_rejected() {
    let fixture = Fixture::new();
    fixture.commit();
    fixture.git(&["commit", "--allow-empty", "-m", "bip! initialize"]);
    failure(fixture.bip(&["verify"]), "Malformed initialization marker");
}

#[test]
fn malformed_bootstrap_trees_are_rejected() {
    for case in [
        "valid",
        "order",
        "submodule",
        "extra",
        "parent",
        "version",
        "missing",
        "prior",
        "symlink",
        "executable",
        "merge",
        "root",
    ] {
        let fixture = Fixture::new();
        fixture.commit();
        if case == "submodule" {
            let oid = fixture.git(&["rev-parse", "HEAD"]);
            fixture.git(&[
                "update-index",
                "--add",
                "--cacheinfo",
                &format!("160000,{oid},sub"),
            ]);
            fixture.git(&["commit", "-m", "Existing submodule"]);
            fixture.git(&["config", "diff.ignoreSubmodules", "all"]);
        }
        if case == "prior" {
            fs::create_dir(fixture.repo.join(".bip")).unwrap();
            fs::write(fixture.repo.join(".bip/prior"), "existing").unwrap();
            fixture.git(&["add", ".bip"]);
            fixture.git(&["commit", "-m", "Prior metadata"]);
            fs::remove_dir_all(fixture.repo.join(".bip")).unwrap();
        }
        metadata(&fixture);
        let identity = fixture.repo.join(".bip/identity.toml");
        match case {
            "order" => {
                let order = fixture.directory.path().join("order");
                fs::write(&order, ".bip/identity.toml\n.bip/allowed_signers\n").unwrap();
                fixture.git(&["config", "diff.orderFile", order.to_str().unwrap()]);
            }
            "submodule" => {
                let oid = fixture.git(&["rev-parse", "HEAD"]);
                fixture.git(&["update-index", "--cacheinfo", &format!("160000,{oid},sub")]);
            }
            "extra" => fs::write(fixture.repo.join("extra"), "unrelated").unwrap(),
            "parent" => {
                let text = fs::read_to_string(&identity).unwrap();
                fs::write(
                    &identity,
                    text.replace(&fixture.git(&["rev-parse", "HEAD"]), &"0".repeat(40)),
                )
                .unwrap();
            }
            "version" => {
                let text = fs::read_to_string(&identity).unwrap();
                fs::write(
                    &identity,
                    text.replace("format_version = 1", "format_version = 99"),
                )
                .unwrap();
            }
            "missing" => fs::remove_file(&identity).unwrap(),
            #[cfg(unix)]
            "symlink" => {
                fs::remove_file(&identity).unwrap();
                std::os::unix::fs::symlink("allowed_signers", &identity).unwrap();
            }
            _ => (),
        }
        fixture.git(&["add", "-A"]);
        if case == "executable" {
            fixture.git(&["update-index", "--chmod=+x", ".bip/identity.toml"]);
        }
        if case == "root" || case == "merge" {
            let tree = fixture.git(&["write-tree"]);
            let parent = fixture.git(&["rev-parse", "HEAD"]);
            let commit = if case == "root" {
                fixture.git(&["commit-tree", &tree, "-m", "bip! init v1"])
            } else {
                let other = fixture.git(&["commit-tree", &tree, "-m", "other"]);
                fixture.git(&[
                    "commit-tree",
                    &tree,
                    "-p",
                    &parent,
                    "-p",
                    &other,
                    "-m",
                    "bip! init v1",
                ])
            };
            fixture.git(&["update-ref", "refs/heads/main", &commit]);
        } else {
            fixture.git(&["commit", "-m", "bip! init v1"]);
        }
        let expected = match case {
            "valid" | "order" => "requires an SSH signature",
            "extra" | "missing" | "submodule" => "only the two bootstrap files",
            "parent" => "Recorded parent",
            "version" => "Unsupported bootstrap metadata version",
            "prior" => "parent already contains",
            "symlink" | "executable" => "regular non-executable",
            "merge" | "root" => "exactly one parent",
            _ => unreachable!(),
        };
        failure(fixture.bip(&["verify"]), expected);
    }
}
