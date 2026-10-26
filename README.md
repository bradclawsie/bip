# bip

Run `cargo run -- --help` to print the full Git commit hash embedded at build
time. Building requires Git and a checkout of the repository. The hash identifies
HEAD; it does not indicate uncommitted changes.

## Repository bootstrap

Install Git with SSH commit-signing support and OpenSSH (`ssh-keygen`). Build
with `cargo build`; the executable is `target/debug/bip`.

Start with an existing, non-bare Git repository with complete history. Create or
rename your intended branch to local `main`, check it out, and create at least
one Git commit. That commit anchors bip initialization. The index and working
tree must be clean, including non-ignored untracked files, and `.bip` must not
already exist.

Explicitly select your SSH signing key:

```sh
bip init --signing-key /path/to/owner-key
```

The key path can name a private key or a public key whose private counterpart is
available through your SSH agent. bip creates a signed initialization commit
directly on `main` and prints its full hash, the permanent repository identity.
It does not create keys, rename branches, fetch history, or change Git settings.

From this repository or an ordinary clone, check the bootstrap:

```sh
bip verify
bip verify --expected-id <full-initialization-commit-hash>
```

Verification reads committed history from local `main`; it can run from another
branch or a dirty checkout. Bare and shallow repositories are unsupported. Both
commands can run from a repository subdirectory. Errors are printed to stderr
with a nonzero exit status.

Successful verification reports the initialization identity and whether it
matched the expected identity you supplied. Without `--expected-id`, it verifies
the bootstrap's internal consistency and signature but does not check an
external identity. Deciding which identity represents the canonical project is
a social decision, made through channels you trust.

Every successful verification states: "Initialization verified; subsequent
ownership changes and governance history were not checked." It does not certify
current ownership or later operations. See [the bootstrap format](docs/bootstrap-format.md)
for the portable metadata and verification rules.

## Initialization failures and recovery

Signing and bootstrap validation finish before bip changes the checkout.
An ordinary publication failure restores the files and index changes made by
the command where safe. Existing metadata is never overwritten; repeated
initialization is refused even if a later commit removed `.bip`.

Avoid editing files or running other Git operations during initialization.
bip guards the update to `main`, but refs, the index, and working-tree files
cannot be updated in one crash-atomic transaction. A crash or incomplete cleanup
can leave bootstrap files or staged changes behind. An incomplete-cleanup error
identifies the remaining state rather than deleting changes it cannot attribute
to the command.

To recover, first stop other Git operations and preserve any files and index
state you need. Inspect `git status`, `git diff --cached`, and `git log -1 main`.
If `main` already contains the initialization commit, run `bip verify` and
inspect checkout differences before making further changes. If initialization
was not published, save any `.bip` contents outside the checkout, unstage only
the bootstrap paths if needed (`git restore --staged -- .bip`), and move the
leftover `.bip` directory outside the checkout. Inspect a reported `index.lock`
and remove it only after confirming no Git process owns it. Resolve other
changes until the checkout is clean before retrying. Do not use a hard reset
or discard unrelated edits as part of recovery.

