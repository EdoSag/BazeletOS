# Independent phone OS: project plan

Nov 25, 2026 · @Edo Sagron

## Goals and decisions so far

The goal is a phone OS that Google's Android restrictions cannot reach: a Linux base, our own userspace written in Rust, and Android apps only as an optional guest.

- **Independence:** nothing in the OS depends on Android code. Waydroid can run Android apps in a container, but the OS works without it.
- **Version one:** boots on one phone, shows our shell, makes a call, sends an SMS, connects to mobile data and Wi-Fi, installs an app from our store. It is not a market-ready phone.
- **Team:** Edo and Claude to start. Everything is open source from the first commit, and community help is invited early.
- **Apps:** Flutter, Tauri, Qt, GTK and web apps, packaged as Flatpaks and distributed through our own store. A just-in-case mirror of Flutter is kept.
- **Hardware:** open, long-supported phones first.
- **Organisation:** a foundation that owns the name and takes donations and grants.
- **Testing:** set up on a real phone once work starts.

## Hardware targets

Start development on a used OnePlus 6 or a PinePhone, and aim the first real release at the Fairphone 5. The first two run a mainline Linux kernel today; the Fairphone has the longest vendor support and its maker is funding mainline work.

| Phone | Why | Catch |
| --- | --- | --- |
| OnePlus 6 / 6T | Cheap second-hand, fast, mainline kernel ([ranked best used phone](https://computingforgeeks.com/?p=179702)) | Camera and VoLTE are weak points |
| PinePhone | Mainline, and one of the few phones reported to [make calls reliably](https://social.kernel.org/notice/B1yTSOA8JzRt5Su7Hc) | Slow hardware |
| Librem 5 | Mainline, hardware kill switches, modem on a removable card | About $799 and on backorder |
| Fairphone 5 | Fairphone's own engineer is [mainlining it](https://fosdem.org/2026/events/attachments/ZNAAZX-mainline_kernel_for_fairphones_-_2026_update/slides/266938/fosdem_20_rsemaak.pdf): audio and main camera in progress | Call audio, camera and VoLTE not finished |
| Pixel 3a | Long-standing community support | Old, no vendor updates |
| Pixel 7 Pro (spare) | Already owned | [No mainline kernel](<https://wiki.postmarketos.org/wiki/Google_Pixel_7_Pro_(google-cheetah)>), so Android drivers only. Keep for later experiments |

- **Rule for choosing:** mainline kernel first. A phone that needs Android vendor drivers (through Halium) reintroduces the dependency we are leaving.
- **VoLTE is the gate for Israel.** With 2G and 3G retired, a phone with no working VoLTE cannot place calls. Check each candidate on an Israeli carrier before committing.
- **Furi Labs FLX1s+** is rated the most usable Linux phone to buy new, but as far as I know it runs on Android drivers, so it fails the rule above. Not verified.

Open question: which carrier and bands the test SIM will use.

## Working with upstream

We build our OS as a layer on top of Nura and do not fork it. Nura is the new name of postmarketOS, [renamed on 27 September 2026](https://itsfoss.com/news/postmarketos-nura-rebranding/).

"Upstream" is the project our code sits on. There are two ways to use it:

- **Fork:** copy all of Nura and change it freely. We would then own every kernel package, every device port and every security update for the base system, forever.
- **Layer:** take Nura as it is and add our own package repository on top. Their base keeps receiving their updates; we only maintain what we wrote.

The layer approach is how the existing phone interfaces (Phosh, Plasma Mobile) ship on Nura today.

| We take from Nura | We write and own |
| --- | --- |
| Kernel and device ports | Shell and compositor |
| Base system and package manager | System services (telephony, network, power, audio) |
| Image builder and installer | Permission broker and sandbox |
| Security updates for the base | Store, core apps, update policy |

Working rules:

1. A bug in the base gets fixed in Nura and sent to them, not patched privately.
2. Our services talk to the base only through standard interfaces (D-Bus, Wayland). That keeps a move to another base, such as Debian-based Mobian, possible.
3. We keep a full mirror of the sources we depend on, so nobody can cut us off.

The cost is that their decisions affect us. A change to their init system or release schedule becomes our work too.

### Base system options

Nura stays the first choice, Mobian is the fallback, and PureOS is not a fit as a base. All three are open source.

| Base | Built on | Phones | Fit |
| --- | --- | --- | --- |
| Nura (formerly postmarketOS) | Alpine Linux; [six-month releases](https://endoflife.date/nura) | Hundreds of device ports, most of them partial | First choice: widest device reach and a small, fast base |
| [Mobian](https://linuxiac.com/?p=193704) | Debian 13; an official Debian derivative run by its community | Stable images for PinePhone, PinePhone Pro, Librem 5, Pixel 3a, OnePlus 6/6T and Pocophone F1; [Fairphone 4 and 5 with limits](https://theregister.com/2025/10/21/mobian_trixie) | Fallback: fewer phones, but Debian's huge package set and long support. Its installer images have no full-disk encryption yet |
| [PureOS](https://puri.sm/posts/pureos-crimson-development-report-april-2026-pureos-crimson-released/) | Debian 12; made by Purism | Purism's own Librem 5 and Liberty Phone | Not a fit: tied to one company's hardware and direction, and an older base than Mobian |

PureOS is still useful as proof. Its phone interface, Phosh, is a shell layered on a standard Linux base, which is the structure this plan uses. Phosh also runs on Mobian and Nura.

## Foundation, license and name

Start under a fiscal host and incorporate a foundation only once there is money and a community to govern. A fiscal host (Open Collective is the common one, from memory) holds donations and pays expenses for a fee, with no legal entity of our own.

**Governance**

- The foundation owns the name, the domains, the signing keys and the store. It does not own the code; the licenses keep that open.
- Contributions use a Developer Certificate of Origin sign-off, not a copyright assignment, so nobody can relicense the project later.
- A written trademark policy says who may ship something under our name.
- Where to incorporate (Israel, the Netherlands, Germany) affects grant eligibility and tax. This needs a lawyer; I am not one.

**Licenses**

| Part | License | Reason |
| --- | --- | --- |
| Shell, services, store | GPL-3.0-or-later | Forks must stay open |
| Libraries and SDK that apps link to | MPL-2.0 | App developers can use them under any license |
| Documentation | CC BY-SA 4.0 | Standard for open docs |

**Name**

The working name is **BazeletOS** (Bazelet, בזלת, is Hebrew for basalt), chosen on 25 November 2026 and recorded in [ADR 0003](adr/0003-name-and-licences.md). A web search found no clash in software, but the trademark registers have not been searched yet, so the name stays provisional and lives in one crate.

The suggestions considered before that:

Nura's team said a descriptive name could not be trademarked, which is one reason they [dropped "postmarketOS"](https://www.notebookcheck.net/postmarketOS-is-now-Nura-after-nearly-a-decade-of-bringing-Linux-to-phones.1410027.0.html). So the name should be short, invented or uncommon, and easy to say in Hebrew and English.

| Name | Meaning |
| --- | --- |
| Dror | Hebrew for freedom, and for the sparrow |
| Tzur | Hebrew for rock or stronghold |
| Kestrel | A small falcon that hovers in place |
| Ember | What stays lit when the fire is banked |
| Anchor | Holds its place whatever the current |
| Nomad | Carries its home with it |

None of these has been checked against trademarks, domains or existing software projects. That check comes before we print anything.

## Funding

NLnet's Restack fund is the best match, and its next deadline is 3 January 2027. FUTO's microgrant is the easiest first money.

| Funder | Fit | Amount | What it requires | How and when |
| --- | --- | --- | --- | --- |
| [NLnet Restack](https://nlnet.nl/funding.html) | Strong: it names "secure, privacy-friendly devices" and alternatives to closed technologies | 5,000 to 50,000 euro, can scale up later | Free and open source; individuals may apply | Online form; deadlines on the 3rd of every odd month, next 3 January 2027 |
| [FUTO microgrant](https://futo.tech/grants) | Good for an early project | 1,000 to 5,000 US dollars, one-time | A small or early-stage project; no formal criteria published | Email grantapps@futo.org; rolling |
| [Open Technology Fund, Internet Freedom Fund](https://www.opentech.fund/funds/internet-freedom-fund/) | Partial: only if framed around people facing surveillance or censorship | 10,000 to 900,000 US dollars; ideal ask 50,000 to 200,000 over 6 to 12 months | Individuals 18+ or organisations of any nationality, outside US-sanctioned countries | Concept note online, then a full proposal if invited; rolling; reply in 6 to 8 weeks |
| [Sovereign Tech Fund](https://sovereign.tech/programs/fund) | Not yet: it funds base technologies others depend on, and does not finance prototypes or user-facing apps | Work must cost more than 50,000 euro | OSI or FSF license; no other public funder for the same work | Online platform; revisit when one of our libraries is used by other projects |
| Donations | Always | Small and recurring | A fiscal host or foundation to receive them | Open Collective, GitHub Sponsors, Liberapay (from memory, not checked) |

Suggested order:

1. Publish the repository and this plan, then email FUTO.
2. Apply to NLnet Restack with a small, concrete first milestone, such as the Rust telephony service or the permission broker. March 2027 is more realistic than January if we want a working prototype to show.
3. Consider OTF once the security model is implemented and can be demonstrated.

What every application will ask for: a public repository, an open license, milestones with a cost for each, and who does the work.

Open questions:

- [ ] Does Restack require a European dimension, and does an Israel-based applicant qualify? Earlier NLnet funds [required one](https://nlnet.nl/core/guideforapplicants/); the Restack guide needs reading.
- [ ] Restack's own maximum per applicant.

## Security model

The OS must keep a user's data private from apps, networks, thieves and from us, and it must say plainly where it cannot. At version one it will be weaker than GrapheneOS on exploit hardening and hardware-backed protection; its advantage is independence and openness, and the gaps are listed at the end.

### What we protect

| Asset | Examples |
| --- | --- |
| Data at rest | Messages, photos, contacts, notes, app data |
| Secrets | Disk key, app credentials, SIM PIN, backup keys |
| Communications | Calls, SMS, network traffic and its metadata |
| Sensors and location | Microphone, camera, GPS, Wi-Fi and Bluetooth scans |
| System integrity | That the OS running is the one we published |
| Availability | Calls, emergency calls and alerts keep working |

### Who we defend against

| Adversary | What they can do | Version one stance |
| --- | --- | --- |
| Thief or finder | Holds a locked phone | Defend fully: encryption, lock screen, USB locked |
| Forensic extraction (seizure, border) | Physical access, time, commercial tools | Defend partly: strong passphrase and auto-reboot; limited without a secure element |
| Evil maid | Brief physical access, then returns the phone | Defend where the bootloader supports verified boot with our keys |
| Malicious or over-collecting app | Runs code on the phone | Defend fully: this is the core of the design |
| Android guest (Waydroid) | A whole second OS in a container | Treat as one untrusted app; off by default |
| Network attacker | Rogue Wi-Fi, fake base station, snooping carrier | Defend partly: encrypted DNS, VPN kill switch, 2G off |
| Remote exploit | Malicious web page, message, Bluetooth or baseband packet | Reduce: memory-safe code, sandboxed parsers, small attack surface |
| Modem and vendor firmware | Closed code with radio access | Assume untrusted; isolate as far as the hardware allows |
| Supply chain | Poisoned dependency, build server or update mirror | Defend: signed and reproducible builds, vetted dependencies |
| The project itself | Us, a compromised maintainer, a stolen key | Defend: no accounts, no telemetry, split signing keys, public builds |
| Targeted state attacker | Zero-days, hardware implants | Out of scope for version one, stated publicly |

### Controls, layer by layer

**Boot and system integrity**

- Verified boot chain using keys the user or the project controls, on phones whose bootloader allows relocking with custom keys.
- Read-only system partition checked with dm-verity; any change breaks boot or shows a warning.
- Rollback protection, so an attacker cannot install an older, vulnerable release.
- The lock screen shows the boot state (verified, unverified, unlocked bootloader).

**Storage**

- Full-disk encryption on by default, with no option to skip it at setup.
- Key derived from the user's passphrase with a memory-hard function (Argon2id).
- Where a secure element exists, the key is also bound to it so guesses are rate-limited in hardware.
- Where none exists, setup requires a longer passphrase and says why.
- Automatic reboot after a set idle period, returning the phone to its fully encrypted state.
- Per-app data directories, so a backup or wipe can act on one app.

**Updates**

- Atomic A/B image updates: the new system is written beside the old one and switched on reboot, with automatic fallback.
- Signed metadata following The Update Framework model: an offline root key, short-lived online keys, and mirrors that cannot forge anything.
- Reproducible builds, so anyone can confirm a release matches the source.
- A stated target for shipping security fixes, published with the first release.

**Apps**

- Every app runs in a sandbox (namespaces plus system-call filtering) with nothing granted by default.
- All access to camera, microphone, location, contacts, files, phone and network goes through one permission broker, which prompts at the moment of use.
- Network access is a permission the user can switch off per app.
- An indicator shows whenever the microphone, camera or location is in use, and a log shows which app used it.
- No advertising identifier, no stable hardware identifiers exposed to apps.
- Android apps reach the system only through the same broker.

**System services**

- Written in Rust; parsers for untrusted data (SMS, cell broadcast, media, Bluetooth) run in their own restricted processes.
- Each service has its own user, the minimum capabilities, and a mandatory access control profile.
- No service other than the smallest possible init runs as root.
- Message-bus policy lists exactly which service may call which.

**Kernel**

- Hardened configuration, signed modules only, lockdown mode on.
- Stay close to mainline long-term releases so fixes arrive quickly.
- Drivers we do not need are not built.

**Network and radio**

- Encrypted DNS by default; Wi-Fi MAC address randomised per network.
- Always-on VPN with a kill switch, and a per-app firewall.
- Connectivity and time checks go to servers we run or the user chooses, never to a third party.
- A switch to disable 2G, which blocks the most common fake-base-station downgrade.
- Prefer phones whose modem is a separate USB device (PinePhone, Librem 5), where it has no direct access to main memory.

**Lock screen and physical access**

- Passphrase or PIN; biometrics only as a convenience on top of it.
- USB data disabled while locked; charging only.
- Notification content hidden while locked by default.
- Later: a duress code that wipes or opens a decoy profile.

**Project and release security**

- Release keys on hardware tokens, with two of three maintainers needed to sign.
- Signed commits, mandatory review, and build servers that hold no long-lived secrets.
- Dependency auditing for every Rust crate we ship, and a published bill of materials per release.
- A security contact, a disclosure policy and public advisories from day one.

### Known gaps at version one

- Mainline-friendly phones mostly lack a secure element we can use, so a weak PIN can be brute-forced offline.
- Verified boot with our own keys depends on the bootloader; some target phones will only reach "tamper-evident".
- Modem firmware stays closed on every phone on the market.
- No hardened memory allocator or exploit mitigations comparable to GrapheneOS yet.
- The browser engine is the largest attack surface (see the browser section).
- Two people cannot review each other's every commit quickly; outside audit is needed before any security claim is marketed.

### Decisions needed

- [ ] Minimum passphrase rule on phones with no secure element.
- [ ] Access control framework: AppArmor, SELinux or Landlock.
- [ ] Whether a phone without relockable verified boot can be an official target.
- [ ] Whether the Zero-Trust apps' existing crypto core can serve as the system keystore.

## Core apps

The Zero-Trust suite already covers almost every core app: about 20 apps on a shared core, nearly all Flutter, including Phone, Contacts, Keyboard, Camera, Browser and Gallery ([repository list](https://api.github.com/users/EdoSag/repos?per_page=100&sort=updated)). Messages is in progress. Settings, Files and the Store are the only essentials with no code yet.

| Role | Zero-Trust app | Port work |
| --- | --- | --- |
| Dialer | Phone | Flutter UI carries over; call handling moves from Android's telephony APIs to our telephony service |
| Messages (SMS) | In progress, not yet public | Design it against our telephony service from the start |
| Contacts | Contacts | Linux build; connect to the system contacts store |
| Keyboard | Keyboard: native Kotlin, Java and C++ keyboard service forked from HeliBoard, plus a Flutter companion app | Rewrite the Android keyboard service as a Wayland input method. The C++ engines (whisper.cpp, llama.cpp) build for Linux; layouts, prediction and swipe decoding need porting from Kotlin and Java |
| Camera | Camera: native Kotlin capture on CameraX (based on GrapheneOS Camera), Flutter vault and settings | Rewrite capture on libcamera; replace Android Keystore and Tink in the vault with our keystore; quality depends on each phone's camera support |
| Gallery | Gallery | Linux build; media access through the permission broker |
| Browser | Browser | Depends on its engine: if it wraps Android's WebView, it needs a new one, and Servo is the candidate |
| Clock and Calendar | Clock, Calendar | Linux build; alarms and reminders need the power service to wake the phone |
| Notes, Tasks, Mail, Authenticator | Same names | Linux build |
| Optional store apps | Fitness, Meds, Time, Quests, Home, Family, Meet, Assist | Port after version one; Fitness needs a new data source if it reads Android's Health Connect |
| Settings, Files, Store | New | Tauri or native Rust |

Port work here is estimated from each app's type. Only the Keyboard and Camera repositories have been read.

- Each Flutter plugin the apps use needs a Linux implementation. Listing those plugins is the first porting task.
- All of them ship as Flatpaks through the store, with no special privileges. If our own apps need a permission the sandbox cannot grant, the sandbox design is wrong.
- Until our dialer and messages apps exist, the existing open-source Linux phone apps can stand in.

Notes from the [Keyboard](https://github.com/EdoSag/Zero-Trust-Keyboard) and [Camera](https://github.com/EdoSag/Zero-Trust-Camera) repositories:

- Both are AGPL-3.0, which is compatible with shipping them beside a GPL-3.0 shell.
- Both already support Hebrew with right-to-left layout, so they double as test cases for the shell.
- The keyboard is the larger port. An Android keyboard is a system service with its own drawing and input pipeline, and none of that API exists on Linux.
- The camera's metadata stripping, vault and scanner logic carry over; only capture and key storage are platform work.
- The keyboard's "codes from SMS" and inserts from other Zero-Trust apps should go through the permission broker on our OS.

Open question: which plugins the apps depend on, and whether `zero_trust_core` has any Android-only code.

## Browser engine

Adopt Servo now for the web views inside the OS, and ship a mature browser beside it until Servo passes a site list we define. Being a real embedder with real users is the most useful help we can give it.

**Where Servo stands**

- Open source under MPL 2.0, written in Rust, hosted by Linux Foundation Europe, and it already targets Linux, Android and OpenHarmony ([Wikipedia](<https://en.wikipedia.org/wiki/Servo_(software)>)).
- Monthly releases: [version 0.7](https://phoronix.com/news/Servo-0.7-Changes) covered September 2026.
- Still described as experimental. In July 2026 Google Maps and OpenStreetMap [rendered but had interaction problems](https://www.phoronix.com/news/Servo-0.4-Released).
- It runs on very little money: a June headline put it at [under 8,000 dollars a month](https://phoronix.com/news/Servo-0.3-Monthly-Changes).

**How we help it mature**

1. **Embed it.** Use Servo for in-app web views, help pages and installed web apps. Every bug we hit on a phone is one desktop testers miss.
2. **Test on phone hardware.** ARM Linux, touch input, small screens, on-screen keyboard and low memory are thinly covered today.
3. **Keep a site list.** Track the sites our users need (Israeli banks, gov.il, maps, webmail) and report each failure upstream with a reduced test case.
4. **Send fixes.** Our team writes Rust already, so fixes go upstream instead of into a private fork.
5. **Fund it.** Put a Servo work package in grant applications and donate through the foundation.

**Fallback**

- A Gecko-based browser ships as the default browser until Servo passes the site list.
- Tauri apps on Linux use WebKitGTK (from memory), so that engine will be on the system too.

Zero-Trust Browser already exists as the suite's browser. If its interface is separate from its engine, it becomes the front end for Servo and for the fallback engine, so users see one browser while the engine underneath changes.

Security note: a browser engine is the most attacked code on a phone, and Servo's sandboxing is young. It runs inside the strictest app sandbox we have, with no exceptions for being a system component.

Open question: the pass mark, meaning which sites and what share of them must work before Servo becomes the default.

## Push notifications

Build one system push service that speaks UnifiedPush, with a default server run by the foundation and the option to point at any other. The details below are from memory and need checking against the current UnifiedPush specification.

| Option | How it works | Verdict |
| --- | --- | --- |
| No push service | Each app keeps its own connection open | Drains the battery; acceptable only as a fallback |
| UnifiedPush | One on-device "distributor" holds a single connection to a push server; apps register an endpoint and their servers post to it | Recommended: open, self-hostable, already used by several open-source apps |
| Web Push | The standard browsers use, with encrypted payloads | Support it for installed web apps, through the same service |
| Our own protocol | A private equivalent of Google's service | No: developers would have to integrate yet another system |
| Google's service via microG | Lets Android apps in the container receive their normal pushes | Optional, off by default, and it contacts Google |

Design points:

- The service is a Rust daemon. Apps reach it through the permission broker, so receiving notifications is a permission like any other.
- Payloads are encrypted between the app's server and the app. The push server sees that a message arrived for a device, not its content.
- Users choose the server: ours, their own, or none.
- One connection, with keep-alive timing tuned so the modem can sleep, designed together with the power service.

Limit: apps that only support Google's or Apple's push will not deliver through this. That mostly affects Android apps, which is what the microG option is for.

## Emergency features and Hebrew support

Emergency calls and cell-broadcast alerts are release blockers: no build is called usable until both work on the target phone. Hebrew and right-to-left layout are built into the shell from its first screen.

**Emergency calls**

- Dialling an emergency number works from the lock screen, with no SIM, and with the SIM locked.
- The call takes priority over everything: it wakes the phone, cannot be blocked by an app, and survives a crashed shell because the telephony service handles it alone.
- The emergency number list comes from the SIM and the network, with a built-in fallback list per country.
- Sending location to the emergency service is a later phase.
- Real emergency numbers cannot be dialled for testing. The test plan needs the carrier's or regulator's approved method.

**Cell-broadcast alerts**

- The modem receives broadcast alerts and the telephony service passes them to a dedicated, minimal alert process.
- An alert wakes the phone, overrides silent mode, shows full-screen and repeats until acknowledged.
- As far as I know, Israel's Home Front Command sends alerts by cell broadcast; the exact channels and message formats need confirming.
- Alert text is untrusted input from the radio, so it is parsed in a restricted process.

**Hebrew and right-to-left**

- Every layout mirrors automatically for right-to-left languages; no screen is designed left-to-right only.
- Mixed Hebrew and English text follows the Unicode bidirectional algorithm, including in notifications, the dialer and the keyboard's suggestion bar.
- A Hebrew keyboard layout and a bundled Hebrew font ship in the first build.
- The UI toolkit is chosen partly on its right-to-left and text-shaping support. Chosen: iced, drawn in software on our own Wayland surfaces, with the Heebo font bundled ([ADR 0004](adr/0004-ui-toolkit-and-font.md)).
- Test strings include mixed-direction text, numbers inside Hebrew, and long words that must wrap.

Open questions:

- [ ] Which cell-broadcast channels Israeli carriers use, and whether the target modem exposes them.
- [ ] How to test emergency calling legally in Israel.

## Backup and migration

Backups are encrypted on the phone with a key only the user holds, and the same mechanism moves a user from one phone to another.

**Backup**

- Encrypted before leaving the phone. The destination never sees readable data or file names.
- Destinations: a local file or USB drive, the user's own server, or any standard storage service. No account with us is needed.
- Incremental and scheduled, running when the phone is charging and on Wi-Fi.
- Covers settings, contacts, messages, call log, photos and each app's data directory.
- Apps can mark data as excluded, and device-bound keys are never exported in usable form.
- A recovery phrase is shown once at setup. Lose it and the backup is unreadable, by design.

**Restore**

- Offered on first boot, before any account or app setup.
- Restores can be partial: one app, or only contacts and messages.
- Every backup format version stays readable by later releases.

**Phone-to-phone transfer**

- The new phone shows a QR code; the old phone scans it; the two connect directly over Wi-Fi or a USB cable.
- The transfer is encrypted with a key from the QR code and never touches a server.
- It uses the backup format, so there is one code path to test.

**Migrating from Android**

- A small companion app on the Android phone exports contacts, SMS, call log, calendar and photos, then sends them with the same QR pairing.
- App data inside Android apps generally cannot be exported by a third party. Apps with their own transfer feature, such as WhatsApp, use that.
- The companion app can be Flutter, sharing code with the phone-side importer.

Open question: write our own backup format or adopt an existing encrypted, deduplicating one.

## Testing

Three levels of tests, with real-phone testing set up once work starts.

| Level | Runs on | Catches |
| --- | --- | --- |
| Unit and service tests | Every commit, in CI | Logic errors in Rust services, permission decisions, parsers |
| Image tests | A virtual machine booting the full OS image | Boot, shell start-up, app install, update and rollback |
| Device tests | A real phone on a USB test rig | Calls, SMS, mobile data, suspend and wake, audio routing, battery drain |

- The device rig flashes each nightly build automatically and reports over a serial or USB connection.
- Call and SMS tests need a second phone and two SIMs. The spare Pixel 7 Pro can be the far end, and the Android side of migration tests.
- Parsers for untrusted input (SMS, cell broadcast, media) get fuzz testing from the start.
- A release needs all three levels green on every official target phone.

To buy or assign when we start: one target phone, two test SIMs, a USB hub with per-port power control.

## Roadmap

Five phases lead to version one, each ending at a gate you can see.

| Phase | Work | Gate |
| --- | --- | --- |
| 1. Foundations | Name, public repository, licenses, threat model agreed, first grant application | Plan and repository are public |
| 2. Boot and shell | Nura base on one phone, our Rust compositor, home screen and launcher, Hebrew and English | Our home screen runs on the phone |
| 3. Phone basics | Lock screen, settings, telephony, network, audio and power services, dialer and messages | A call, an SMS and mobile data all work |
| 4. Apps and store | Sandbox, permission broker, store, Zero-Trust apps ported, Flutter plugins for Linux | An app installs from the store and asks for a permission |
| 5. Safety and updates | Disk encryption, verified boot, signed A/B updates, emergency calls, cell-broadcast alerts | Version one released |

After version one: Android apps in Waydroid, push service, backup and migration, Servo as default, second phone.

Phase 2 has a desktop gate before the phone gate: one command starts the compositor in a window on a Linux desktop, shows a lock screen, unlocks to a home screen and launches an app, in English and Hebrew. That gate needs a basic lock screen, so the lock screen starts in phase 2 as a screen with no passcode; passcodes and what they unlock stay in phases 3 and 5. How the compositor, lock screen and home screen fit together is in [ADR 0005](adr/0005-shell-and-session-lock.md).

Each phase ends at something that can be shown on the phone, and the next phase does not start until it works. No dates are set yet; they depend on hours per week and on which phone we choose.

First steps:

- [x] Choose the name (BazeletOS, working name)
- [ ] Search the trademark registers and register a domain
- [x] Create the public repository with the licenses
- [x] Desktop gate: lock screen, home screen and app launch in a window
- [ ] Buy the development phone and two test SIMs
- [ ] List the plugins the Zero-Trust apps use
- [ ] Read NLnet Restack's guide and pick the January or March deadline
- [ ] Email FUTO about a microgrant
