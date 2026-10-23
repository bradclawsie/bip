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

At time of repository creation, the creator must designate themselves as an owner:

```bash
> bip git init -- alice@example.org
```

`bip` will do the following here:

1. Execute the `git init ...` to actually create the repository.
2. Set mandatory ssh signing on commits and tags.
3. Create an authority declaration for the creator with the provided email,
which the creator must sign with their ssh private key.
4. Add a sentinel directory `.bip` to indicate that this is 
a `bip`-enabled repository.
5. Add the initial owner's authority declaration to the directory 
`/.bip/authority`.
6. Add a file `/.bip/repository-id.pkl` containing a unique id for
this repository.
7. Commit these files to `main` as signed by the creator.

## The authority declaration

An authority declaration is used by `bip` to indicate a particular user
can make a merge to `main` (and other owner operations).

The authority declaration is a file in a reserved directory `/.bip/authority`.
The file name will be derived from the user's ssh key, using a process 
like this command line example:

```bash
awk '{print $2}' ~/.ssh/id_ed25519.pub | base64 -d | sha256sum | cut -d' ' -f1
```

This produces a string that is a safe file name on any operating system, and is
tied directly to the ssh key. The user will be able to discover their authority
declaration file name with this shortcut:

```bash
> bip authority-declaration-file
```

The file itself will be written as a `pkl` file:

```pkl
ctime = 1792773366
email = "alice@example.org"
key = "ssh-ed25519..."
repository-id = "$current-repository-id"
```

The user will be required to sign this file with their ssh credentials in addition
to signing the commit that adds it.

`bip` will provide a special command

```bash
> bip create-authority-declaration
```

that will guide the creation of this file for a user. During the repository
creation process described above, this command will be called on behalf
of the repository creator.

## Using the authority declaration

When `bip` is going to perform an operation that can only be completed
by a repository owner (such as merging to `main`), the user's ssh public
key will be obtained, and the authority declaration filename determined
must be present in the `main` branch.

The signature of the authority declaration file will be confirmed with
the user's ssh public key, and the repository-id in the authority declaration
must match what the `main` branch has in the file `/.bip/repository-id.pkl`.

An implication of this is that certain operations (such as a merge to `main`)
can only be performed where an owner's ssh keys are available. This means 
owner operations must happen where the owner is logged in. This is a design
decision of `bip` intended to keep responsible owners in the loop and accountable.

## Add a new owner

Let's say that in addition to Alice, there is a desire to add another
user, Bob, as an owner. As the current sole owner, Alice must complete
this operation.

This is just another merge to `main` as described above, but let's
spell it out.

Bob will make a branch called `make-bob-an-owner`.

In that repository, Bob will run: 

```bash
> bip create-authority-declaration
```

which creates his authority declaration file along which he will sign
with his ssh credentials.

The commit of this file is also signed like all commits.

Alice can then fetch this branch and also obtain Bob's ssh public
key to verify the file. Alice must then decide to merge this to
`main` using her own judgement, following the process for merging
to `main` described above.

Now Bob has his authority file on the `main` branch, and he is 
therefore also a repository owner now.

## Forking

Forking in `bip` establishes a new identification and ownership
for a copy of a repository.

The command

```bash
> bip fork -- bob@example.org
```

Will replace the repository id located at `/.bip/repository-id.pkl` with
a new value and will replace all existing authority declarations with a
new single authority declaration for `bob@example.org`. The changes will
be written to `main` as a commit, signed by the new owner.

## Archiving

Archiving a repository in `bip` turns the repository into a read-only
resource.

The command

```bash
> bip archive
```

Removes all authority declarations and commits this change to `main`. Since
there are no authority declarations to make merges to `main`, the repository
