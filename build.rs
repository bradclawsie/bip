use std::process::Command;

fn git(args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .output()
        .expect("Git must be installed to determine the build commit");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("Git output must be UTF-8")
}

fn main() {
    println!(
        "cargo:rustc-env=BIP_COMMIT_HASH={}",
        git(&["rev-parse", "HEAD"]).trim()
    );

    // Resolve these paths through Git to support worktrees as well as checkouts.
    let paths = git(&[
        "rev-parse",
        "--git-path",
        "HEAD",
        "--git-path",
        "refs",
        "--git-path",
        "packed-refs",
    ]);
    for path in paths.lines() {
        println!("cargo:rerun-if-changed={path}");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
