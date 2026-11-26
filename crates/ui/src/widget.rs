//! Widgets that follow the reading direction.
//!
//! Screens are written once, in logical order (start to end), and these
//! helpers place things on the correct physical side. Use them instead of
//! the toolkit's own left and right.
//!
//! Text alignment is done by placing a text that is only as wide as its
//! content inside a container. The toolkit's own text alignment clips or
//! hides right-to-left text, as found in the first rendering test.

use iced_core::{Alignment, Length, Padding, alignment::Horizontal, widget::text::Wrapping};
use iced_widget::{Container, Row, Text, container, text};

use crate::{Direction, Edge, Insets, Side, row_order, runtime::Element};

/// A row whose children are given in reading order.
pub fn row<'a, M: 'a>(
    direction: Direction,
    children: Vec<Element<'a, M>>,
) -> Row<'a, M, crate::Theme, crate::Renderer> {
    Row::with_children(row_order(direction, children)).align_y(Alignment::Center)
}

/// Padding given as start and end instead of left and right.
pub fn padding(direction: Direction, insets: Insets) -> Padding {
    let physical = insets.resolve(direction);
    Padding {
        top: physical.top,
        right: physical.right,
        bottom: physical.bottom,
        left: physical.left,
    }
}

/// The toolkit's horizontal alignment for a logical edge.
pub fn horizontal(direction: Direction, edge: Edge) -> Horizontal {
    match edge.side(direction) {
        Side::Left => Horizontal::Left,
        Side::Right => Horizontal::Right,
    }
}

/// A piece of text that wraps, even inside a long word.
pub fn label<'a>(content: impl text::IntoFragment<'a>) -> Text<'a, crate::Theme, crate::Renderer> {
    text(content).wrapping(Wrapping::WordOrGlyph)
}

/// Fills the width and puts the content at the start or the end.
pub fn align<'a, M: 'a>(
    direction: Direction,
    edge: Edge,
    content: impl Into<Element<'a, M>>,
) -> Container<'a, M, crate::Theme, crate::Renderer> {
    container(content)
        .width(Length::Fill)
        .align_x(horizontal(direction, edge))
}

/// Fills the width and centres the content.
pub fn centered<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
) -> Container<'a, M, crate::Theme, crate::Renderer> {
    container(content).center_x(Length::Fill)
}
