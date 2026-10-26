# Design

## Context

See `proposal.md` for motivation and `specs/repository-bootstrap/spec.md` for the behavior contract. The current project is a Rust 2024 binary with no dependencies or tests. `src/bip.rs` prints the build commit for `--help`; `build.rs` obtains that hash. There is no existing repository discovery, metadata parser, signing integration, or command framework.

The vision commits to Git history, SSH signatures, and portable metadata. The ownership document is exploratory: its date/parent ID file and merge-only language are refined here by the accepted initialization-commit identity and explicit bootstrap exception. Choosing a socially canonical repository remains outside cryptographic verification.

## Goals / Non-Goals

**Goals:**
- Make a bootstrap independently reproducible and verifiable from committed objects, without local trust configuration or private-key access during verification.
- Separate candidate discovery, bootstrap validation, expected identity comparison, and the result's governance limits.
- Keep ordinary errors from publishing a partial initialization or consuming unrelated user changes.

**Non-Goals:**
- Define authorization for later owner transitions or certify the current tip's governance.
- Introduce custom cryptography, private-key storage, Git hooks, network access, or a global identity registry.
- Guarantee crash-atomic updates across Git refs, the index, and working-tree files, or coordinate arbitrary simultaneous Git writers.

## Decisions

### 1. CLI surface and proposed defaults

Use `bip init --signing-key <path>` and `bip verify [--expected-id <commit-hash>]`. These names and flag spellings are proposed concrete defaults for review. Both commands discover the enclosing repository, including invocation from a subdirectory. Errors go to stderr with nonzero status; successful results go to stdout with status zero. Do not add `review` or `show` commands in this change. Retain the existing build-hash `--help` behavior.

`init` requires the preconditions in the spec; `verify` reads local `refs/heads/main` regardless of the checked-out branch or dirty files. Neither command creates `main`, renames branches, fetches, or changes Git configuration. A remote-tracking branch is not a substitute for local `main`. Reject bare and shallow repositories explicitly.

Alternatives: automatic branch migration would conceal a governance decision; implementing full `review` now would imply historical checks we have not designed.

### 2. Versioned, signed metadata

Create exactly two regular, non-executable files in the initialization tree:

- `.bip/identity.toml`: UTF-8 TOML with `format_version` (integer 1), `initialized_at` (RFC 3339 UTC string), and `parent_commit` (full hexadecimal Git object ID). Require these fields, reject unknown fields for version 1, and validate their types. The date records the initializer's declared time, not independently trusted time.
- `.bip/allowed_signers`: an OpenSSH allowed-signers file with one effective entry, principal `bip-owner`, option `namespaces="git"`, and the selected ordinary SSH public key. Permit blank lines and comments when reading, but require exactly that principal and namespace and exactly one key; reject certificates, wildcard principals, and additional authorization options in bootstrap version 1. Key comments have no identity meaning.

The signature binds the key, metadata, parent, tree, and commit message together. The parent tree must have no `.bip` path; the bootstrap must add only these two files and leave all existing paths unchanged. Validate data as Git objects, not by checking out repository-controlled paths. Respect the repository's object format rather than assuming 40-character SHA-1 IDs.

These file names, field encodings, and fixed principal are proposed format details. A fixed principal supplies the OpenSSH verification label; it does not claim a person's name, email, or forge account. TOML provides a readable format with an explicit evolution boundary. Arbitrary owner-list options and certificates can be considered in a later format.

### 3. Commit identity and candidate discovery

The exact commit subject is `bip! init v1`. The initializer emits that subject without an additional body. Verification reserves any subject beginning with `bip! init` as an initialization candidate, then requires the sole candidate to have the exact supported subject. A body is not used for metadata or discovery.

Traverse the entire first-parent history of a captured local `main` tip, without path filtering. Markers on merged side histories do not count. Zero candidates means uninitialized; multiple candidates mean ambiguous; one unsupported or malformed candidate fails with a specific diagnostic. Never skip a bad candidate to find a convenient valid one, and never let an expected ID suppress ambiguity.

The full hash of the valid initialization commit is the repository identity. Do not attempt to embed that hash inside its own metadata. `parent_commit` records the preceding Git commit instead. Normal subsequent commits preserve identity; rewriting the bootstrap changes it. An expected ID must be a full object ID, not a Git revision expression, prefix, URL, or branch name; normalize hexadecimal case before comparison.

Alternative: a random stored identifier would add a second identity mechanism without solving initial trust selection. A marker alone is only a locator, never proof of ownership.

### 4. Use Git and OpenSSH for signatures

Invoke installed tools through Rust process APIs with separate arguments, never through shell interpolation. Use Git's SSH commit-signing and verification support, backed by `ssh-keygen`. Report missing tools and unsupported signing capabilities clearly; do not fall back to unsigned initialization or another signature format.

