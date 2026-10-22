# Ownership

What does it mean to "own" a repository? `git` outsources 
ownership authority to external mechanisms such as domain
name or trademark ownership, social reputation or control
of a distribution mechanism, among other mechanisms.

Maybe `bip` can do better.

We know that ssh signing will be mandatory and central to
how `bip` operates. 

When a `bip` repository is created, there will be one
branch named `main`. This is a special, reserved name
and `bip` will only allow merge commits, no "edited" commits.
At the time of repository creation, the person creating
the repository will be asked to use their ssh private key
to sign a file containing the date, their email, and their
ssh public key. The file name will be the fingerprint of
their public key. The `bip` process itself is granted permission
to make a single commit of this file to `main`, and this commit
is signed.

```bash
> bip init alice@example.org
> Enter private key password to self-sign initial authority:...
```

This creates a new repository with Alice's authority declaration:

```bash
> ls .authority
>
> $alice-ssh-key-fingerprint $alice-ssh-key-fingerprint.sig
>
> cat .authority/$alice-ssh-key-fingerprint
{"time":1792685176, "email": "alice@example.org", "key":"..."}
```

Now somewhere, Bob clones this `bip` repository and creates
a branch `add-bob-to-owners`, and in that branch creates his
own authority declaration:

```bash
> bip new-branch add-bob-to-owners
> bip new-authority bob@example.org # won't work on main!
> ls .authority
>
> $alice-ssh-key-fingerprint $alice-ssh-key-fingerprint.sig
> $bob-ssh-key-fingerprint $bob-ssh-key-fingerprint.sig
>
> cat .authority/$bob-ssh-key-fingerprint
{"time":1792685178, "email": "bob@example.org", "key":"..."}
> bip commit "add bob"
> Enter private key password to sign commit:...
```

Bob's commit is signed with his private key. The branch `add-bob-to-owners`
becomes available to Alice. She can see his authority declaration and verify
the signature with his public key - it is contained in the authority declaration.

Alice can then sign the merge into `main`:

```bash
> bip branch
> main
> bip merge add-bob-to-owners
> Enter private key password to sign merge commit:...
```

Alice's private key is now available to verify that the file 
`.authority/$alice-ssh-key-fingerprint.sig` is her signature for
her authority declaration `.authority/$alice-ssh-key-fingerprint` 
and that the public key in that declaration matches hers. Since
there is a match, the merge completes. Otherwise, the merge is
declined.

An implication of this is that merges to `main` must be done only
where an owner's ssh private key is available. This means merging
to `main` will be an activity that is intended to be done locally
by a real user, not as a website process.

Should some random person wish to fork the repository, a special
`fork` command will recreate the initial authority file, thus 
breaking the connection of the owners of the forked repository
and establishing a new owner. This is useful as it prevents changes
in a fork from being attributed to people who are unaware.

Also, there will be a command that removes all authority files. This
is the `archive` command. As there are no authority files on the
`main` branch following this command, no more commits can be made 
(although the repository can be forked and commits can start anew).

There may be sneaky ways to get around all of these measures. We
won't guarantee the `bip` will be airtight initially. We also
can't completely get around the leverage of social reputation,
domain/trademark ownership and distribution ownership, but
`bip` should provide some improvement with the measures described.
