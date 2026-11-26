//! The system shell: the lock screen and the home screen.
//!
//! The compositor starts this program and gives it the only connection that
//! may lock the session. The shell locks as soon as it starts.

mod apps;
mod model;
mod view;

use std::{
    path::Path,
    process::{Child, ExitCode},
    time::Duration,
};

use i18n::Locale;
use ui::runtime::{Action, Config, Element, Kind, Program, Role};

use crate::model::{Clock, Effect, Message, Model};

/// The current local time.
fn now() -> Clock {
    let now = jiff::ZonedDateTime::now();
    Clock {
        hour: now.hour().unsigned_abs(),
        minute: now.minute().unsigned_abs(),
        weekday: now.weekday().to_sunday_zero_offset().unsigned_abs(),
        day: now.day().unsigned_abs(),
        month: now.month().unsigned_abs(),
    }
}

struct Shell {
    model: Model,
    /// Apps started from here that have not exited yet.
    children: Vec<Child>,
}

impl Program for Shell {
    type Message = Message;

    fn view(&self, role: Role) -> Element<'_, Message> {
        match role {
            Role::Lock => view::lock(&self.model),
            Role::Home | Role::Window => view::home(&self.model),
        }
    }

    fn update(&mut self, message: Message) -> Vec<Action> {
        match self.model.update(message) {
            Effect::None => Vec::new(),
            Effect::Unlock => vec![Action::Unlock],
            Effect::Lock => vec![Action::Lock],
            Effect::Launch(app) => {
                match apps::launch(&app, self.model.locale) {
                    Ok(child) => {
                        self.children.push(child);
                        self.model.launch_failed = false;
                    }
                    Err(error) => {
                        eprintln!("could not start {}: {error}", app.program.display());
                        self.model.launch_failed = true;
                    }
                }
                Vec::new()
            }
        }
    }

    fn tick(&mut self) {
        self.model.clock = now();
        // Collect apps that exited, so they do not linger as zombies.
        self.children
            .retain_mut(|child| matches!(child.try_wait(), Ok(None)));
    }
}

fn main() -> ExitCode {
    let locale = Locale::from_env(|name| std::env::var(name).ok());
    let own_directory = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .unwrap_or_default();
    let path = std::env::var_os("PATH").unwrap_or_default();
    let apps = apps::discover(&own_directory, &path, |candidate| candidate.is_file());

    let shell = Shell {
        model: Model::new(locale, apps, now()),
        children: Vec::new(),
    };
    let config = Config {
        kind: Kind::Shell,
        app_id: branding::Component::Shell.app_id(),
        title: branding::PRODUCT_NAME.to_owned(),
        tick: Some(Duration::from_secs(1)),
    };
    match ui::runtime::run(shell, config) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{}: {error}", branding::Component::Shell.app_id());
            ExitCode::FAILURE
        }
    }
}
