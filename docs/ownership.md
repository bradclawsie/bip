# Ownership

**Status:** Exploratory

This document explores the ownership model for bip.

It contains established design decisions, proposed
mechanisms, and unresolved questions. It is not a
behavioral specification.

Accepted requirements will be maintained through
OpenSpec.

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

Since branch `main` is special with respect to ownership
assertions, `bip` will only allow merging other branches
to it, and individual edited commits will be flagged
by `bip review`.

## Building the chain of trust

When a `git` repository is initialized with `bip init`,
the user performing the initialization will be considered
the first owner and have their `ssh` public key
entered into the new `.bip/allowed_signers` file.

A repository ID file will be created at the same time
with the current date and parent commit hash listed.
This will allow others to see at what point the repository
was configured for `bip`.

## Proposed invariants

Following from the discussion above, these are considered
invariant:

- Branch `main` is special and contains the authoritative
ownership record.
- Branch `main` can only be updated with `bip merge`.
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

