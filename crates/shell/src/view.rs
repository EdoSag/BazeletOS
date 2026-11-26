//! The shell's screens.
//!
//! Everything here is written in reading order and placed through
//! `ui::widget`, so the same code gives the English and the Hebrew layout.

use i18n::{Text, messages, tr};
use iced_core::{Alignment, Background, Border, Color, Length, Theme};
use iced_widget::{Column, Space, button, column, container};
use ui::{
    Edge, Element, Insets,
    runtime::FONT_BOLD,
    widget::{align, centered, label, padding, row},
};

use crate::model::{Message, Model};

/// Apps per row of the grid.
const COLUMNS: usize = 4;
/// Space around a screen's content.
const MARGIN: f32 = 20.0;
/// Colours for app tiles, used in turn.
const TILE_COLORS: [u32; 6] = [0x3b82f6, 0x22c55e, 0xf59e0b, 0xef4444, 0x8b5cf6, 0x06b6d4];

fn rgb(value: u32) -> Color {
    Color::from_rgb8((value >> 16) as u8, (value >> 8) as u8, value as u8)
}

fn muted(theme: &Theme) -> Color {
    theme.extended_palette().background.strong.color
}

/// A small button for the top bar.
fn bar_button<'a>(text: &'a str, message: Message) -> Element<'a, Message> {
    button(label(text).size(14))
        .padding([6, 12])
        .style(button::secondary)
        .on_press(message)
        .into()
}

/// The button that switches to the other language, named in that language.
fn language_button<'a>(model: &Model) -> Element<'a, Message> {
    bar_button(
        messages::language_name(model.locale.next()),
        Message::SwitchLanguage,
    )
}

/// The lock screen: the time, the date and an unlock button.
pub fn lock(model: &Model) -> Element<'_, Message> {
    let direction = model.locale.direction();
    let clock = model.clock;
    let date =
        messages::date(model.locale, clock.weekday, clock.day, clock.month).unwrap_or_default();

    let unlock = button(centered(label(tr(model.locale, Text::Unlock)).size(18)))
        .width(Length::Fill)
        .padding(14)
        .style(button::primary)
        .on_press(Message::Unlock);

    column![
        align(direction, Edge::End, language_button(model)),
        Space::new().height(96),
        centered(label(clock.time()).size(84).font(FONT_BOLD)),
        centered(label(date).size(18)),
        Space::new().height(Length::Fill),
        centered(
            label(tr(model.locale, Text::NoPasscodeYet))
                .size(13)
                .style(|theme: &Theme| iced_widget::text::Style {
                    color: Some(
                        theme
                            .extended_palette()
                            .background
                            .weak
                            .text
                            .scale_alpha(0.7)
                    ),
                })
        ),
        Space::new().height(12),
        unlock,
    ]
    .padding(MARGIN)
    .into()
}

/// One app in the grid: a coloured tile with the name's first letter, and
/// the name below it.
fn tile<'a>(model: &Model, index: usize) -> Element<'a, Message> {
    let Some(app) = model.apps.get(index) else {
        // An empty cell keeps the columns of a short last row in place.
        return Space::new().width(Length::FillPortion(1)).into();
    };
    let name = tr(model.locale, app.name);
    let initial: String = name.chars().take(1).collect();
    let color = rgb(TILE_COLORS[index % TILE_COLORS.len()]);

    let icon = container(label(initial).size(28).font(FONT_BOLD).color(Color::WHITE))
        .center(60)
        .style(move |_: &Theme| container::Style {
            background: Some(Background::Color(color)),
            border: Border {
                radius: 16.0.into(),
                ..Border::default()
            },
            ..container::Style::default()
        });

    button(
        column![icon, label(name).size(13)]
            .spacing(6)
            .align_x(Alignment::Center)
            .width(Length::Fill),
    )
    .width(Length::FillPortion(1))
    .padding(4)
    .style(button::text)
    .on_press(Message::Launch(index))
    .into()
}

