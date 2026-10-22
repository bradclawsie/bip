# Ownership

What does it mean to "own" a repository? `git` outsources 
ownership authority to external mechanisms such as domain
name or trademark ownership, social reputation or control
of a distribution mechanism, among other mechanisms.

Maybe `bip` can do better.

We know that ssh signing will be mandatory and central to
how `bip` operates. 

When a repository is created, the creator will have
their ssh public key and email written to a file. That
file will be encrypted with the user's public ssh key.
The file's name will be the fingerprint of the user's 
ssh key. This is called an authority file. The reserved
branch `main` must have at least one authority file or
the repository cannot accept commits.

If the creator should wish to commit their ssh-signed changes 
to the reserved branch name `main`, the fingerprint of their
ssh public key will be calculated and a there must be a 
corresponding authority file in `main`. The creator must then decrypt
that authority file and they will be asked to visually assert
that the contents of the authority file are correct. The
decryption process assures us that authority files can't just
be dropped in by anyone. There is no protection from someone
changing `bip` itself to alter or ignore this process - users
will be encouraged to make sure their version of `bip` is
safe and authoritative (using sha256sum etc).

Should the creator wish to add a new contributor who also has
the authority to merge to `main`, the contributor simply needs
to create a new branch with their own authority file added to
it (in branches that are not `main`, this is permitted). A 
pull request can be made for their branch, and the creator can
merge it with the process already described. Note that the creator
can't read the contents of the authority file; if the contributor
passed along a corrupt authority file, they will not be able to
decrypt it and won't ever be able to merge to `main`.

The new contributor now has the ability to merge to `main`,
including merges that remove the authority files of others.
Be careful who you invite as a collaborator! We assume people
are generally decent.

An implication of this is that merges to `main` must be done only
where an owner's ssh private key is available. This means merging
to `main` will be an activity that is intended to be done locally
by a real user, not as a website process. There will be no way to
change the main repository from `main`; it is a reserved name.

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
