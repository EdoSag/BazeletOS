# 0004. UI toolkit and bundled font

- Status: Accepted
- Date: 2026-11-25

## Context

The shell and the system apps need a UI toolkit. The plan asks for one
chosen partly on right-to-left and text-shaping support, and for a Hebrew
font in the first build. The code is GPL-3.0-or-later, and libraries that
apps link against will be MPL-2.0.

Two Rust toolkits were looked at: iced 0.14 and Slint 1.19. Neither mirrors
a layout for right-to-left languages by itself. Both shape and reorder
Hebrew text. iced is MIT licensed. Slint is GPL-3.0-only or under Slint's
own licences, which does not sit well with MPL-2.0 libraries for apps.
Only iced was actually rendered in a test (`spikes/rtl-iced` in the project
files); Slint was judged from its source and documentation.

iced's own window code cannot create the two kinds of surface the shell
needs: a session lock surface and a layer surface.

## Decision

Use iced for layout, widgets, text and drawing, and our own small runtime
(`crates/ui/src/runtime.rs`) to put the result on Wayland surfaces.

- The runtime uses smithay-client-toolkit for the Wayland side: app
  windows, layer surfaces and session lock surfaces.
- Screens are drawn in software with iced's tiny-skia renderer into shared
  memory buffers. No GPU is needed by the shell or by apps built this way.
- Layout direction is ours: screens are written in reading order and placed
  through `ui::widget`, which mirrors rows, padding and alignment.
- Text is aligned by placing shrink-to-fit text inside a container. iced's
  own text alignment clipped or hid Hebrew text in the rendering test.
- Bundle Heebo (Regular and Bold) as the system font. It covers Hebrew and
  Latin in one family, is licensed under the SIL Open Font License 1.1, and
  is compiled into the programs, so text looks the same on every machine.

## Consequences

- Hebrew and English use the same screen code. A test renders the home
  screen in both and checks that the grid mirrors.
- Software drawing redraws the whole surface on every change. That is fine
  for a lock screen and a launcher at this size. It is untested on phone
  hardware and may need damage tracking or a GPU renderer later.
- The runtime handles a pointer only. Touch, keyboard and text input are
  not implemented yet, so there are no text fields. Touch is needed before
  the shell is usable on a phone.
- We depend on iced's lower-level crates (`iced_core`, `iced_runtime`,
  `iced_widget`, `iced_tiny_skia`) instead of the `iced` crate. Their API
  may change between releases with less notice.
- Known defect from the rendering test, still open: a Hebrew line that
  contains an isolated phone number did not wrap.
- The font choice is easy to change: two files and one constant.
- If iced turns out not to fit, the screens and the runtime are the parts
  to replace; `i18n` and the direction rules in `ui` do not depend on it.
