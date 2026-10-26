# repository-bootstrap Specification

## Purpose

Establish and independently verify a repository's signed initial ownership declaration, using its initialization commit as a permanent identity carried by ordinary Git history.

## Requirements

### Requirement: Supported repository context

Bootstrap commands SHALL operate on local, non-bare Git repositories with complete history and a local `refs/heads/main`. They SHALL reject unsupported contexts with a nonzero exit status and an actionable diagnostic, without migrating branches, fetching history, or modifying the repository.

#### Scenario: No local main branch
- **WHEN** a repository has no local `main`, even if it has a remote-tracking `origin/main`
- **THEN** the command fails and instructs the user to create or rename the intended branch to `main`

#### Scenario: Bare or shallow repository
- **WHEN** either bootstrap command runs in a bare or shallow repository
- **THEN** it fails, identifies the unsupported context, and explains the need for a working-tree checkout or complete history respectively

#### Scenario: Outside a repository
- **WHEN** either bootstrap command runs outside a Git repository
- **THEN** it fails and explains that an existing Git repository is required

### Requirement: Initialization preconditions

`bip init` SHALL require `main` to be checked out with at least one existing commit, a clean index and working tree including non-ignored untracked files, no existing `.bip` path, and no initialization marker on the first-parent history of `main`. Failed preconditions SHALL leave repository state unchanged.

#### Scenario: Main is not checked out
- **WHEN** initialization runs on another branch or detached HEAD
- **THEN** it fails and instructs the user to check out `main`

#### Scenario: No existing commit
- **WHEN** initialization runs on an unborn `main`
- **THEN** it fails and explains that a first Git commit must be created on `main` to anchor bip initialization

#### Scenario: Dirty checkout
- **WHEN** initialization encounters staged changes, tracked working-tree changes, or non-ignored untracked files
- **THEN** it fails and asks the user to commit, stash, or otherwise resolve those changes

#### Scenario: Existing metadata or initialization
- **WHEN** `.bip` already exists, including as an ignored path, or a first-parent initialization marker already exists
- **THEN** initialization fails without overwriting metadata or creating another initialization commit

### Requirement: Explicit initial owner

`bip init --signing-key <path>` SHALL require explicit selection of an SSH signing key, record exactly one corresponding public key as the first owner, and demonstrate possession through the initialization commit's SSH signature. It SHALL NOT silently select a key from Git configuration or generate a new key.

#### Scenario: Key selection omitted
- **WHEN** initialization is requested without `--signing-key`
- **THEN** it fails with usage guidance and leaves the repository unchanged

#### Scenario: Public key with an agent
- **WHEN** the selected path contains a public key and its private counterpart is available through the SSH agent
- **THEN** initialization can use that key to sign and record the matching public key

#### Scenario: Key cannot sign
- **WHEN** the selected key is invalid, inaccessible, or cannot produce the required signature
- **THEN** initialization fails with an explanatory diagnostic and leaves `main`, the index, and working-tree files unchanged

### Requirement: Portable bootstrap metadata

Initialization SHALL add regular files `.bip/identity.toml` and `.bip/allowed_signers`. The identity file SHALL record format version 1, a UTC initialization timestamp, and the full parent commit hash. The OpenSSH allowed-signers file SHALL contain exactly one owner entry using principal `bip-owner` and namespace `git`. The format SHALL be documented for independent implementations.

#### Scenario: Metadata records the bootstrap context
- **WHEN** initialization succeeds from commit P
- **THEN** `identity.toml` contains `format_version = 1`, `initialized_at` as an RFC 3339 UTC string, and `parent_commit` as P's full object ID
- **AND** `allowed_signers` contains the explicitly selected owner's public key

### Requirement: Signed initialization event and identity

Initialization SHALL create a single-parent SSH-signed commit directly on `main`, changing only the two bootstrap files, with the exact subject `bip! init v1`. The full object ID of that commit SHALL be the repository identity and SHALL be printed on success. Initialization SHALL leave the index and working tree clean.

#### Scenario: Successful initialization
- **WHEN** all initialization preconditions hold and signing succeeds
- **THEN** `main` advances by one commit whose parent is the previous tip and whose signature verifies against the recorded initial owner
- **AND** the command exits successfully and prints the full initialization commit hash

#### Scenario: Clone preserves identity
- **WHEN** a repository is cloned through Git with its initialization history intact
- **THEN** bootstrap verification discovers the same full repository identity without a forge or access to the owner's private key

### Requirement: Initialization discovery

