//! Layout direction helpers.
//!
//! Screens are described in logical terms, "start" and "end", never "left"
//! and "right". These helpers turn the logical description into a physical
//! one for the current [`Direction`], so every screen mirrors for
//! right-to-left languages without per-screen work.
//!
//! This crate knows nothing about a UI toolkit. It only holds the rules,
//! which keeps them easy to test.

pub use i18n::Direction;

/// A physical horizontal side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    /// The left side of the screen.
    Left,
    /// The right side of the screen.
    Right,
}

/// A logical horizontal position, relative to the reading direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Edge {
    /// Where a line of text starts: left in English, right in Hebrew.
    Start,
    /// Where a line of text ends: right in English, left in Hebrew.
    End,
}

impl Edge {
    /// The physical side this edge is on for `direction`.
    #[must_use]
    pub const fn side(self, direction: Direction) -> Side {
        match (self, direction) {
            (Self::Start, Direction::Ltr) | (Self::End, Direction::Rtl) => Side::Left,
            (Self::Start, Direction::Rtl) | (Self::End, Direction::Ltr) => Side::Right,
        }
    }
}

/// Space around a box in logical terms.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Insets {
    /// Space above.
    pub top: f32,
    /// Space on the side where text ends.
    pub end: f32,
    /// Space below.
    pub bottom: f32,
    /// Space on the side where text starts.
    pub start: f32,
}

/// Space around a box in physical terms.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PhysicalInsets {
    /// Space above.
    pub top: f32,
    /// Space on the right.
    pub right: f32,
    /// Space below.
    pub bottom: f32,
    /// Space on the left.
    pub left: f32,
}

impl Insets {
    /// The same space on all four sides.
    #[must_use]
    pub const fn all(value: f32) -> Self {
        Self {
            top: value,
            end: value,
            bottom: value,
            start: value,
        }
    }

    /// Resolves start and end to left and right for `direction`.
    #[must_use]
    pub const fn resolve(self, direction: Direction) -> PhysicalInsets {
        let (left, right) = match direction {
            Direction::Ltr => (self.start, self.end),
            Direction::Rtl => (self.end, self.start),
        };
        PhysicalInsets {
            top: self.top,
            right,
            bottom: self.bottom,
            left,
        }
    }
}

/// Puts the children of a row, given from start to end, into the
/// left-to-right order a toolkit draws them in.
#[must_use]
pub fn row_order<T>(direction: Direction, mut children: Vec<T>) -> Vec<T> {
    if direction == Direction::Rtl {
        children.reverse();
    }
    children
}

/// The physical column, counted from the left, of the cell at logical
/// `index` in a row of `columns` cells that fills from the start side.
///
/// Returns `None` if `index` is not inside the row.
#[must_use]
pub const fn physical_column(direction: Direction, index: usize, columns: usize) -> Option<usize> {
    if index >= columns {
        return None;
    }
    Some(match direction {
        Direction::Ltr => index,
        Direction::Rtl => columns - 1 - index,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_is_left_in_ltr_and_right_in_rtl() {
        assert_eq!(Edge::Start.side(Direction::Ltr), Side::Left);
        assert_eq!(Edge::Start.side(Direction::Rtl), Side::Right);
        assert_eq!(Edge::End.side(Direction::Ltr), Side::Right);
        assert_eq!(Edge::End.side(Direction::Rtl), Side::Left);
    }

    #[test]
    fn insets_swap_sides_in_rtl() {
        let insets = Insets {
            top: 1.0,
            end: 2.0,
            bottom: 3.0,
            start: 4.0,
        };
        let ltr = insets.resolve(Direction::Ltr);
        assert_eq!((ltr.left, ltr.right), (4.0, 2.0));
        let rtl = insets.resolve(Direction::Rtl);
        assert_eq!((rtl.left, rtl.right), (2.0, 4.0));
        // Vertical space never changes.
        assert_eq!((ltr.top, ltr.bottom), (rtl.top, rtl.bottom));
    }

    #[test]
    fn mirroring_twice_is_the_identity() {
        let insets = Insets {
            top: 1.0,
            end: 2.0,
            bottom: 3.0,
            start: 4.0,
        };
        let rtl = insets.resolve(Direction::Rtl);
        let back = Insets {
            top: rtl.top,
            end: rtl.right,
            bottom: rtl.bottom,
            start: rtl.left,
        }
        .resolve(Direction::Rtl);
        let ltr = insets.resolve(Direction::Ltr);
        assert_eq!(back, ltr);
    }

    #[test]
    fn uniform_insets_do_not_depend_on_direction() {
        let insets = Insets::all(8.0);
        assert_eq!(
            insets.resolve(Direction::Ltr),
            insets.resolve(Direction::Rtl)
        );
    }

    #[test]
    fn row_order_reverses_only_in_rtl() {
        let row = vec!["clock", "spacer", "battery"];
        assert_eq!(row_order(Direction::Ltr, row.clone()), row);
        assert_eq!(
            row_order(Direction::Rtl, row),
            vec!["battery", "spacer", "clock"]
        );
    }

    #[test]
    fn the_first_app_sits_at_the_start_side() {
        // A four-column app grid: the first app is leftmost in English and
        // rightmost in Hebrew.
        assert_eq!(physical_column(Direction::Ltr, 0, 4), Some(0));
        assert_eq!(physical_column(Direction::Rtl, 0, 4), Some(3));
        assert_eq!(physical_column(Direction::Rtl, 3, 4), Some(0));
    }

    #[test]
    fn physical_column_rejects_out_of_range() {
        assert_eq!(physical_column(Direction::Ltr, 4, 4), None);
        assert_eq!(physical_column(Direction::Rtl, 0, 0), None);
    }
}
