// Copyright 2025 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Standard shapes for the secondary windows an application opens.
//!
//! A preset answers the two questions every extra toplevel has to answer the same way:
//! what [`window::Settings`] it opens with, and what frame it draws around its content.
//! Hand rolling either is how windows drift apart, and how a fixed size dialog ends up
//! offering to maximize itself.
//!
//! ```ignore
//! // In update(), when opening the window:
//! let preset = WindowPreset::dialog(Size::new(480.0, 600.0));
//! let (id, task) = window::open(preset.settings(Self::APP_ID));
//!
//! // In view_window(), when drawing it:
//! preset.view(
//!     self.core(),
//!     id,
//!     Chrome::dialog(fl!("preview"), Message::WindowClose(id), Message::WindowDrag(id)),
//!     content,
//! )
//! ```

use crate::{Apply, Core, Element};
use iced::{Length, Size, window};
use std::borrow::Cow;

/// The shape of a secondary application window.
///
/// Every preset uses client side decorations. Server side ones would hand the chrome to
/// the compositor, which decorates every window alike: it has no way to know a window is
/// not resizable, and xdg-decoration gives a client no way to ask for fewer buttons.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WindowPreset {
    /// A fixed size window that cannot be resized, for dialogs such as About or a
    /// properties sheet.
    ///
    /// Equal minimum and maximum sizes are also what COSMIC compositors read as "this is
    /// a dialog", so the window floats instead of joining the tiling layout.
    Dialog {
        /// Both the initial and the only size of the window.
        size: Size,
    },

    /// A resizable companion window, for panes an application splits off from its main
    /// window, such as a preview or a view options sheet.
    ///
    /// A resizable window tiles like any other unless its app id is listed in the
    /// compositor's tiling exceptions; the preset cannot arrange that on its own.
    Utility {
        /// Size the window opens at.
        size: Size,
        /// Smallest size the user can shrink it to.
        min_size: Size,
    },
}

impl WindowPreset {
    /// A fixed size dialog window.
    #[must_use]
    pub const fn dialog(size: Size) -> Self {
        Self::Dialog { size }
    }

    /// A resizable companion window.
    #[must_use]
    pub const fn utility(size: Size, min_size: Size) -> Self {
        Self::Utility { size, min_size }
    }

    /// Whether the user can resize a window of this shape.
    #[must_use]
    pub const fn is_resizable(&self) -> bool {
        matches!(self, Self::Utility { .. })
    }

    /// Settings to open this window with, for an application with `app_id`.
    ///
    /// `app_id` is not optional in practice: window settings default to an empty
    /// application id on Linux, and a window without one gets no taskbar icon and matches
    /// no desktop entry.
    #[must_use]
    #[cfg_attr(not(target_os = "linux"), allow(unused_variables))]
    pub fn settings(&self, app_id: &str) -> window::Settings {
        let (size, min_size, max_size) = match *self {
            Self::Dialog { size } => (size, Some(size), Some(size)),
            Self::Utility { size, min_size } => (size, Some(min_size), None),
        };

        #[allow(unused_mut)]
        let mut settings = window::Settings {
            size,
            min_size,
            max_size,
            resizable: self.is_resizable(),
            // Load bearing, and not implied by `resizable`. The client side drag-resize
            // handler decides a window is resizable from `!is_decorated()` alone and
            // never consults `is_resizable()`, so a non-resizable window keeps showing
            // resize cursors within this border unless it is zero.
            resize_border: if self.is_resizable() { 8 } else { 0 },
            decorations: false,
            transparent: true,
            ..Default::default()
        };

        #[cfg(target_os = "linux")]
        {
            settings.platform_specific.application_id = app_id.to_string();
        }

        settings
    }

