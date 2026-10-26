# Ownership

**Status:** Exploratory

This document explores the ownership model for bip.

It contains established design decisions, proposed
mechanisms, and unresolved questions. It is not a
behavioral specification.

Accepted requirements will be maintained through
OpenSpec.

The first implemented capability establishes and verifies the signed initial
ownership declaration. See [the bootstrap format](bootstrap-format.md) for its
exact rules. Owner transitions, `bip merge`, and full historical `bip review`
remain future capabilities; bootstrap verification does not certify those
operations or current ownership.

## Definition

_Ownership_ is a `bip` concept that relates to
elevated privileges associated with some repository
contributors. On forge sites, the same concept is
encoded in organization membership or individual
privileges associated with a contributor's account.

## How ownership is defined

A repository configured for `bip` will have a
`.bip` directory that contains a `allowed_signers`
file. The format of this file is the same as
that supported by `ssh`. Contributors who have
ownership privileges will have their public `ssh`
key in this file. A contributor performing an
operation that requires ownership privileges will
prove their status through an `ssh` signing task
that must be verified by checking their corresponding
public key in `.bip/allowed_signers`.

Ownership status is always asserted from the `main`
branch - this branch name is reserved in `bip`
as a mandatory source of truth for ownership assertions.
These assertions must honor the history of the commit
log. Owners will come and go. Review of owner operations
must therefore take in to consideration the contents
of `.bip/allowed_signers` at the time of a particular
authorized event.

The intended contributor workflow updates `main` by merging other branches.
This is a voluntary governance convention, not a restriction on Git itself.
bip protocol operations can create direct commits when required: the signed
initialization commit is the first explicit exception. Future `bip review`
must distinguish permitted protocol operations from unauthorized direct edits.

## Building the chain of trust

`bip init --signing-key <path>` requires an existing, checked-out local `main`
with at least one commit and a clean index and working tree. The repository
must have complete history and a working-tree checkout; `.bip` must not already
exist. Users create or rename `main` and make the first Git commit themselves.

The initializer explicitly selects an SSH key. bip records its public key as
the first owner in `.bip/allowed_signers`, together with versioned initialization
date and parent information in `.bip/identity.toml`. It creates and verifies a
signed, single-parent commit directly on `main`, with subject `bip! init v1`.
The full hash of that signed commit is the permanent repository identity.
It cannot be embedded inside that same commit; the metadata instead records
the preceding commit's hash. Later owner changes preserve repository identity;
rewriting the initialization commit changes it.

The initialization event is discovered only on the first-parent history of
local `main`. Multiple initialization markers are ambiguous and rejected.
`bip verify` validates this event against its own committed initial-owner key,
not current files or later owner declarations. This self-declaration establishes
the bootstrap; later owner additions will need authorization from previously
established owners.

`bip verify --expected-id <full-commit-hash>` additionally checks an identity
obtained through a channel the user trusts. Without that argument, verification
reports a valid bootstrap without checking an external identity. In either
case it explicitly states that subsequent ownership changes and governance
history were not checked. Choosing the canonical project remains social.

## Proposed invariants

Following from the discussion above, these are considered
invariant:

- Branch `main` is special and contains the authoritative
ownership record.
- Ordinary contributor updates to `main` use `bip merge`; defined bip protocol
operations, including initialization, can create direct commits.
- Authorized operations are validated against the `.bip/allowed_signers`
as present on branch `main` at the time of the operation.
- `bip review` must assert the consistency of authorized operations
over time, relative to the state of `.bip/allowed_signers`.

## Example: adding a new owner

Assume a repository was created, and initialized
for `bip` by Alice. The `main` branch shall now
have a `.bip/allowed_signers` file with Alice's `ssh`
public key.

Bob wishes to become an owner of the repository also,
and convinces Alice to allow him to gain the privilege.

The changes necessary to complete this task require
Bob's `ssh` public key to be listed in `.bip/allowed_signers`
on branch `main`. Merging a change to `main` is limited
to owners when contributors consent to using `bip merge`.
Since Alice is the only owner at present, she must complete
this task.

Bob will create a branch of the repository using `git`
and edit `.bip/allowed_signers` to include his own `ssh`
public key, and signs the commit using his `ssh` key.

Alice will fetch Bob's branch and verify the signature
on the commit using the public `ssh` key Bob committed.
If she is satisfied, she can complete the task by
using a `bip merge` command to complete the merge (instead
of a `git` merge). This `bip merge` command will 
make sure Alice's `ssh` public key is in 
`.bip/allowed_signers` as committed on `main`, and
then make the signed commit. Now Bob is also an owner.

## Limits of ownership

Use of `bip` is voluntary and consensual for 
contributors. Anyone can clone a `git` repo,
mutate it, and present it to the world as the
authoritative repository. `bip`-defined ownership must
therefore be accompanied by ongoing efforts to
use distribution and social reputation to guarantee
that a particular repository is considered canonical.

`bip review` will allow contributors to determine if 
participants are honoring the ownership model.

