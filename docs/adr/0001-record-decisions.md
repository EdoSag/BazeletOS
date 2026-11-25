# 0001. Record architecture decisions

- Status: Accepted
- Date: 2026-11-25

## Context

The project is one part-time developer working with an AI assistant, and it
invites outside contributors early. Decisions made in conversation are lost
unless they are written down, and later contributors need to know not only
what was decided but why.

## Decision

We keep architecture decision records in `docs/adr`, one decision per file,
numbered in order.

- `docs/PLAN.md` describes the current state of the plan.
- An ADR explains why a decision in the plan was made or changed.
- A record is not edited after it is accepted, apart from its status line.
  A changed decision gets a new record that supersedes the old one.
- Choices that are hard to undo (licences, public names, on-disk formats,
  key handling, anything that touches a real device) need a record before
  the change is merged.

## Consequences

Small extra work for each significant change. In return the history of the
design can be read without the chat logs it came from.
