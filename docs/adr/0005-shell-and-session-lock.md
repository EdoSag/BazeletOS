# 0005. The shell is a client, and the session starts locked

- Status: Accepted
- Date: 2026-11-25

## Context

The security model says a locked phone must defend fully against someone
who holds it. That has to stay true when the lock screen program crashes.
The lock screen and home screen also should not live inside the
compositor, where a bug in drawing code would run with the compositor's
control over all input and output.

## Decision

The lock screen and home screen are drawn by a separate program, the
shell, which is a Wayland client. The compositor owns the lock state.

- The session starts locked. Until an unlock is accepted the compositor
  draws only the lock surface and sends input only to it. Apps and the home
  screen are left out of the frame, not covered.
- The shell uses two standard protocols: `ext-session-lock-v1` for the lock
  screen and `wlr-layer-shell` for the home screen.
- The compositor starts the shell itself and hands it an already connected
  socket. Only that connection can see the lock and layer shell globals.
  Apps connect to the public socket and cannot lock, unlock, or cover the
  screen.
- If the shell goes away while the session is locked, the session stays
  locked and the screen is blank until a new shell takes the lock. If the
  shell goes away while unlocked, the compositor starts it again and the
  new shell locks at once.
- A second client cannot take a lock that is already held.
- Apps are started by the shell for now, as plain child processes with the
  shell's privileged socket closed.

## Consequences

- The lock rules are a small state machine (`crates/compositor/src/session.rs`)
  with unit tests, apart from any Wayland code.
- There is no authentication yet. The lock screen has an Unlock button and
  says so on screen. Passcodes, key handling and what unlocking releases
  are separate decisions and need their own ADR before any code.
- The lock screen and home screen run in one process. Splitting the lock
  screen into its own smaller process is worth doing before it handles a
  passcode.
- Apps are not sandboxed and inherit the shell's environment. Starting
  apps moves to a launcher service with a sandbox in phase 4.
- The shell's privileges rest on which socket it uses. That is enough on a
  desktop. On a phone the compositor must also make sure the shell binary
  is the one from the verified system image.
- Verified only on a desktop: under X11 with software rendering, with
  screenshots, and started (not looked at) under a Wayland host.
