// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::{Element, Renderer};
use iced::widget::scrollable::{Direction, Scrollable, Scrollbar};

pub fn scrollable<'a, Message>(
    element: impl Into<Element<'a, Message>>,
) -> Scrollable<'a, Message, crate::Theme, Renderer> {
    vertical(element)
}

pub fn vertical<'a, Message>(
    element: impl Into<Element<'a, Message>>,
) -> Scrollable<'a, Message, crate::Theme, Renderer> {
    // WMDE: the padding goes on the scrollbar itself. `Scrollable::padding` writes into the
    // scrollbar of the current direction and `direction` then replaces that scrollbar whole,
    // so called in that order the 8 px gap the bar had in 1.8 was dropped in every
    // `cosmic::widget::scrollable` of the stack and the bar ran flush with the edge.
    iced::widget::scrollable(element).direction(Direction::Vertical(
        Scrollbar::new().width(8.0).scroller_width(8.0).padding(8.0),
    ))
}

pub fn horizontal<'a, Message>(
    element: impl Into<Element<'a, Message>>,
) -> Scrollable<'a, Message, crate::Theme, Renderer> {
    iced::widget::scrollable(element).direction(Direction::Horizontal(
        Scrollbar::new().width(8.0).scroller_width(8.0),
    ))
}
