# 0002. Build on Nura as a layer, never a fork

- Status: Accepted
- Date: 2026-11-25

## Context

A phone OS needs a kernel, device ports, a base system, a package manager
and security updates for all of them. One part-time developer cannot
maintain those. Nura (formerly postmarketOS) already does, on Alpine Linux,
for many phones. Mobian (Debian based) is a comparable project.

## Decision

BazeletOS is a set of our own packages added on top of an unmodified Nura
base. We do not fork the base.

- A bug in the base is fixed in Nura and sent upstream, not patched
  privately.
- Our code talks to the base only through standard interfaces such as
  Wayland and D-Bus, so that a move to another base stays possible. Mobian
  is the fallback.
- We write and own: the compositor and shell, the system services, the
  permission broker and sandbox policy, the store, and the update policy.

## Consequences

We inherit Nura's decisions and schedule. Its releases lose support about a
month after the next one, so our layer has to follow each six-month release
within weeks.

**Known tension with the security model.** The security model in
`docs/PLAN.md` asks for a verified read-only system partition, atomic A/B
image updates, signed kernel modules, kernel lockdown and a hardened kernel
configuration. A package layer on a stock Nura base does not provide those.
Each such control will need one of: work contributed upstream to Nura, our
own image build on top of Nura's packages, or an explicit statement that
the control is not met on a given device. This is not resolved. It must be
settled in a later ADR before phase 5 of the roadmap, and the security
model is not to be weakened quietly to make the conflict go away.
