//! The About app: says what this system is.
//!
//! It is also the first ordinary app built on the system toolkit, so the
//! home screen always has something to launch.

use std::process::ExitCode;

use i18n::{Locale, Text, tr, tr_name};
use iced_core::Length;
use iced_widget::{Space, button, column};
use ui::{
    Edge,
    runtime::{Action, Config, Element, FONT_BOLD, Kind, Program, Role},
    widget::{align, centered, label},
};

struct About {
    locale: Locale,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Close,
}

impl Program for About {
    type Message = Message;

    fn view(&self, _role: Role) -> Element<'_, Message> {
        let direction = self.locale.direction();
        let version = format!(
            "{} {}",
            tr(self.locale, Text::Version),
            i18n::isolate_ltr(env!("CARGO_PKG_VERSION"))
        );
        column![
            align(
                direction,
                Edge::Start,
                label(tr_name(
                    self.locale,
                    Text::AboutHeading,
                    branding::PRODUCT_NAME
                ))
                .size(26)
                .font(FONT_BOLD)
            ),
            Space::new().height(8),
            align(direction, Edge::Start, label(version).size(15)),
            Space::new().height(20),
            align(
                direction,
                Edge::Start,
                label(tr(self.locale, Text::AboutBody)).size(16)
            ),
            Space::new().height(Length::Fill),
            button(centered(label(tr(self.locale, Text::Close)).size(18)))
                .width(Length::Fill)
                .padding(14)
                .style(button::secondary)
                .on_press(Message::Close),
        ]
        .padding(20)
        .into()
    }

    fn update(&mut self, message: Message) -> Vec<Action> {
        match message {
            Message::Close => vec![Action::Exit],
        }
    }
}

fn main() -> ExitCode {
    let about = About {
        locale: Locale::from_env(|name| std::env::var(name).ok()),
    };
    let config = Config {
        kind: Kind::App,
        app_id: branding::Component::About.app_id(),
        title: tr_name(about.locale, Text::AboutHeading, branding::PRODUCT_NAME),
        tick: None,
    };
    match ui::runtime::run(about, config) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{}: {error}", branding::Component::About.app_id());
            ExitCode::FAILURE
        }
    }
}
