# Vision

## Purpose

`bip` will be a command line tool that people can use
alongside `git` in order to enjoy features that have
recently required using a centralized forge.

The central thesis of `bip` is that many collaboration 
and governance features traditionally provided by 
centralized forges can instead be supported through 
decentralized `git` workflows, if users are willing
to embrace certain practices.

Among the features intended are issues, pull requests 
and repository ownership. Instead of relying on a 
centralized forge, `bip` will allow these features to 
be embedded in the repository itself, working alongside 
other tools.

## Goals

- Support forge-independent collaboration using ordinary
`git` repositories.
- Define portable, documented metadata formats.
- Establish verifiable repository governance without
centralized services.
- Preserve compatibility with existing `git` workflows.
- Enable independent implementations of the same
conventions.

## Non-goals

`bip` works with `git` and does not seek to replace it.
`bip` does not seek to encrypt or obfuscate anything in
the repository. `bip` does not seek to limit the use
of repository users who do not use it. 

`bip` does not require a centralized service or a privileged 
repository instance to determine authoritative state. 
Authority is derived from verifiable repository metadata
and its associated rules.

## Design principles

- Forge independence. The entire point of `bip` is that
the information forges use to create features can live
in `git` itself.
- Portability. The data formats and practices used by
`bip` should be documented and open to third-parties.
- Interoperability. `bip` will rely on `ssh` for signing
and verification. `bip` will rely on `git` for logging,
branch management and decentralization.
- Pragmatism. `bip` will favor achievable, incremental
improvements over expanding its scope to solve problems
already addressed effectively by existing tools.

## Envisioned capabilities

- Ownership. Allow the use of `ssh` keys to express
that certain users are considered to have elevated
roles. Certain operations in `bip` will only be available
to owners. Verify that the ownership model is being
honored. Associated with ownership will be a unique
value identifying the repository and its association
with a particular set of owners.
- Issue tracking. Author, edit, and change the status
of issues. The format of the issues shall be in a 
documented textual format.
- Pull requests. Pull requests shall combine branching,
issue tracking, and the ownership model to recreate the
experience people currently utilize on forge sites.
- Forking. A fork will combine branching and ownership.
- Archiving. Archiving will prevent further authorized
changes to a repository.

## Open questions

- Are there more fundamental issues with `git` integration
that undermine `bip`'s proposed features and value?
- Is it possible to provide a portable experience through
terminal interaction for features such as issues and pull-requests?
- Will out-of-band communication mechanisms need to be integrated
to communicate metadata updates to concerned parties?

## Status of this document

This document describes the current vision and architectural 
direction of `bip`. It is intended to guide design and 
implementation, not to serve as a complete behavioral
specification.

Specific mechanisms described here may be refined or replaced
as the project develops. Detailed requirements and accepted
behavior will be captured through OpenSpec.

Where this document leaves behavior unspecified, implementations
should not assume that a design decision has already been made.
