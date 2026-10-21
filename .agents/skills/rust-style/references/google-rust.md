# Google Rust conventions for bip

Selected guidance from the [Google Rust Style Guide](https://google.github.io/styleguide/rustguide.html),
reviewed 2026-10-21. This is a curated adaptation, not adoption of the whole guide.

The bip defaults below are repository conventions. Google-only preferences are
informational. Official API guidelines are recommendations, not language requirements.

## Formatting and imports

Use rustfmt. Reserve formatting skips for meaningful alignment or generated
code. Prefer explicit imports; narrow glob imports can be useful in tests
and enum-heavy code.

Sources: [formatting](https://google.github.io/styleguide/rustguide.html#formatting-other),
[skips](https://google.github.io/styleguide/rustguide.html#formatting-optout),
[glob imports](https://google.github.io/styleguide/rustguide.html#glob-imports).

**bip default:** Follow configured rustfmt, otherwise its defaults.
Google additionally prefers imports grouped by source module, locally scoped
trait imports, and module-qualified free functions. These are advisory here,
not universal Rust requirements. The official guide permits nested import
groups.

Comparison: [Google imports](https://google.github.io/styleguide/rustguide.html#use-statement),
[local traits](https://google.github.io/styleguide/rustguide.html#prefer-local-trait-imports),
[free functions](https://google.github.io/styleguide/rustguide.html#toplevel-functions),
[official imports](https://doc.rust-lang.org/style-guide/items.html#use-statements),
[official formatting baseline](https://doc.rust-lang.org/style-guide/).

## Ownership and function boundaries

Prefer borrowing to unnecessary copying; shared ownership and cloning remain
reasonable when they simplify the design. Accept borrowed data rather than
its container. Choose iterators or collections according to callers' needs.

Sources: [ownership](https://google.github.io/styleguide/rustguide.html#ownership-borrowing),
[borrowed inputs](https://google.github.io/styleguide/rustguide.html#flexible-references),
[returned iterators](https://google.github.io/styleguide/rustguide.html#return-iterators),
[input iterators](https://google.github.io/styleguide/rustguide.html#accept-iterators).

**bip default:** Prefer `&str`, `&[T]`, and `&T` when container-specific
operations are unnecessary. Do not require generic parameters for ordinary
application helpers.

## Types, traits, and naming

Use newtypes for meaningful distinctions, named fields for otherwise obscure
tuples, standard traits for standard behavior, and derives when suitable.
Name conversions consistently with ownership.

Sources: [newtypes](https://google.github.io/styleguide/rustguide.html#newtypes-vs-type-aliases),
[named fields](https://google.github.io/styleguide/rustguide.html#prefer-named-fields),
[standard traits](https://google.github.io/styleguide/rustguide.html#known-traits),
[derives](https://google.github.io/styleguide/rustguide.html#derive),
[conversion names](https://google.github.io/styleguide/rustguide.html#naming-methods).

**Distinctions:**

- Google prescribes `try_new()` for constructors returning `Result`.
  Standard Rust also uses fallible `new()`, such as `CString::new`.
  bip does not require a universal constructor prefix.
- Google favors adding application trait implementations as needed.
  The Rust API Guidelines encourage reusable types to implement applicable
  common traits eagerly. Apply each recommendation in its stated context.

Comparison: [Google constructors](https://google.github.io/styleguide/rustguide.html#naming-initialization-functions),
[`CString::new`](https://doc.rust-lang.org/std/ffi/struct.CString.html#method.new),
[Google trait scope](https://google.github.io/styleguide/rustguide.html#traits-yagni),
[official common traits](https://rust-lang.github.io/api-guidelines/interoperability.html#types-eagerly-implement-common-traits-c-common-traits).

## Errors and readable implementation

Choose `Option`, `Result`, or panic according to the contract. Prefer matching
to checking then unwrapping; explain invariant panics with `expect`. Use loops
when iterator chains obscure behavior.

Sources: [errors](https://google.github.io/styleguide/rustguide.html#reporting-errors),
[matching](https://google.github.io/styleguide/rustguide.html#dont-check-then-unwrap),
[`expect`](https://google.github.io/styleguide/rustguide.html#expect-over-unwrap),
[iteration](https://google.github.io/styleguide/rustguide.html#combinator-chains).

**Distinctions:**

- Google generally prohibits `catch_unwind`, with a defensive-use exception.
  Official Rust supports deliberate unwind boundaries but discourages using
  them for routine errors; aborting panics cannot be caught.
- Google prohibits accessing poisoned mutex contents. Official Rust exposes
  recovery and describes poisoning as advisory.

bip adopts neither blanket prohibition. Any such use needs an explanation
of its recovery behavior and relevant invariants; ordinary failures use
`Result`.

Comparison: [Google unwind policy](https://google.github.io/styleguide/rustguide.html#catch-unwind),
[official unwind documentation](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html),
[Google poisoning policy](https://google.github.io/styleguide/rustguide.html#poison),
[official poisoning documentation](https://doc.rust-lang.org/std/sync/struct.Mutex.html#poisoning).

## Documentation and unsafe code

Document public contracts and panic conditions. Prefer safe implementations;
isolate necessary unsafe code, explain its justification, and record safety
invariants with `SAFETY:` comments.

Sources: [documentation](https://google.github.io/styleguide/rustguide.html#writing-doc),
[unsafe code](https://google.github.io/styleguide/rustguide.html#unsafe).

For public API documentation, consult the official guidance on
[errors, panics, and safety](https://rust-lang.github.io/api-guidelines/documentation.html#function-docs-include-error-panic-and-safety-considerations-c-failure).
Examples should demonstrate meaningful usage.

## Scope and excluded platform policies

bip uses ordinary Cargo organization and Rust testing conventions unless
repository configuration establishes otherwise. Do not import Chromium's
small-crate/flat-layout requirements, custom import mechanism, async/thread
restrictions, Fuchsia mutex choice, or platform test frameworks.

Sources: [crate layout](https://google.github.io/styleguide/rustguide.html#structure),
[imports](https://google.github.io/styleguide/rustguide.html#import-macro),
[async](https://google.github.io/styleguide/rustguide.html#async-await-chrome),
[threads](https://google.github.io/styleguide/rustguide.html#std-thread),
[mutex](https://google.github.io/styleguide/rustguide.html#fuchsia-mutex),
[tests](https://google.github.io/styleguide/rustguide.html#test-framework).

Other Google rules, including leaf-only implementation organization and
cross-team `non_exhaustive` policies, are not automatically bip requirements.
Evaluate API compatibility needs in context; official Rust guidance favors
encapsulation while allowing public fields for passive data structures.

Comparison: [Google organization](https://google.github.io/styleguide/rustguide.html#non-leaf-modules),
[Google enums](https://google.github.io/styleguide/rustguide.html#non-exhaustive-enums),
[Google structs](https://google.github.io/styleguide/rustguide.html#non-exhaustive-structs),
[official encapsulation guidance](https://rust-lang.github.io/api-guidelines/future-proofing.html#structs-have-private-fields-c-struct-private).
