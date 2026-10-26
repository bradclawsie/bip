# Proposal

## Why

bip needs a small, independently verifiable foundation for repository governance before adding collaboration features. A signed initialization event lets an owner establish a portable repository identity that another person can inspect and verify through an ordinary Git clone.

## What Changes

- Add `bip init --signing-key <path>` to explicitly select the first owner's SSH key and create a signed initialization commit directly on `main`.
- Require an existing local `main`, checked out with at least one commit, a clean working tree and index, complete history, and no existing bip initialization or `.bip` metadata.
- Record versioned bootstrap metadata and the first owner's SSH allowed-signers entry under `.bip/`.
- Identify the repository by the full hash of its signed initialization commit. Discover that commit through a versioned commit-message marker on the first-parent history of local `main`.
- Add read-only `bip verify [--expected-id <commit-hash>]` to verify the bootstrap metadata and SSH signature, optionally compare an expected identity, and explicitly leave subsequent governance unchecked.
- Document the format, supported workflow, failure behavior, and the social boundary of choosing which repository identity to trust.

## Capabilities

### New Capabilities

- `repository-bootstrap`: Establish, discover, and verify the signed initial ownership declaration and permanent repository identity.

### Modified Capabilities

None. The project has no existing capability specifications.

## Impact

- Extend the current CLI in `src/bip.rs` and introduce focused Rust modules for Git integration, bootstrap metadata, initialization, and verification as needed.
- Use installed Git and OpenSSH signing tools; no forge, network service, or separate key store is required. Add narrowly scoped Rust dependencies for argument handling or metadata only where useful.
- Add integration coverage using temporary repositories and disposable SSH keys, plus user and format documentation. Clarify the exploratory ownership document's bootstrap exception to merge-only contributor changes.
- Preserve the current build-hash `--help` behavior and build script.
- Exclude owner transitions, merges, full historical governance review, issues, pull requests, fork policy, bare repositories, shallow repositories, and canonical-repository selection.