/// The home screen: a bar with the lock and language buttons, and the apps.
pub fn home(model: &Model) -> Element<'_, Message> {
    let direction = model.locale.direction();

    let bar = row(
        direction,
        vec![
            label(tr(model.locale, Text::Apps))
                .size(22)
                .font(FONT_BOLD)
                .into(),
            Space::new().width(Length::Fill).into(),
            language_button(model),
            Space::new().width(8).into(),
            bar_button(tr(model.locale, Text::Lock), Message::Lock),
        ],
    );

    let mut grid = Column::new().spacing(18);
    for first in (0..model.apps.len()).step_by(COLUMNS) {
        let cells = (first..first + COLUMNS)
            .map(|index| tile(model, index))
            .collect();
        grid = grid.push(row(direction, cells).align_y(Alignment::Start));
    }

    let mut content = column![bar, Space::new().height(28), grid];
    if model.launch_failed {
        content = content.push(Space::new().height(20)).push(
            container(align(
                direction,
                Edge::Start,
                label(tr(model.locale, Text::CouldNotStart)).size(14),
            ))
            .padding(padding(direction, Insets::all(12.0)))
            .style(|theme: &Theme| container::Style {
                background: Some(Background::Color(muted(theme))),
                border: Border {
                    radius: 12.0.into(),
                    ..Border::default()
                },
                ..container::Style::default()
            }),
        );
    }
    content.padding(MARGIN).into()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use i18n::Locale;

    use super::*;
    use crate::{apps::App, model::Clock};

    const SIZE: (u32, u32) = (400, 800);

    fn model(locale: Locale) -> Model {
        let app = |name| App {
            name,
            program: PathBuf::from("/bin/true"),
        };
        let mut model = Model::new(
            locale,
            vec![app(Text::About), app(Text::Terminal)],
            Clock {
                hour: 14,
                minute: 42,
                weekday: 3,
                day: 25,
                month: 11,
            },
        );
        model.locked = false;
        model
    }

    fn render(element: Element<'_, Message>) -> Vec<u8> {
        let mut renderer = ui::runtime::renderer();
        ui::runtime::render(&mut renderer, &ui::runtime::theme(), element, SIZE, 1)
    }

    /// The average x position of the pixels with exactly this colour.
    fn mean_x(pixels: &[u8], color: u32) -> f32 {
        let wanted = [(color >> 16) as u8, (color >> 8) as u8, color as u8];
        let xs: Vec<u32> = pixels
            .chunks_exact(4)
            .enumerate()
            .filter(|(_, pixel)| pixel[..3] == wanted)
            .map(|(index, _)| index as u32 % SIZE.0)
            .collect();
        assert!(!xs.is_empty(), "colour {color:06x} is not on the screen");
        xs.iter().sum::<u32>() as f32 / xs.len() as f32
    }

    #[test]
    fn the_app_grid_mirrors_in_hebrew() {
        let middle = SIZE.0 as f32 / 2.0;
        let first = TILE_COLORS[0];
        let second = TILE_COLORS[1];

        let english = render(home(&model(Locale::En)));
        assert!(mean_x(&english, first) < mean_x(&english, second));
        assert!(mean_x(&english, first) < middle);

        let hebrew = render(home(&model(Locale::He)));
        assert!(mean_x(&hebrew, first) > mean_x(&hebrew, second));
        assert!(mean_x(&hebrew, first) > middle);

        // The first tile sits as far from its edge in both directions.
        let from_left = mean_x(&english, first);
        let from_right = SIZE.0 as f32 - mean_x(&hebrew, first);
        assert!(
            (from_left - from_right).abs() < 2.0,
            "{from_left} vs {from_right}"
        );
    }

    #[test]
    fn both_screens_draw_in_both_languages() {
        for locale in Locale::ALL {
            let model = model(locale);
            for pixels in [render(lock(&model)), render(home(&model))] {
                assert_eq!(pixels.len(), (SIZE.0 * SIZE.1 * 4) as usize);
                let background = &pixels[..4];
                assert!(pixels.chunks_exact(4).any(|pixel| pixel != background));
            }
        }
    }
}