An explicitly selected path can name a private SSH key or a public key whose counterpart is available through an agent. Obtain the corresponding public key using OpenSSH where necessary; do not parse or persist private-key material. Pass explicit per-invocation signing configuration so a user's GPG defaults or configured signing key cannot change the selected owner. Keep existing configuration unchanged.

For verification, materialize the candidate's validated allowed-signers bytes into a private temporary file and point Git's SSH verifier at that file through per-invocation configuration. Explicitly use the standard OpenSSH verifier and exclude user revocation/allowed-signers overrides from this bootstrap decision. A later revocation does not retroactively establish or disprove bootstrap possession in this scope. Require a successful SSH verification under the one recorded principal and key, not merely a recognizable signature header or generic trusted-key result. Disable replacement-object history and reject legacy graft overrides so the checked graph is the actual commit graph.

Alternative: implementing SSH signature parsing and verification in Rust would create unnecessary cryptographic and interoperability work.

### 5. Prepare before publishing

Use Git plumbing with a temporary index initialized from the captured parent tree. Write bootstrap blobs and a tree without staging user files or invoking commit hooks. Create the SSH-signed commit with the captured parent, and run the same bootstrap validation used by `verify` on that prepared object before changing the working tree or branch.

Recheck the branch, clean state, and absence of `.bip` before publication. Materialize only the prepared bootstrap files and update the real index using Git, then advance `refs/heads/main` with an expected-old-value ref update. Manage the index through Git's locking conventions; fail rather than bypass an existing lock. A changed branch tip must not be overwritten. Successful initialization leaves the index and working tree matching the new commit and prints its full identity.

If ordinary publication fails, restore only command-owned file and index changes, preserving concurrent user changes, and report any cleanup that cannot be performed safely. Never use a broad hard reset or delete pre-existing `.bip` contents. Failed signing occurs before publication and therefore leaves the checkout untouched; unreachable prepared Git objects are harmless and can be garbage collected. Temporary resources are cleaned up on normal exit.

There is no transaction spanning all Git and filesystem state. A process crash may leave metadata or an index change; subsequent `init` refuses to overwrite it and diagnostics/documentation explain how to inspect and recover. The supported workflow assumes no simultaneous editing or Git operations; expected-old-value updates protect `main` even if that assumption is violated.

### 6. Narrow verification results

Capture `main` once and inspect immutable objects from that tip. Validate the single-parent event, metadata schema, parent field, absence of prior `.bip`, bootstrap-only tree changes, and the initial owner's SSH signature before printing success. Current owner files and working-tree edits have no role in this check. Verification creates only temporary files outside tracked repository state and never updates refs, the index, working-tree files, or configuration.

On success, print the full repository identity, whether an expected identity was matched or not supplied, and: `Initialization verified; subsequent ownership changes and governance history were not checked.` Identity mismatch, ambiguity, and malformed or invalid bootstrap data fail without a success claim. Even successful expected-ID matching does not select the socially canonical repository or validate subsequent changes.

### 7. Implementation boundaries and verification

Keep argument dispatch in `src/bip.rs` and introduce small modules for Git process interaction, metadata, initialization, and verification where they clarify responsibilities. Share the bootstrap validator between initialization and verification. Follow repository Rust conventions; choose dependencies during implementation only for concrete argument parsing, TOML, timestamp, or temporary-file needs.

Exercise the contract through integration tests with isolated Git configuration, temporary repositories, and disposable OpenSSH keys. Cover a normal clone and verification without private-key access, a public key backed by a test agent, signature/metadata rejection, first-parent discovery, expected identities, precondition failures, and signing/publication failure preservation. Keep tests offline and independent of the developer's keys, agent, hooks, and Git settings.

## Risks / Trade-offs

- Self-declared bootstrap ownership can be mistaken for social authority -> distinguish signature validity, expected identity matching, and canonical-project selection in output and documentation.
- Later ownership tampering can coexist with a valid bootstrap -> print the historical verification limitation on every success; defer transition validation explicitly.
- First-parent history is a protocol convention -> document that a side-branch bootstrap is not discovered and rewriting the bootstrap changes identity.
- Whole-history discovery costs grow with repository history -> accept a read-only linear traversal initially; caching is unnecessary for the first capability.
- Git/OpenSSH versions and agent behavior vary -> test actual signing/verification paths and report capability failures rather than relying only on version numbers.
- Ref, index, and file publication cannot be universally atomic -> prepare and verify first, use guarded publication, test rollback, and document crash recovery limits.

## Migration Plan

This adds the first metadata format; there is no existing bip state to migrate. Users prepare a clean, committed `main` themselves. Initialization adds one signed commit and reports its identity for sharing through an appropriate social channel. Existing `.bip` state is refused rather than migrated or overwritten. Update `docs/ownership.md` during implementation to describe this bootstrap exception and identity scheme, and add user-facing format and command documentation. No automatic history rewrite, branch migration, or rollback command is introduced.
