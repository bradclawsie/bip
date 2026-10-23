# Why should bip exist?

`bip` is intended to be an exploration of source control
that is fun, approachable, hackable, and a real benefit
to users. The fundamental premise of `bip` is that you 
should not need a forge to complete your experience. Forges
have become centralized chokepoints that represent
a foundational challenge to the idea of decentralized version
control.

## Git

`git` is very good at handling large amounts of code. It
is universal and ubiquitous, and has fantastic tool support.
No one really wants to migrate off of `git`. `bip` will use
`git` and provide support for enforcing conventions that
will enable every git repository to provide local, decentralized
support for features that people currently get from forges. If
for some reason you don't like using `bip`, you can stop using
it and return to your previous `git` workflow.

## What is missing

- Issues
- Pull requests
- Ownership model
- Improved repository visualization
- tbd...

## What `bip` actually provides

Any of these enhanced workflow features can be embdedded
in a `git` repository if a team is sufficiently disciplined.
Everything needed to encode support for the missing features can be
provided through reserved path names and reserved file formats.
`bip` just makes it easy to work with a particular pre-determined
set of opinionated formats for repository metadata.

As the real value is in the metadata file formats and not the 
`bip` tool, it should be possible to build websites, agent 
integrations and other tools that interact with issues and pull
requests in a portable manner.
