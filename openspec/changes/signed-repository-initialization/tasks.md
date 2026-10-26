# Tasks

## 1. Repository and command boundaries

- [x] 1.1 Add command parsing for `init --signing-key <path>` and `verify [--expected-id <commit-hash>]`, retaining build-hash `--help`; verify missing arguments, unknown commands, and exit/output behavior with CLI tests.
- [x] 1.2 Add Git repository discovery and captured local `main` resolution, including subdirectory invocation and actual object-format handling; verify ordinary, missing-main, unborn-main, detached, outside-repository, bare, and shallow cases in temporary repositories with isolated Git configuration.
- [x] 1.3 Add initialization precondition checks for checked-out `main`, clean index/worktree including non-ignored untracked files, and absent `.bip`; verify failures preserve refs/index/files, including an ignored or symlink `.bip` path.
- [x] 1.4 Document repository prerequisites and command usage in README, including the first-commit explanation and required Git/OpenSSH tools; check examples against the CLI tests and diagnostics.

## 2. Bootstrap format and discovery

- [x] 2.1 Implement version-1 identity metadata serialization/validation and the single-owner OpenSSH allowed-signers representation; verify field types, UTC timestamps, full parent hashes, unknown fields, duplicate owners, invalid keys, forbidden options, and unsupported versions with focused format tests.
- [x] 2.2 Implement first-parent initialization discovery using the reserved subject prefix, with exact version-1 subject validation; verify no marker, one marker, malformed/unsupported markers, multiple markers, and markers confined to merged side history, without expected-ID ambiguity bypasses.
- [x] 2.3 Implement shared committed-object bootstrap validation for parent structure, absent prior `.bip`, regular non-executable metadata files, parent-field agreement, and bootstrap-only tree changes; verify malformed tree and commit fixtures are rejected.
- [x] 2.4 Document metadata fields, signer-line grammar, message marker, first-parent discovery, and commit-hash identity in a repository format document; verify each documented rule has a matching validator or scenario and no self-referential ID field.

## 3. SSH signing and bootstrap verification

- [x] 3.1 Integrate explicit private-key/public-key-with-agent selection and Git/OpenSSH signing through argument-based subprocess calls; verify disposable-key signing, agent-backed signing, unusable keys, and missing signing tools without consulting developer keys or silently using configured alternatives.
- [x] 3.2 Implement signature verification using the candidate's owner entry in temporary verifier input and explicit SSH verification configuration, disabling replacement history and rejecting graft overrides; verify unsigned/wrong-key signatures fail and unrelated local trust settings or working-tree signer changes cannot affect the result.
- [x] 3.3 Connect read-only `bip verify` to captured-main discovery, shared validation, signature checking, full expected-ID validation/comparison, and bounded success output; verify matching/mismatching/missing/abbreviated expected IDs and preservation of refs/index/worktree/configuration from a dirty non-main checkout.
- [x] 3.4 Document bootstrap validity versus expected identity matching and social canonicality, with the mandatory later-governance limitation; verify command-output tests include the limitation on every successful verification path.

## 4. Signed initialization publication

- [x] 4.1 Prepare the two-file tree with a temporary index, create the single-parent signed initialization commit, and validate it before publication; verify its exact subject, parent, file delta, selected owner, and identity, and verify signing failures leave refs/index/worktree unchanged.
- [x] 4.2 Add pre-publication rechecks, locked index updates, guarded `main` advancement, and cleanup limited to command-owned changes; verify success leaves a clean checkout and publication-error fixtures cover an existing index lock, moved `main`, and an uncreatable metadata path without losing unrelated state.
- [x] 4.3 Complete repeated-initialization refusal, missing-tool diagnostics, and explicit incomplete-cleanup reporting; verify repeated init, existing metadata, a historical marker after metadata removal, and a simulated cleanup failure do not claim success or overwrite existing content.
- [x] 4.4 Update ownership documentation for the protocol initialization exception, permanent commit identity, bootstrap self-declaration, and deferred owner transitions; document crash-recovery limits and inspect the described recovery steps for preservation of user data.

## 5. End-to-end acceptance

- [x] 5.1 Exercise initialization followed by ordinary local clone and verification with and without the expected ID, removing private-key/agent access for verification; verify both copies report the identical full initialization hash and explicit governance limits.
- [x] 5.2 Exercise later owner-file changes and later duplicate markers across clone boundaries; verify original-owner bootstrap checking remains independent of current owner files while ambiguous discovery always fails.
- [x] 5.3 Run repository-required Rust formatting, checks, applicable tests, and Clippy; validate this OpenSpec change strictly and confirm the documented init/verify examples agree with the completed CLI behavior, reporting any unavailable tool coverage.
