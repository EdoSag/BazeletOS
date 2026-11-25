//! Locales, text direction and helpers for mixed-direction text.
//!
//! Hebrew is a first-class language of the system, so text direction is
//! part of the lowest layer. The helpers here wrap pieces of text in
//! Unicode direction controls so that a phone number or an English name
//! inside a Hebrew sentence is drawn in the right order.

/// Whether text and layout run left-to-right or right-to-left.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    /// Left-to-right, as in English.
    Ltr,
    /// Right-to-left, as in Hebrew.
    Rtl,
}

/// A language the system's own interface is translated into.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Locale {
    /// English.
    En,
    /// Hebrew.
    He,
}

impl Locale {
    /// Every supported locale.
    pub const ALL: [Self; 2] = [Self::En, Self::He];

    /// The direction this locale's text and layout run in.
    #[must_use]
    pub const fn direction(self) -> Direction {
        match self {
            Self::En => Direction::Ltr,
            Self::He => Direction::Rtl,
        }
    }

    /// The locale's BCP 47 language tag.
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::He => "he",
        }
    }

    /// Finds the locale for a language tag such as `he`, `he-IL` or
    /// `en_US.UTF-8`. Only the language part is considered.
    #[must_use]
    pub fn from_tag(tag: &str) -> Option<Self> {
        let language = tag.split(['-', '_', '.', '@']).next()?;
        Self::ALL
            .into_iter()
            .find(|locale| locale.tag().eq_ignore_ascii_case(language))
    }
}

/// U+2066 LEFT-TO-RIGHT ISOLATE.
const LRI: char = '\u{2066}';
/// U+2067 RIGHT-TO-LEFT ISOLATE.
const RLI: char = '\u{2067}';
/// U+2068 FIRST STRONG ISOLATE.
const FSI: char = '\u{2068}';
/// U+2069 POP DIRECTIONAL ISOLATE.
const PDI: char = '\u{2069}';

fn is_isolate_control(c: char) -> bool {
    matches!(c, LRI | RLI | FSI | PDI)
}

fn wrap(opener: char, text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2 * opener.len_utf8());
    out.push(opener);
    // Drop isolate controls already in the text. Left in, an unmatched one
    // could end our isolate early and let the text reorder its surroundings.
    out.extend(text.chars().filter(|c| !is_isolate_control(*c)));
    out.push(PDI);
    out
}

/// Marks `text` as a unit with a fixed direction, isolated from the text
/// around it.
///
/// Use this for a whole label, so that a Hebrew sentence that happens to
/// start with an English word is still laid out right-to-left.
#[must_use]
pub fn isolate(direction: Direction, text: &str) -> String {
    match direction {
        Direction::Ltr => wrap(LRI, text),
        Direction::Rtl => wrap(RLI, text),
    }
}

/// Marks `text` as left-to-right and isolated from the text around it.
///
/// Use this for phone numbers, addresses, file paths and similar values
/// placed inside a sentence in any language. Without it, a number such as
/// `+972-50-123-4567` inside Hebrew text is drawn with its parts reordered.
#[must_use]
pub fn isolate_ltr(text: &str) -> String {
    wrap(LRI, text)
}

/// Isolates `text` from the text around it and lets its own first strong
/// character decide its direction.
///
/// Use this for text whose language is unknown, such as a contact name or
/// a message preview. Such text is untrusted: isolating it stops it from
/// reordering the interface text next to it.
#[must_use]
pub fn isolate_unknown(text: &str) -> String {
    wrap(FSI, text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hebrew_is_right_to_left() {
        assert_eq!(Locale::He.direction(), Direction::Rtl);
        assert_eq!(Locale::En.direction(), Direction::Ltr);
    }

    #[test]
    fn from_tag_reads_only_the_language() {
        assert_eq!(Locale::from_tag("he"), Some(Locale::He));
        assert_eq!(Locale::from_tag("he-IL"), Some(Locale::He));
        assert_eq!(Locale::from_tag("HE_il.UTF-8"), Some(Locale::He));
        assert_eq!(Locale::from_tag("en_US.UTF-8"), Some(Locale::En));
        assert_eq!(Locale::from_tag("fr"), None);
        assert_eq!(Locale::from_tag(""), None);
    }

    #[test]
    fn tags_round_trip() {
        for locale in Locale::ALL {
            assert_eq!(Locale::from_tag(locale.tag()), Some(locale));
        }
    }

    #[test]
    fn isolate_ltr_wraps_a_phone_number() {
        assert_eq!(
            isolate_ltr("+972-50-123-4567"),
            "\u{2066}+972-50-123-4567\u{2069}"
        );
    }

    #[test]
    fn isolate_uses_the_direction() {
        assert_eq!(isolate(Direction::Rtl, "שלום"), "\u{2067}שלום\u{2069}");
        assert_eq!(isolate(Direction::Ltr, "hello"), "\u{2066}hello\u{2069}");
    }

    #[test]
    fn isolate_unknown_uses_first_strong() {
        assert_eq!(isolate_unknown("דנה"), "\u{2068}דנה\u{2069}");
    }

    #[test]
    fn untrusted_text_cannot_close_the_isolate_early() {
        // A contact name that tries to break out with its own controls.
        let hostile = "a\u{2069}\u{2067}b\u{2066}c\u{2068}";
        let wrapped = isolate_unknown(hostile);
        assert_eq!(wrapped, "\u{2068}abc\u{2069}");
        assert_eq!(
            wrapped.chars().filter(|c| is_isolate_control(*c)).count(),
            2
        );
    }

    #[test]
    fn empty_text_is_still_balanced() {
        assert_eq!(isolate_ltr(""), "\u{2066}\u{2069}");
    }
}