    /// Draws `content` inside the standard window frame for this shape: a header bar on
    /// top, and the same 1px border and corner radius the main window template uses.
    ///
    /// The header is the one the [`Chrome`] asks for, not one derived from the shape: a
    /// resizable window may still carry the dialog title bar.
    pub fn view<'a, Message: Clone + 'static>(
        &self,
        core: &Core,
        id: window::Id,
        chrome: Chrome<'a, Message>,
        content: impl Into<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        crate::widget::column::with_capacity(2)
            .push(chrome.header(core, id))
            .push(content.into())
            .apply(crate::widget::container)
            .padding(1)
            .width(Length::Fill)
            .height(Length::Fill)
            .class(frame_class())
            .into()
    }
}

/// The frame a client-decorated secondary window draws around its header and content: the
/// window background, and the 1px border and corner radius the compositor would otherwise
/// have drawn. Pair it with `padding(1)`.
///
/// [`WindowPreset::view`] uses it; a window that sizes itself to its content (and so cannot
/// fill a preset's fixed size) builds the same frame from this and [`Chrome::header`].
#[must_use]
pub fn frame_class() -> crate::theme::Container<'static> {
    let window_corner_radius = crate::theme::active()
        .cosmic()
        .radius_s()
        .map(|x| if x < 4.0 { x } else { x + 4.0 });

    crate::theme::Container::custom(move |theme| crate::widget::container::Style {
        background: Some(iced::Background::Color(
            theme.cosmic().background(theme.transparent).base.into(),
        )),
        border: iced::Border {
            color: theme.cosmic().bg_divider().into(),
            width: 1.0,
            radius: window_corner_radius.into(),
        },
        ..Default::default()
    })
}

/// Messages a framed window's header bar emits, and which title bar it is.
///
/// A secondary window carries one of two title bars: [`TitleBar::Dialog`] (title and
/// close) or [`TitleBar::Normal`] (title, minimize, maximize and close). Tabs belong to
/// main windows only.
///
/// The messages have to come from the application because the toolkit's own chrome
/// actions are hardwired to the main window: reusing them would drag or close the wrong
/// window. Handle them with `window::close(id)`, `core.drag(Some(id))`,
/// `core.minimize(Some(id))` and `core.toggle_maximize(Some(id))`.
///
/// [`TitleBar::Dialog`]: crate::core::TitleBar::Dialog
/// [`TitleBar::Normal`]: crate::core::TitleBar::Normal
pub struct Chrome<'a, Message> {
    title: Cow<'a, str>,
    on_close: Message,
    on_drag: Message,
    /// Minimize and maximize messages; present only on the normal title bar.
    normal: Option<(Message, Message)>,
}

impl<'a, Message> Chrome<'a, Message> {
    /// The dialog title bar: `title` and a close button.
    pub fn dialog(title: impl Into<Cow<'a, str>>, on_close: Message, on_drag: Message) -> Self {
        Self {
            title: title.into(),
            on_close,
            on_drag,
            normal: None,
        }
    }

    /// The normal title bar: `title`, minimize, maximize and close. A double click on the
    /// bar sends `on_maximize` too.
    pub fn normal(
        title: impl Into<Cow<'a, str>>,
        on_close: Message,
        on_drag: Message,
        on_minimize: Message,
        on_maximize: Message,
    ) -> Self {
        Self {
            title: title.into(),
            on_close,
            on_drag,
            normal: Some((on_minimize, on_maximize)),
        }
    }

    /// The title bar of window `id`.
    pub fn header(self, core: &Core, id: window::Id) -> Element<'a, Message>
    where
        Message: Clone + 'static,
    {
        let mut header = crate::widget::header_bar()
            .title(self.title)
            .focused(core.focus_chain().contains(&id))
            .on_close(self.on_close)
            .on_drag(self.on_drag);

        if let Some((on_minimize, on_maximize)) = self.normal {
            header = header
                .on_minimize(on_minimize)
                .on_maximize(on_maximize.clone())
                .on_double_click(on_maximize);
        }

        header.into()
    }
}
