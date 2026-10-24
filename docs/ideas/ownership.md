# Ownership

What does it mean to "own" a repository? `git` outsources 
ownership authority to external mechanisms such as domain
name or trademark ownership, social reputation or control
of a distribution mechanism, among other mechanisms.

Maybe `bip` can do better.

## What is meant by "ownership"

`git` doesn't treat one user differently than another,
and beyond naming a default branch, `git` doesn't treat
branches differently either.

Over time, tools and forges have assigned special meaning
to the default branch. This has proven useful.

`bip` shall designate a single branch - `main` as being
special and ownership will be defined as control of this
branch. `bip` shall only allow merges to `main`, not individual
commits.

## Ownership workflows

After repository creation, the creator must designate themselves 
as an owner:

```bash
> bip init
```

`bip` will do the following here:

1. Verify that the current working directory is a `git` repository.
2. Configure for ssh signing on commits and tags.
3. Add a sentinel directory `.bip` to indicate that this is 
a `bip`-enabled repository.
4. Add the initial owner's public key in `/.bip/allowed_signers`
in the format accepted for this file.
5. Configure the repository to use `/.bip/allowed_signers` for
asserting authority.
6. Add a file `/.bip/repository-id.pkl` containing a unique id for
this repository.
7. Commit these files to `main` as signed by the creator.

The signature in (7) establishes the first link in the chain
of trust, and `/.bip/allowed_signers` is considered trusted
initially by default.

## The authority declaration

Authority is granted from the file `/.bip/allowed_signers` which
is already an established format utilized by `git`.

New owners can be added to the file through merge requests.

## Using the authority declaration

Repository ownership is a pattern supported by `bip`. Certain
operations will be limited to contributors listed in the
`/.bip/allowed_signers` file. Chief among these operations is
the ability to merge a branch to `main`. Let's explore that.

When this is run by an owner:

```bash
> git branch
* main
> bip merge $some-branch
```

A merge commit will be made and signed using the owner's ssh key.
`bip` will then assert that the corresponding public key is in
`/.bip/allowed_signers` for the first parent of the merge commit
(`main`, right before the merge commit). If it isn't, the merge
commit is undone.

An implication of this signing process is that certain operations
like merging to `main` must be done in a local environment where
the owner's ssh key is decrypted.

## Add a new owner

Let's say that in addition to an initial owner Alice, there is a 
desire to add another user, Bob, as an owner. 

As the current sole owner, Alice must complete this operation.

This is just another merge to `main` as described above, but let's
spell it out.

Bob will make a branch called `make-bob-an-owner`.

In that branch, he will add the appropriate line in
`/.bip/allowed_signers`. He makes a normal signed `git` commit.

Alice fetches his branch and runs

```bash
> bip merge make-bob-an-owner
```

which makes a signed `git` commit, but then also checks that Alice's
ssh public key is available in `/.bip/allowed_signers`. It is, so the
commit is retained.

## Forking

Forking in `bip` establishes a new identification and ownership
for a copy of a repository.

The command

```bash
> bip fork -- bob@example.org
```

Will replace the repository id located at `/.bip/repository-id.pkl` with
a new value and will replace `/.bip/allowed_signers` with a new file
with a single ssh public key - that belongin to the forking owner.

## Archiving

Archiving a repository in `bip` turns the repository into a read-only
resource.

The command

```bash
> bip archive
```

Removes all entries in `/.bip/allowed_signers` and commits this 
change to `main`, which will prevent `bip` from allowing further
merges to `main`.

## Enforcing our rules

`bip` is about establishing patterns and formats for a better
`git` experience. But there is no way to make `bip` mandatory.
But we can check to make sure the recommended patterns are
being followed.

A command

```bash
> bip review
```

Will scan commits to `main` in a similar fashion as something
like

```bash
> git log --main --show-signature --first-parent
```

and assert that each committer was in `/.bip/allowed_signers`
in `main` when each respective merge commit was performed.
