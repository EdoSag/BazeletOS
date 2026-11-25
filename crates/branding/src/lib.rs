//! The product name and application IDs.
//!
//! This is the only crate that may spell the product name or the
//! application ID prefix. Everything else takes them from here, so a rename
//! is a change to this file. See `docs/adr/0003-name-and-licences.md`.

/// The product name as shown to people.
pub const PRODUCT_NAME: &str = "BazeletOS";

/// The reverse-DNS prefix for application IDs and D-Bus names.
pub const APP_ID_PREFIX: &str = "dev.sagron.bazeletos";

/// A component of the system that needs its own application ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Component {
    /// The Wayland compositor.
    Compositor,
    /// The lock screen and home screen.
    Shell,
}

impl Component {
    /// The last segment of the component's application ID.
    const fn segment(self) -> &'static str {
        match self {
            Self::Compositor => "Compositor",
            Self::Shell => "Shell",
        }
    }

    /// The component's application ID, for example
    /// `dev.sagron.bazeletos.Shell`.
    ///
    /// The same string is used as the Wayland `app_id` and as the D-Bus
    /// well-known name.
    #[must_use]
    pub fn app_id(self) -> String {
        format!("{APP_ID_PREFIX}.{}", self.segment())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// D-Bus well-known names: at least two elements separated by dots, each
    /// starting with a letter or underscore and holding only `[A-Za-z0-9_]`,
    /// at most 255 bytes in total.
    fn is_valid_bus_name(name: &str) -> bool {
        let elements: Vec<&str> = name.split('.').collect();
        name.len() <= 255
            && elements.len() >= 2
            && elements.iter().all(|element| {
                let mut chars = element.chars();
                chars
                    .next()
                    .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
                    && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
            })
    }

    #[test]
    fn app_ids_are_valid_bus_names() {
        for component in [Component::Compositor, Component::Shell] {
            let id = component.app_id();
            assert!(is_valid_bus_name(&id), "{id} is not a valid bus name");
        }
    }

    #[test]
    fn app_ids_start_with_the_prefix() {
        assert_eq!(Component::Shell.app_id(), "dev.sagron.bazeletos.Shell");
        assert!(Component::Compositor.app_id().starts_with(APP_ID_PREFIX));
    }

    #[test]
    fn product_name_is_not_empty() {
        assert!(!PRODUCT_NAME.trim().is_empty());
    }
}
