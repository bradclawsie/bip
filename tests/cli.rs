use std::process::Command;

#[test]
fn help_prints_only_build_hash() {
    let output = Command::new(env!("CARGO_BIN_EXE_bip"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{}\n", env!("BIP_COMMIT_HASH"))
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn invalid_commands_explain_usage() {
    for args in [
        vec!["init"],
        vec!["init", "--signing-key"],
        vec!["verify", "--expected-id"],
        vec!["unknown"],
        vec!["verify", "extra"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_bip"))
            .args(args)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .contains("Usage: bip init --signing-key")
        );
    }
}
