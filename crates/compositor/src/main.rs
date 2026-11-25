//! The Wayland compositor.
//!
//! This is a placeholder. The compositor arrives with the first milestone:
//! a Smithay compositor that runs in a window on a Linux desktop.

use branding::{Component, PRODUCT_NAME};

fn main() {
    println!(
        "{PRODUCT_NAME} compositor ({}): not implemented yet.",
        Component::Compositor.app_id()
    );
}
