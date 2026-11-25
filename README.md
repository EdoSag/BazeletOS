# BazeletOS

A phone operating system that is independent of Android: a Linux base
([Nura](https://nura.eco), formerly postmarketOS) with our own userspace
written in Rust.

**Status: pre-alpha.** Nothing here runs on a phone yet. What exists is a
compositor that runs in a window on a Linux desktop and shows a lock
screen, a home screen and an app launcher, in English and Hebrew. The lock
screen has no passcode yet.

"BazeletOS" is a working name. It lives in one place,
[`crates/branding`](crates/branding), so it stays easy to change.

## What is in this repository

| Path | What it is |
| --- | --- |
| `crates/branding` | The product name and application IDs. Nothing else hard-codes them. |
| `crates/compositor` | The Wayland compositor, built on Smithay. Runs in a window for now. |
| `crates/shell` | The lock screen and home screen. |
| `crates/about` | A small About app, the first app built on the system toolkit. |
| `crates/ui` | The toolkit layer: direction-aware layout, so every screen mirrors for right-to-left languages, and the code that shows screens on Wayland surfaces. |
| `crates/i18n` | Locales, interface strings and helpers for mixed Hebrew and English text. |
| `assets` | The bundled font. |
| `xtask` | Developer commands, run as `cargo xtask <command>`. |
| `docs/PLAN.md` | The project plan: goals, security model, roadmap. |
| `docs/adr` | Architecture decision records: what was decided, and why. |

System services, the permission broker and the store get their own crates
when their milestone starts.

## Building and running

You need Linux with a graphical session (Wayland or X11), a C compiler and
linker, and the development files for libxkbcommon. `rust-toolchain.toml`
pins the Rust version and `rustup` installs it for you.

```sh
# Arch, EndeavourOS
sudo pacman -S --needed base-devel libxkbcommon wayland mesa

# Debian, Ubuntu, Pop!_OS
sudo apt install build-essential libxkbcommon-dev libwayland-dev libegl1
```

Then:

```sh
cargo xtask run     # start the session in a window
cargo xtask check   # fmt, clippy and tests, the same checks CI runs
```

`cargo xtask run` opens a phone-sized window with the lock screen. Press
**Unlock** to reach the home screen, and press an app to start it. The
button in the corner switches between English and Hebrew. **Lock** returns
to the lock screen. Ctrl+Alt+Backspace closes the app in front; there is no
home gesture yet. Closing the window ends the session.

The home screen lists the About app and, if they are installed, a terminal
(for example `foot`), a file manager and a calculator.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Every commit needs a
`Signed-off-by` line (the Developer Certificate of Origin). Security issues
go through [SECURITY.md](SECURITY.md), not public issues.

## Licences

- Code in this repository: [GPL-3.0-or-later](LICENSE).
- Libraries that apps link against, once they exist:
  [MPL-2.0](LICENSES/MPL-2.0.txt). None exist yet.
- Documentation: [CC BY-SA 4.0](LICENSES/CC-BY-SA-4.0.txt).
- The bundled Heebo font: [SIL Open Font License 1.1](LICENSES/OFL-1.1.txt).
  See [`assets`](assets/README.md).

The reasons are recorded in
[ADR 0003](docs/adr/0003-name-and-licences.md).
