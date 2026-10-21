---
name: rust-style
description: Apply bip's Rust conventions when writing, modifying, or reviewing Rust code and APIs.
---

# Rust style

Use this skill for Rust implementation and review in bip.

Follow repository instructions and configured tooling. Use the official Rust
Style Guide as the formatting baseline and the Rust API Guidelines as API
design guidance.

Read the relevant topics in [references/google-rust.md](references/google-rust.md)
for selected Google conventions and documented differences. That reference
identifies bip defaults separately from Google-specific preferences.

Favor readable, direct application code. Apply reusable-library guidance when
the API actually serves reusable-library needs.

During review, distinguish correctness issues, repository conventions, and
optional suggestions. Explain concrete benefits rather than enforcing
unadopted Google preferences.

For changed Rust code, run the applicable formatter, checks, and tests using
the repository's toolchain. Report checks that could not run. Do not introduce
toolchain, lint, or formatter configuration solely to enforce this skill.
