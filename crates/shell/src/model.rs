//! What the shell knows and how it reacts, apart from drawing.

use i18n::Locale;

use crate::apps::App;

/// The wall-clock time shown on the lock screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Clock {
    /// Hour, 0 to 23.
    pub hour: u8,
    /// Minute, 0 to 59.
    pub minute: u8,
    /// Day of the week, Sunday is 0.
    pub weekday: u8,
    /// Day of the month, from 1.
    pub day: u8,
    /// Month, January is 1.
    pub month: u8,
}

impl Clock {
    /// The time as `14:05`.
    pub fn time(self) -> String {
        format!("{}:{:02}", self.hour, self.minute)
    }
}

/// Something the user did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message {
    /// Pressed unlock on the lock screen.
    Unlock,
    /// Pressed lock on the home screen.
    Lock,
    /// Pressed the language switch.
    SwitchLanguage,
    /// Pressed the app at this position in the grid.
    Launch(usize),
}

/// What the shell must do after a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    /// Nothing outside the model.
    None,
    /// Ask the compositor to unlock the session.
    Unlock,
    /// Ask the compositor to lock the session.
    Lock,
    /// Start an app.
    Launch(App),
}

/// The shell's state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Model {
    /// The interface language.
    pub locale: Locale,
    /// The apps on the home screen.
    pub apps: Vec<App>,
    /// The current time.
    pub clock: Clock,
    /// Whether the session is locked. Starts locked.
    pub locked: bool,
    /// Whether the last app launch failed.
    pub launch_failed: bool,
}

impl Model {
    /// A locked shell.
    pub fn new(locale: Locale, apps: Vec<App>, clock: Clock) -> Self {
        Self {
            locale,
            apps,
            clock,
            locked: true,
            launch_failed: false,
        }
    }

    /// Applies a message and says what to do next.
    pub fn update(&mut self, message: Message) -> Effect {
        match message {
            Message::SwitchLanguage => {
                self.locale = self.locale.next();
                Effect::None
            }
            Message::Unlock if self.locked => {
                self.locked = false;
                Effect::Unlock
            }
            Message::Lock if !self.locked => {
                self.locked = true;
                self.launch_failed = false;
                Effect::Lock
            }
            // The home screen cannot be reached while locked, but a launch
            // is refused here as well rather than trusting that.
            Message::Launch(index) if !self.locked => match self.apps.get(index) {
                Some(app) => Effect::Launch(app.clone()),
                None => Effect::None,
            },
            Message::Unlock | Message::Lock | Message::Launch(_) => Effect::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use i18n::Text;

    use super::*;

    fn model() -> Model {
        let apps = vec![App {
            name: Text::About,
            program: PathBuf::from("/opt/about"),
        }];
        Model::new(Locale::En, apps, Clock::default())
    }

    #[test]
    fn starts_locked() {
        assert!(model().locked);
    }

    #[test]
    fn nothing_launches_while_locked() {
        let mut model = model();
        assert_eq!(model.update(Message::Launch(0)), Effect::None);
    }

    #[test]
    fn unlock_then_launch() {
        let mut model = model();
        assert_eq!(model.update(Message::Unlock), Effect::Unlock);
        assert!(matches!(
            model.update(Message::Launch(0)),
            Effect::Launch(_)
        ));
        assert_eq!(model.update(Message::Launch(7)), Effect::None);
    }

    #[test]
    fn unlock_and_lock_only_act_once() {
        let mut model = model();
        assert_eq!(model.update(Message::Lock), Effect::None);
        assert_eq!(model.update(Message::Unlock), Effect::Unlock);
        assert_eq!(model.update(Message::Unlock), Effect::None);
        assert_eq!(model.update(Message::Lock), Effect::Lock);
        assert!(model.locked);
    }

    #[test]
    fn the_language_switch_works_on_both_screens() {
        let mut model = model();
        assert_eq!(model.update(Message::SwitchLanguage), Effect::None);
        assert_eq!(model.locale, Locale::He);
        model.update(Message::Unlock);
        model.update(Message::SwitchLanguage);
        assert_eq!(model.locale, Locale::En);
    }

    #[test]
    fn the_clock_pads_minutes() {
        let clock = Clock {
            hour: 9,
            minute: 5,
            ..Clock::default()
        };
        assert_eq!(clock.time(), "9:05");
    }
}
