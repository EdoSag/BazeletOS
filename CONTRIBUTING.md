# Contributing

Thank you for helping. This project is small and early, so the rules are
short.

## Developer Certificate of Origin

We do not ask for a copyright assignment. Instead every commit must carry a
sign-off line, which certifies that you have the right to submit the work
under the project's licence. The full text is the
[Developer Certificate of Origin 1.1](https://developercertificate.org/).

Add the line with `git commit -s`:

```
Signed-off-by: Your Name <you@example.com>
```

CI rejects pull requests that contain a commit without it.

## Before you open a pull request

Run the same checks CI runs:

```sh
cargo xtask check
```

That is `cargo fmt --check`, `cargo clippy` with warnings as errors, and
`cargo test`. CI also runs `cargo deny` for licences and known
vulnerabilities in dependencies.

## What we expect in a change

- **Small commits** that each do one thing.
- **Tests for logic.** Service logic, permission decisions, parsers and
  layout helpers need unit tests.
- **No `unsafe` without a reason.** The workspace denies `unsafe` code. If
  you need it, allow it on the smallest possible item and write a
  `// SAFETY:` comment that explains why it is sound.
- **Right-to-left from the start.** No screen may be designed left-to-right
  only. Use the helpers in `crates/ui` and `crates/i18n`.
- **No hard-coded product name or application ID.** Take them from
  `crates/branding`.
- **Decisions get recorded.** If a change alters a decision in
  `docs/PLAN.md`, update the plan and add an ADR in `docs/adr` that says
  why.
- **The security model is binding.** If a design cannot meet the security
  model in `docs/PLAN.md`, say so in the pull request; do not weaken it
  quietly.

## New dependencies

Every crate we ship is audited. When you add one, say in the pull request
why it is needed and what its licence is. `deny.toml` lists the licences we
accept.

## Conduct

Everyone taking part is expected to follow the
[code of conduct](CODE_OF_CONDUCT.md).
