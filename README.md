# BazeletOS

A phone operating system that is independent of Android: a Linux base
([Nura](https://nura.eco), formerly postmarketOS) with our own userspace
written in Rust.

**Status: pre-alpha.** Nothing here runs on a phone yet. The first milestone
is a compositor that runs in a window on a Linux desktop and shows a lock
screen, a home screen and an app launcher, in English and Hebrew.

"BazeletOS" is a working name. It lives in one place,
[`crates/branding`](crates/branding), so it stays easy to change.

## What is in this repository

| Path | What it is |
| --- | --- |
| `crates/branding` | The product name and application IDs. Nothing else hard-codes them. |
| `crates/compositor` | The Wayland compositor, built on Smithay. A stub for now. |
| `crates/shell` | The lock screen and home screen. A stub for now. |
| `crates/ui` | Layout direction helpers, so every screen mirrors for right-to-left languages. |
| `crates/i18n` | Locales and helpers for mixed Hebrew and English text. |
| `xtask` | Developer commands, run as `cargo xtask <command>`. |
| `docs/PLAN.md` | The project plan: goals, security model, roadmap. |
| `docs/adr` | Architecture decision records: what was decided, and why. |

System services, the permission broker and the store get their own crates
when their milestone starts.

## Building

You need a recent stable Rust toolchain; `rust-toolchain.toml` pins the
version and `rustup` installs it for you.

```sh
cargo xtask check   # fmt, clippy and tests, the same checks CI runs
cargo xtask run     # start the session (prints a placeholder for now)
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Every commit needs a
`Signed-off-by` line (the Developer Certificate of Origin). Security issues
go through [SECURITY.md](SECURITY.md), not public issues.

## Licences

- Code in this repository: [GPL-3.0-or-later](LICENSE).
- Libraries that apps link against, once they exist:
  [MPL-2.0](LICENSES/MPL-2.0.txt). None exist yet.
- Documentation: [CC BY-SA 4.0](LICENSES/CC-BY-SA-4.0.txt).

The reasons are recorded in
[ADR 0003](docs/adr/0003-name-and-licences.md).
