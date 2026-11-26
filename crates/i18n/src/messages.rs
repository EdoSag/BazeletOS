//! The system's own interface strings, in every supported language.
//!
//! The strings are compiled in for now. A translation file format is an
//! on-disk format and will be chosen in an ADR before translators need it.
//!
//! `{name}` stands for the product name, which the caller passes in, so the
//! name itself is spelled only in the `branding` crate.

use crate::Locale;

/// A piece of interface text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Text {
    /// The button that unlocks the lock screen.
    Unlock,
    /// Shown on the lock screen while there is no passcode support.
    NoPasscodeYet,
    /// The heading above the app grid.
    Apps,
    /// The button that locks the session.
    Lock,
    /// App name: about this system.
    About,
    /// App name: terminal.
    Terminal,
    /// App name: file manager.
    Files,
    /// App name: calculator.
    Calculator,
    /// Shown when an app could not be started.
    CouldNotStart,
    /// About: heading. Takes `{name}`.
    AboutHeading,
    /// About: the version label.
    Version,
    /// About: what this build is.
    AboutBody,
    /// A button that closes the current app.
    Close,
}

impl Text {
    /// Every text, for tests.
    pub const ALL: [Self; 13] = [
        Self::Unlock,
        Self::NoPasscodeYet,
        Self::Apps,
        Self::Lock,
        Self::About,
        Self::Terminal,
        Self::Files,
        Self::Calculator,
        Self::CouldNotStart,
        Self::AboutHeading,
        Self::Version,
        Self::AboutBody,
        Self::Close,
    ];
}

/// The text in the given language.
#[must_use]
pub const fn tr(locale: Locale, text: Text) -> &'static str {
    match locale {
        Locale::En => match text {
            Text::Unlock => "Unlock",
            Text::NoPasscodeYet => "Development build. There is no passcode yet.",
            Text::Apps => "Apps",
            Text::Lock => "Lock",
            Text::About => "About",
            Text::Terminal => "Terminal",
            Text::Files => "Files",
            Text::Calculator => "Calculator",
            Text::CouldNotStart => "The app could not be started.",
            Text::AboutHeading => "About {name}",
            Text::Version => "Version",
            Text::AboutBody => {
                "An early development build. It runs in a window on a desktop and is not ready for a phone."
            }
            Text::Close => "Close",
        },
        Locale::He => match text {
            Text::Unlock => "ביטול נעילה",
            Text::NoPasscodeYet => "גרסת פיתוח. עדיין אין קוד נעילה.",
            Text::Apps => "יישומים",
            Text::Lock => "נעילה",
            Text::About => "אודות",
            Text::Terminal => "מסוף",
            Text::Files => "קבצים",
            Text::Calculator => "מחשבון",
            Text::CouldNotStart => "לא ניתן היה להפעיל את היישום.",
            Text::AboutHeading => "אודות {name}",
            Text::Version => "גרסה",
            Text::AboutBody => {
                "גרסת פיתוח מוקדמת. היא פועלת בחלון על מחשב שולחני ואינה מוכנה לטלפון."
            }
            Text::Close => "סגירה",
        },
    }
}

/// The text with `{name}` replaced by a name.
///
/// The name is isolated, so a Latin name inside a Hebrew sentence does not
/// reorder the words around it.
#[must_use]
pub fn tr_name(locale: Locale, text: Text, name: &str) -> String {
    tr(locale, text).replace("{name}", &crate::isolate_unknown(name))
}

/// A language's own name, for the language switch.
#[must_use]
pub const fn language_name(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "English",
        Locale::He => "עברית",
    }
}

/// A calendar date as written in the given language, for example
/// "Wednesday, 25 November" or "יום רביעי, 25 בנובמבר".
///
/// `weekday` counts from Sunday as 0 and `month` from January as 1. Values
/// out of range give `None`.
#[must_use]
pub fn date(locale: Locale, weekday: u8, day: u8, month: u8) -> Option<String> {
    const EN_DAYS: [&str; 7] = [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ];
    const EN_MONTHS: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    const HE_DAYS: [&str; 7] = [
        "יום ראשון",
        "יום שני",
        "יום שלישי",
        "יום רביעי",
        "יום חמישי",
        "יום שישי",
        "שבת",
    ];
    const HE_MONTHS: [&str; 12] = [
        "ינואר",
        "פברואר",
        "מרץ",
        "אפריל",
        "מאי",
        "יוני",
        "יולי",
        "אוגוסט",
        "ספטמבר",
        "אוקטובר",
        "נובמבר",
        "דצמבר",
    ];
    if !(1..=31).contains(&day) {
        return None;
    }
    let weekday = usize::from(weekday);
    let month = usize::from(month.checked_sub(1)?);
    Some(match locale {
        Locale::En => format!("{}, {day} {}", EN_DAYS.get(weekday)?, EN_MONTHS.get(month)?),
        // "ב" joins the month: "25 in November".
        Locale::He => format!(
            "{}, {day} ב{}",
            HE_DAYS.get(weekday)?,
            HE_MONTHS.get(month)?
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has_hebrew(text: &str) -> bool {
        text.chars().any(|c| ('\u{0590}'..='\u{05FF}').contains(&c))
    }

    #[test]
    fn every_text_exists_in_every_language() {
        for text in Text::ALL {
            for locale in Locale::ALL {
                assert!(!tr(locale, text).is_empty(), "{text:?} in {locale:?}");
            }
        }
    }

    #[test]
    fn hebrew_texts_are_in_hebrew() {
        for text in Text::ALL {
            assert!(has_hebrew(tr(Locale::He, text)), "{text:?}");
            assert!(!has_hebrew(tr(Locale::En, text)), "{text:?}");
        }
    }

    #[test]
    fn placeholders_match_between_languages() {
        for text in Text::ALL {
            let en = tr(Locale::En, text).contains("{name}");
            let he = tr(Locale::He, text).contains("{name}");
            assert_eq!(en, he, "{text:?}");
        }
    }

    #[test]
    fn a_name_is_isolated_inside_a_sentence() {
        let heading = tr_name(Locale::He, Text::AboutHeading, "Name");
        assert_eq!(heading, "אודות \u{2068}Name\u{2069}");
    }

    #[test]
    fn dates_are_written_per_language() {
        assert_eq!(
            date(Locale::En, 3, 25, 11).as_deref(),
            Some("Wednesday, 25 November")
        );
        assert_eq!(
            date(Locale::He, 3, 25, 11).as_deref(),
            Some("יום רביעי, 25 בנובמבר")
        );
    }

    #[test]
    fn dates_out_of_range_are_refused() {
        assert_eq!(date(Locale::En, 7, 1, 1), None);
        assert_eq!(date(Locale::En, 0, 0, 1), None);
        assert_eq!(date(Locale::En, 0, 1, 0), None);
        assert_eq!(date(Locale::En, 0, 1, 13), None);
    }
}
