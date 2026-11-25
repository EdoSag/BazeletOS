# 0003. Working name, application ID and licences

- Status: Accepted
- Date: 2026-11-25

## Context

The repository needed a name, an application ID prefix and licence files
before any code could be published. All three are awkward to change once
other people depend on them.

## Decision

**Name.** The working name is BazeletOS. Bazelet (בזלת) is Hebrew for
basalt. It is short, easy to say in Hebrew and English, and does not
describe an operating system, which matters for a trademark.

**Application ID prefix.** `dev.sagron.bazeletos`, used for D-Bus names and
application IDs, for example `dev.sagron.bazeletos.Shell`.

**One place for both.** The name and the prefix are defined only in
`crates/branding`. Crates are named by role (`compositor`, `shell`, `ui`),
not by brand, so a rename touches one crate.

**Licences.**

| Part | Licence | Reason |
| --- | --- | --- |
| Compositor, shell, services, store | GPL-3.0-or-later | Forks must stay open |
| Libraries and SDK that apps link to | MPL-2.0 | Apps can use them under any licence, while changes to our files stay open |
| Documentation | CC BY-SA 4.0 | Standard for open documentation |

Contributions are accepted under the Developer Certificate of Origin, with
no copyright assignment, so nobody can relicense the project alone later.

## Consequences

- The name is provisional. A web search on 2026-11-25 found no software or
  telecom product called Bazelet; the name is used by a medical cannabis
  producer and by a winery in Israel. The USPTO, EUIPO, WIPO and Israeli
  trademark registers have **not** been searched. That search, and
  registering a domain, must happen before the name is used in any release
  or printed material.
- "Or later" lets the project move to a future GPL version without asking
  every contributor. It also means we cannot link code that is
  GPL-3.0-only into a file we want to keep "or later"; the combined work
  would be GPL-3.0-only.
- MPL-2.0 was chosen over Apache-2.0 for libraries. No library exists yet,
  so this is cheap to revisit until the first one is published.
- The application ID prefix assumes control of the `sagron.dev` domain.
