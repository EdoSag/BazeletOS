//! The apps shown on the home screen.
//!
//! There is no app store or app database yet. The grid holds the system's
//! own About app and a few well-known desktop programs, each shown only if
//! it is installed. This list goes away when real app packages exist.

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};

use i18n::{Locale, Text};

/// An app on the home screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct App {
    /// The app's name, as a translatable text.
    pub name: Text,
    /// The program to start.
    pub program: PathBuf,
}

/// Programs to look for, by app. The first one found is used.
const CANDIDATES: [(Text, &[&str]); 3] = [
    (
        Text::Terminal,
        &[
            "foot",
            "alacritty",
            "kitty",
            "cosmic-term",
            "konsole",
            "weston-terminal",
        ],
    ),
    (
        Text::Files,
        &["cosmic-files", "thunar", "pcmanfm-qt", "dolphin", "nemo"],
    ),
    (
        Text::Calculator,
        &["galculator", "qalculate-gtk", "kcalc", "gnome-calculator"],
    ),
];

/// Finds the first of `names` in the directories of a `PATH` value.
pub fn find_program(
    names: &[&str],
    path: &OsString,
    exists: impl Fn(&Path) -> bool,
) -> Option<PathBuf> {
    names.iter().find_map(|name| {
        std::env::split_paths(path)
            .map(|directory| directory.join(name))
            .find(|candidate| exists(candidate))
    })
}

/// The apps to show: About first, then whatever is installed.
///
/// `own_directory` is where this program lives; the About app is built next
/// to it.
pub fn discover(own_directory: &Path, path: &OsString, exists: impl Fn(&Path) -> bool) -> Vec<App> {
    let mut apps = Vec::new();
    let about = own_directory.join("about");
    if exists(&about) {
        apps.push(App {
            name: Text::About,
            program: about,
        });
    }
    for (name, names) in CANDIDATES {
        if let Some(program) = find_program(names, path, &exists) {
            apps.push(App { name, program });
        }
    }
    apps
}

/// Starts an app as a Wayland client of this session.
///
/// The app connects to the public socket named in `WAYLAND_DISPLAY`. The
/// shell's own privileged connection is not passed on: its descriptor is
/// close-on-exec and `WAYLAND_SOCKET` is removed.
pub fn launch(app: &App, locale: Locale) -> std::io::Result<Child> {
    Command::new(&app.program)
        .env_remove("WAYLAND_SOCKET")
        // Keep desktop toolkits from opening a window on the host desktop.
        .env_remove("DISPLAY")
        .env("GDK_BACKEND", "wayland")
        .env("QT_QPA_PLATFORM", "wayland")
        .env("LANG", locale.posix())
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .stdin(Stdio::null())
        .spawn()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path() -> OsString {
        OsString::from("/usr/local/bin:/usr/bin")
    }

    #[test]
    fn finds_the_first_installed_candidate() {
        let found = find_program(&["foot", "kitty"], &path(), |p| {
            p == Path::new("/usr/bin/kitty")
        });
        assert_eq!(found, Some(PathBuf::from("/usr/bin/kitty")));
    }

    #[test]
    fn earlier_candidates_win() {
        let found = find_program(&["foot", "kitty"], &path(), |_| true);
        assert_eq!(found, Some(PathBuf::from("/usr/local/bin/foot")));
    }

    #[test]
    fn nothing_installed_means_no_apps() {
        assert!(discover(Path::new("/opt/system"), &path(), |_| false).is_empty());
    }

    #[test]
    fn about_comes_first_when_present() {
        let apps = discover(Path::new("/opt/system"), &path(), |p| {
            p == Path::new("/opt/system/about") || p == Path::new("/usr/bin/foot")
        });
        let names: Vec<Text> = apps.iter().map(|app| app.name).collect();
        assert_eq!(names, [Text::About, Text::Terminal]);
        assert_eq!(apps[0].program, PathBuf::from("/opt/system/about"));
    }
}