Verification SHALL inspect the entire first-parent history of local `main` for commit subjects beginning with the reserved prefix `bip! init`. Exactly one candidate SHALL exist and its subject SHALL equal `bip! init v1`. Missing, multiple, malformed, or unsupported-version candidates SHALL fail verification, even when an expected identity is supplied.

#### Scenario: Marker only on a merged side branch
- **WHEN** an initialization marker occurs only outside the first-parent history of `main`
- **THEN** it is excluded from initialization discovery

#### Scenario: No initialization found
- **WHEN** no first-parent candidate exists
- **THEN** verification fails and reports that no bip initialization was found

#### Scenario: Multiple candidates
- **WHEN** two or more first-parent candidates exist
- **THEN** verification reports ambiguous initialization and fails, including when one candidate matches `--expected-id`

#### Scenario: Unsupported or malformed marker
- **WHEN** the sole candidate has subject `bip! init v2` or another subject beginning with `bip! init` that is not exactly `bip! init v1`
- **THEN** verification fails with an unsupported-version or malformed-marker diagnostic as appropriate

### Requirement: Bootstrap verification

`bip verify` SHALL validate the candidate's recorded metadata, single-parent structure, bootstrap-only changes, and SSH signature using the initial owner entry from that commit. The recorded parent SHALL match its actual parent, which SHALL contain no `.bip` path. Invalid or unsupported bootstrap data SHALL cause a nonzero exit status.

#### Scenario: Valid first ownership declaration
- **WHEN** the sole candidate satisfies the bootstrap format and its signature verifies against its declared owner
- **THEN** bootstrap verification succeeds and prints its full identity

#### Scenario: Marker without a valid bootstrap
- **WHEN** a candidate is unsigned, signed by another key, has invalid metadata, has a mismatching parent field, or changes files beyond the two bootstrap files
- **THEN** verification fails and explains the failed check

#### Scenario: Unsupported metadata representation
- **WHEN** a bootstrap file is missing, is a symlink, uses an unsupported format version, or the owner file does not declare exactly one permitted initial key
- **THEN** verification fails without treating the declaration as valid ownership

### Requirement: Expected identity comparison

`bip verify --expected-id <commit-hash>` SHALL accept a full hexadecimal object ID in the repository's object format and require equality with the discovered initialization commit. Abbreviated IDs and revision expressions SHALL be rejected. Without an expected ID, success SHALL explicitly report that no expected identity was checked.

#### Scenario: Expected identity matches
- **WHEN** a valid bootstrap's full identity matches the supplied expected ID
- **THEN** verification succeeds and reports the identity match

#### Scenario: Expected identity differs or is invalid
- **WHEN** the supplied expected ID differs from the discovered identity or is not a full object ID
- **THEN** verification fails with an identity mismatch or invalid-input diagnostic respectively

#### Scenario: No external identity supplied
- **WHEN** verification succeeds without `--expected-id`
- **THEN** output distinguishes a valid bootstrap from a checked external identity and makes no claim of canonical-repository selection

### Requirement: Historical verification boundary

Verification SHALL use committed bootstrap objects rather than current working-tree files, the current owner list, or user-configured allowed signers. It SHALL require neither a clean checkout nor `main` to be checked out. Every successful verification SHALL state that subsequent ownership changes and governance history were not checked.

#### Scenario: Later owner data differs
- **WHEN** `main` has later commits changing ownership metadata or the working tree contains different owner data
- **THEN** verification checks the initialization against its original owner declaration and reports only bootstrap validity
- **AND** it states that subsequent ownership changes and governance history were not checked

#### Scenario: Verification from another branch
- **WHEN** another branch is checked out with local edits while local `main` contains a valid bootstrap
- **THEN** verification reads local `main` and leaves refs, the index, working-tree files, and repository configuration unchanged

### Requirement: Safe initialization failures

Initialization SHALL prepare and verify the signed bootstrap before publishing it to `main`. Detected failures SHALL not overwrite pre-existing files or unrelated user changes. If `main` changes during preparation, initialization SHALL refuse to overwrite that change. Failed publication SHALL restore changes made by the command where safely possible and report any incomplete cleanup explicitly.

#### Scenario: Signing fails before publication
- **WHEN** the signer exits unsuccessfully or the prepared signature does not verify
- **THEN** no initialization commit becomes reachable from `main` and the index and working tree remain unchanged

#### Scenario: Main moves during preparation
- **WHEN** `main` no longer points to the parent captured by initialization
- **THEN** initialization fails without replacing the new tip

#### Scenario: Publication cleanup cannot complete
- **WHEN** an ordinary publication error occurs and restoring command-created files or index changes fails
- **THEN** the command exits unsuccessfully and identifies the remaining state and recovery steps without claiming successful initialization
