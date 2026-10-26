# Repository bootstrap format, version 1

A bip repository's identity is the full Git object ID of its signed
initialization commit. No metadata file contains that commit's own hash.
Cloning preserves identity; rewriting the initialization commit changes it.
Identity does not change when later commits change owners.

## Locating initialization

Walk the complete first-parent history of local `refs/heads/main`. A candidate
has a commit-message first line beginning with the reserved prefix `bip! init`.
Exactly one candidate must exist, and its first line must be exactly:

```text
bip! init v1
```

The initializer writes that line with no body. A verifier ignores any body.
No candidates means uninitialized; multiple candidates mean ambiguous history.
A sole unsupported version or malformed marker fails verification. An expected
identity never bypasses these checks. Markers only on merged side histories do
not count. Git replacement objects are ignored; legacy graft overrides are
rejected.

## Committed metadata

The initialization commit has exactly one parent, whose tree has no `.bip`
path. It adds exactly two regular, non-executable files (Git mode `100644`),
leaving all existing paths unchanged:

`.bip/identity.toml` is UTF-8 TOML:

```toml
format_version = 1
initialized_at = "2026-10-26T18:00:00Z"
parent_commit = "0123456789abcdef0123456789abcdef01234567"
```

All fields are mandatory; unknown fields are rejected in version 1. The version
is an integer. The date is an RFC 3339 string with UTC offset, recording the
initializer's declared time. It is not trusted evidence of wall-clock time.
The parent is a string containing the full hexadecimal object ID of the actual
parent commit. SHA-1 repositories use 40 hexadecimal digits; SHA-256 repositories
use 64. Hexadecimal comparison is case insensitive.

`.bip/allowed_signers` uses OpenSSH allowed-signers syntax:

```text
bip-owner namespaces="git" <key-type> <base64-public-key> [comment]
```

Exactly one effective entry is permitted. Blank lines and lines whose first
non-whitespace character is `#` are ignored. Fields are separated by whitespace.
The principal must be exactly `bip-owner` and the only option must be exactly
`namespaces="git"`. Certificates, wildcard principals, additional options, and
additional owner entries are unsupported. OpenSSH validates the ordinary public
key. A trailing key comment has no identity meaning. The principal is a fixed
protocol label, not a person's account name or email address.

## Signature and trust boundary

The initialization commit carries a Git SSH signature under namespace `git`,
verified against its own committed initial-owner entry. Its marker alone grants
no authority. The signature binds the commit's tree, parent, and message, proving
possession of the declared owner's key for this bootstrap event.

Verification uses the bootstrap's committed files, not current owner files,
working-tree edits, or user-configured allowed signers. It does not require a
private key or agent. It checks the metadata schema, parent, tree changes, and
signature. A supplied expected identity must be a full hexadecimal object ID in
the repository's format; abbreviated IDs and revision expressions are rejected.

A valid bootstrap alone does not establish social canonicality. People choose
which identity to trust through their own channels. This version does not verify
subsequent ownership changes or governance history, and successful output always
states that limit. Future ownership changes will require authorization from
previously established owners; self-declaration is only the bootstrap rule.
