//! The lock screen and home screen.
//!
//! This is a placeholder. The shell arrives with the first milestone and
//! runs as Wayland clients of the compositor, so that a crash in the shell
//! cannot unlock the session.

use branding::{Component, PRODUCT_NAME};
use i18n::Locale;
use ui::Edge;

fn main() {
    println!(
        "{PRODUCT_NAME} shell ({}): not implemented yet.",
        Component::Shell.app_id()
    );
    for locale in Locale::ALL {
        println!(
            "  locale {}: layout starts on the {:?} side",
            locale.tag(),
            Edge::Start.side(locale.direction())
        );
    }
}
