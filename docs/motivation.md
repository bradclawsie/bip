# Why should bip exist?

`bip` is intended to be an exploration of source control
that is fun, approachable, hackable, and a real benefit
to users. The fundamental premise of `bip` is that you 
should not need a forge to complete your experience. Forges
have become centralized chokepoints that not only represent
a foundational challenge to the idea of decentralized version
control, but they are now also a real impediment to progress
as they experience heavy load and are routinely unavailable.

## What about git?

`git` is a very successful version control system. It is
used everywhere and it will never die. Entire economies
rely on it. But it is a victim of its own success; it can't
really change. `git` has become inextricably intertwined
with a small list of well-known forges that have locked
users in to a centralized model.

## Some things that would be nice

None of these have been figured out, they are just daydreams right now.

The repository itself should contain:

- The best ideas of `git`.
- Issues.
- PRs.
- Mandatory signing using ssh keys.
- A list of ssh public keys that are allowed to merge to the master branch.
- A way to solve the problem of notifying others of changes (poll or broadcast).
- Interfaces and tools for exploring the repository in-place.

In general, a `bip` repository should give the developer everything needed to
work in-place without a forge. 

## Non-goals

`bip` should not:

- Be `git` compatible.
- Try to be object storage for large/binary files that shouldn't be in version control.
- Allow rewriting history.
- Aim to outperform `git`.
- Aim to out-feature `git`.
