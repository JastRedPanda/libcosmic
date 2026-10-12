// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::cosmic_theme::{Density, Spacing};
use crate::{Element, theme, widget};
use apply::Apply;
use derive_setters::Setters;
use iced_core::widget::tree;
use iced_core::{Length, Size, Vector, Widget, layout, text};
use std::borrow::Cow;

/// WMDE: the fill of the strip under a title bar, where menu bars and search fields live.
///
/// Also what the active tab of a tabs title bar is colored with, so the tab reads as part
/// of the strip below it. The fill is snapped: an unsnapped strip ends on a fractional row,
/// and the sliver between it and a snapped neighbour belongs to neither - with nothing
/// opaque behind the body, that sliver shows the desktop as a hairline.
#[must_use]
pub fn strip_class() -> theme::Container<'static> {
    theme::Container::custom(|theme| widget::container::Style {
        background: Some(iced::Background::Color(strip_color(theme))),
        snap: true,
        ..Default::default()
    })
}

/// WMDE: the color [`strip_class`] fills with.
#[must_use]
pub fn strip_color(theme: &crate::Theme) -> iced::Color {
    theme.cosmic().primary(theme.transparent).base.into()
}

#[must_use]
pub fn header_bar<'a, Message>() -> HeaderBar<'a, Message> {
    HeaderBar {
        title: Cow::Borrowed(""),
        on_close: None,
        on_drag: None,
        on_maximize: None,
        on_minimize: None,
        on_right_click: None,
        start: Vec::new(),
        center: Vec::new(),
        end: Vec::new(),
        density: None,
        focused: false,
        maximized: false,
        sharp_corners: false,
        is_ssd: false,
        on_double_click: None,
    }
}

#[derive(Setters)]
pub struct HeaderBar<'a, Message> {
    /// Defines the title of the window
    #[setters(skip)]
    title: Cow<'a, str>,

    /// A message emitted when the close button is pressed.
    #[setters(strip_option)]
    on_close: Option<Message>,

    /// A message emitted when dragged.
    #[setters(strip_option)]
    on_drag: Option<Message>,

    /// A message emitted when the maximize button is pressed.
    #[setters(strip_option)]
    on_maximize: Option<Message>,

    /// A message emitted when the minimize button is pressed.
    #[setters(strip_option)]
    on_minimize: Option<Message>,

    /// A message emitted when the header is double clicked,
    /// usually used to maximize the window.
    #[setters(strip_option)]
    on_double_click: Option<Message>,

    /// A message emitted when the header is right clicked.
    #[setters(strip_option)]
    on_right_click: Option<Message>,

    /// Elements packed at the start of the headerbar.
    #[setters(skip)]
    start: Vec<Element<'a, Message>>,

    /// Elements packed in the center of the headerbar.
    #[setters(skip)]
    center: Vec<Element<'a, Message>>,

    /// Elements packed at the end of the headerbar.
    #[setters(skip)]
    end: Vec<Element<'a, Message>>,

    /// Controls the density of the headerbar.
    #[setters(strip_option)]
    density: Option<Density>,

    /// Focused state of the window
    focused: bool,

    /// Maximized state of the window
    maximized: bool,

    /// Whether the corners of the window should be sharp
    sharp_corners: bool,

    /// HeaderBar used for server-side decorations
    is_ssd: bool,
}

impl<'a, Message: Clone + 'static> HeaderBar<'a, Message> {
    /// Defines the title of the window
    #[must_use]
    pub fn title(mut self, title: impl Into<Cow<'a, str>> + 'a) -> Self {
        self.title = title.into();
        self
    }

    /// Pushes an element to the start region.
    #[must_use]
    pub fn start(mut self, widget: impl Into<Element<'a, Message>> + 'a) -> Self {
        self.start.push(widget.into());
        self
    }

    /// Pushes an element to the center region.
    #[must_use]
    pub fn center(mut self, widget: impl Into<Element<'a, Message>> + 'a) -> Self {
        self.center.push(widget.into());
        self
    }

    /// Pushes an element to the end region.
    #[must_use]
    pub fn end(mut self, widget: impl Into<Element<'a, Message>> + 'a) -> Self {
        self.end.push(widget.into());
        self
    }
}

pub struct HeaderBarWidget<'a, Message> {
    start: Element<'a, Message>,
    center: Option<Element<'a, Message>>,
    end: Element<'a, Message>,
    /// WMDE: the window controls are a region of their own, so they can sit flush
    /// in the corner while everything else stays inside `padding`.
    controls: Element<'a, Message>,
    /// WMDE: header padding, applied here instead of by the wrapping container.
    padding: [u16; 4],
}

impl<'a, Message> HeaderBarWidget<'a, Message> {
    pub fn new(
        start: Element<'a, Message>,
        center: Option<Element<'a, Message>>,
        end: Element<'a, Message>,
        controls: Element<'a, Message>,
        padding: [u16; 4],
    ) -> Self {
        Self {
            start,
            center,
            end,
            controls,
            padding,
        }
    }

    fn elems(&self) -> impl Iterator<Item = &Element<'a, Message>> {
        std::iter::once(&self.start)
            .chain(std::iter::once(&self.end))
            .chain(std::iter::once(&self.controls))
            .chain(self.center.as_ref())
    }

    fn elems_mut(&mut self) -> impl Iterator<Item = &mut Element<'a, Message>> {
        std::iter::once(&mut self.start)
            .chain(std::iter::once(&mut self.end))
            .chain(std::iter::once(&mut self.controls))
            .chain(self.center.as_mut())
    }
}

impl<'a, Message: Clone + 'static> Widget<Message, crate::Theme, crate::Renderer>
    for HeaderBarWidget<'a, Message>
{
    fn diff(&mut self, tree: &mut tree::Tree) {
        if let Some(center) = &mut self.center {
            tree.diff_children(&mut [&mut self.start, &mut self.end, &mut self.controls, center]);
        } else {
            tree.diff_children(&mut [&mut self.start, &mut self.end, &mut self.controls]);
        }
    }

    fn children(&self) -> Vec<tree::Tree> {
        self.elems().map(tree::Tree::new).collect()
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Shrink,
        }
    }

    fn layout(
        &mut self,
        tree: &mut tree::Tree,
        renderer: &crate::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let width = limits.max().width;
        let height = limits.max().height;
        let gap = 8.0;

        let [pad_top, pad_right, pad_bottom, pad_left] = self.padding.map(f32::from);

        // WMDE: the window controls sit flush in the top-right corner of the bar,
        // Windows-style, so they ignore the header padding and are not centred
        // vertically like every other region - they are shorter than the bar (see
        // `window_controls`) and the gap belongs below them. Everything else is laid
        // out inside the padding, to the left of them.
        let controls_node = self.controls.as_widget_mut().layout(
            &mut tree.children[2],
            renderer,
            &layout::Limits::new(Size::ZERO, Size::new(width, height)),
        );
        let controls_width = controls_node.size().width;

        let inner_height = (height - pad_top - pad_bottom).max(0.0);
        let inner_right = (width - controls_width - pad_right).max(pad_left);

        let end_node = self.end.as_widget_mut().layout(
            &mut tree.children[1],
            renderer,
            &layout::Limits::new(Size::ZERO, Size::new(inner_right - pad_left, inner_height)),
        );
        let end_width = end_node.size().width;

        let start_available = (inner_right - end_width - gap - pad_left).max(0.0);
        let start_node = self.start.as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &layout::Limits::new(Size::ZERO, Size::new(start_available, inner_height)),
        );
        let start_width = start_node.size().width;

        let vcenter = |node: layout::Node, x: f32| -> layout::Node {
            let dy = pad_top + ((inner_height - node.size().height) / 2.0).max(0.0);
            node.translate(Vector::new(x, dy))
        };

        let mut child_nodes = Vec::with_capacity(4);
        child_nodes.push(vcenter(start_node, pad_left));
        child_nodes.push(vcenter(end_node, inner_right - end_width));
        child_nodes.push(controls_node.translate(Vector::new(width - controls_width, 0.0)));

        if let Some(center) = &mut self.center {
            let slot_start = pad_left + start_width + gap;
            let slot_end = (inner_right - end_width - gap).max(slot_start);
            let slot_width = slot_end - slot_start;
            // this instead of `node.size().width` prevents center jitter as text ellipsizes
            let natural_width = center
                .as_widget_mut()
                .layout(
                    &mut tree.children[3],
                    renderer,
                    &layout::Limits::new(Size::ZERO, Size::new(width, inner_height)),
                )
                .size()
                .width;

            let node = center.as_widget_mut().layout(
                &mut tree.children[3],
                renderer,
                &layout::Limits::new(Size::ZERO, Size::new(slot_width, inner_height)),
            );

            // Centered on the window, not on the free slot, unless that would run
            // into the regions on either side.
            let ideal_x = (width - natural_width) / 2.0;
            let max_x = (slot_end - natural_width).max(slot_start);
            let center_x = ideal_x.clamp(slot_start, max_x);

            // WMDE: the title lines up with the caption glyphs, which sit centred in
            // the top-aligned controls, not with the middle of the whole bar - that
            // left it 9px below the glyphs. It then goes 2px further down, which is
            // where it reads as level with them (user's call). A bar without controls,
            // or a centre taller than them, keeps the old centring.
            const TITLE_DROP: f32 = 2.0;
            let controls_height = child_nodes[2].size().height;
            let node = if node.size().height > 0.0 && node.size().height <= controls_height {
                let dy = (controls_height - node.size().height) / 2.0 + TITLE_DROP;
                node.translate(Vector::new(center_x, dy))
            } else {
                vcenter(node, center_x)
            };

            child_nodes.push(node)
        }

        layout::Node::with_children(Size::new(width, height), child_nodes)
    }

    fn draw(
        &self,
        tree: &tree::Tree,
        renderer: &mut crate::Renderer,
        theme: &crate::Theme,
        style: &iced_core::renderer::Style,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        viewport: &iced_core::Rectangle,
    ) {
        self.elems()
            .zip(&tree.children)
            .zip(layout.children())
            .for_each(|((e, s), l)| {
                e.as_widget()
                    .draw(s, renderer, theme, style, l, cursor, viewport);
            });
    }

    fn update(
        &mut self,
        state: &mut tree::Tree,
        event: &iced_core::Event,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        renderer: &crate::Renderer,
        clipboard: &mut dyn iced_core::Clipboard,
        shell: &mut iced_core::Shell<'_, Message>,
        viewport: &iced_core::Rectangle,
    ) {
        self.elems_mut()
            .zip(&mut state.children)
            .zip(layout.children())
            .for_each(|((e, s), l)| {
                e.as_widget_mut()
                    .update(s, event, l, cursor, renderer, clipboard, shell, viewport);
            });
    }

    fn mouse_interaction(
        &self,
        state: &tree::Tree,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        viewport: &iced_core::Rectangle,
        renderer: &crate::Renderer,
    ) -> iced_core::mouse::Interaction {
        self.elems()
            .zip(&state.children)
            .zip(layout.children())
            .map(|((e, s), l)| {
                e.as_widget()
                    .mouse_interaction(s, l, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or(iced_core::mouse::Interaction::None)
    }

    fn operate(
        &mut self,
        state: &mut tree::Tree,
        layout: iced_core::Layout<'_>,
        renderer: &crate::Renderer,
        operation: &mut dyn iced_core::widget::Operation<()>,
    ) {
        self.elems_mut()
            .zip(&mut state.children)
            .zip(layout.children())
            .for_each(|((e, s), l)| {
                e.as_widget_mut().operate(s, l, renderer, operation);
            });
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut tree::Tree,
        layout: iced_core::Layout<'b>,
        renderer: &crate::Renderer,
        viewport: &iced_core::Rectangle,
        translation: Vector,
    ) -> Option<iced_core::overlay::Element<'b, Message, crate::Theme, crate::Renderer>> {
        self.elems_mut()
            .zip(&mut state.children)
            .zip(layout.children())
            .find_map(|((e, s), l)| {
                e.as_widget_mut()
                    .overlay(s, l, renderer, viewport, translation)
            })
    }

    fn drag_destinations(
        &self,
        state: &tree::Tree,
        layout: iced_core::Layout<'_>,
        renderer: &crate::Renderer,
        dnd_rectangles: &mut iced_core::clipboard::DndDestinationRectangles,
    ) {
        self.elems()
            .zip(&state.children)
            .zip(layout.children())
            .for_each(|((e, s), l)| {
                e.as_widget()
                    .drag_destinations(s, l, renderer, dnd_rectangles);
            });
    }

    #[cfg(feature = "a11y")]
    /// get the a11y nodes for the widget
    fn a11y_nodes(
        &self,
        layout: iced_core::Layout<'_>,
        state: &tree::Tree,
        p: iced::mouse::Cursor,
    ) -> iced_accessibility::A11yTree {
        iced_accessibility::A11yTree::join(
            self.elems()
                .zip(&state.children)
                .zip(layout.children())
                .map(|((e, s), l)| e.as_widget().a11y_nodes(l, s, p)),
        )
    }
}

impl<'a, Message: Clone + 'static> From<HeaderBarWidget<'a, Message>> for Element<'a, Message> {
    fn from(w: HeaderBarWidget<'a, Message>) -> Self {
        Element::new(w)
    }
}

impl<'a, Message: Clone + 'static> HeaderBar<'a, Message> {
    /// Converts the headerbar builder into an Iced element.
    pub fn view(mut self) -> Element<'a, Message> {
        let Spacing {
            space_xxxs,
            space_xxs,
            ..
        } = theme::spacing();
        let is_ssd = self.is_ssd;

        // Take ownership of the regions to be packed.
        let start = std::mem::take(&mut self.start);
        let center = std::mem::take(&mut self.center);
        let end = std::mem::take(&mut self.end);

        let padding = if is_ssd {
            [2, 8, 2, 8]
        } else {
            match (
                self.density.unwrap_or_else(crate::config::header_size),
                self.maximized, // window border handling
            ) {
                (Density::Compact, true) => [4, 8, 4, 8],
                (Density::Compact, false) => [3, 7, 4, 7],
                (_, true) => [8, 8, 8, 8],
                (_, false) => [7, 7, 8, 7],
            }
        };

        // WMDE: let the active theme override the header padding (e.g. drop the
        // bottom gap so title-bar tabs sit flush on the strip below). Height is
        // recomputed from padding, so it tracks the override automatically.
        // `[0, 0, 0, 0]` (the default) means "unset" -> keep the toolkit padding.
        // Server-side decorations are exempt: the compositor sizes that title bar
        // to a fixed height of its own, and a taller header would be clipped.
        let hp = theme::header_padding();
        let padding = if is_ssd || hp == [0, 0, 0, 0] {
            padding
        } else {
            hp
        };

        let height = 32.0 + f32::from(padding[0]) + f32::from(padding[2]);
        let controls = self.window_controls(height);

        let start = widget::row::with_children(start)
            .spacing(space_xxxs)
            .align_y(iced::Alignment::Center)
            .into();
        let center = if !center.is_empty() {
            Some(
                widget::row::with_children(center)
                    .spacing(space_xxxs)
                    .align_y(iced::Alignment::Center)
                    .into(),
            )
        } else if !self.title.is_empty() {
            Some(
                widget::text::heading(self.title)
                    .wrapping(text::Wrapping::None)
                    .ellipsize(text::Ellipsize::End(text::EllipsizeHeightLimit::Lines(1)))
                    .into(),
            )
        } else {
            None
        };
        let end = widget::row::with_children(end)
            .spacing(space_xxs)
            .align_y(iced::Alignment::Center)
            .into();

        let mut widget = HeaderBarWidget::new(start, center, end, controls, padding)
            .apply(widget::container)
            .class(theme::Container::HeaderBar {
                focused: self.focused,
                sharp_corners: self.sharp_corners,
                transparent: if is_ssd { false } else { true },
            })
            .height(Length::Fixed(height))
            .apply(widget::mouse_area);

        if let Some(message) = self.on_drag {
            widget = widget.on_drag(message);
        }
        if let Some(message) = self.on_maximize {
            widget = widget.on_release(message);
        }
        if let Some(message) = self.on_double_click {
            widget = widget.on_double_press(message);
        }
        if let Some(message) = self.on_right_click {
            widget = widget.on_right_press(message);
        }

        widget.into()
    }

    /// Creates the widget for window controls.
    ///
    /// WMDE: Windows-style caption buttons - a 46x28 rectangle each, flush against
    /// the top right corner of the bar and packed edge to edge with no gap. They do
    /// NOT run the full height of the bar: measured off `ref/w11/Maximize.png` and
    /// `Close.png`, the hover fill is 42 device pixels tall at 150%, exactly 28
    /// logical, with a flat bottom edge and the same height in every column, inside
    /// a title bar of 37. `height` is that bar height, and only caps the button so a
    /// theme with a smaller `header_padding` cannot make it overflow.
    ///
    /// Nothing here aligns the buttons to the top - `HeaderBarWidget::layout` places
    /// the controls node at `y = 0` without vertical centring, unlike every other
    /// region of the bar.
    fn window_controls(&mut self, height: f32) -> Element<'a, Message> {
        /// Width of one caption button, as in Windows.
        const WIDTH: f32 = 46.0;
        /// Height of the button, which is also the height of its hover fill.
        const HEIGHT: f32 = 28.0;
        /// Glyphs are drawn 10px inside a 16px box, again as in Windows.
        const ICON_SIZE: u16 = 16;

        // The glyph centres inside the button, so this is what puts it 14 from the top
        // of the bar. Windows measures 13.3; centring on the full bar would put it at
        // 19.5, which is where it used to sit.
        let height = height.min(HEIGHT);

        let mut controls: Vec<(&'static str, bool, Message)> = Vec::with_capacity(3);

        if let Some(message) = self.on_minimize.take() {
            controls.push(("wmde-window-minimize-symbolic", false, message));
        }

        if let Some(message) = self.on_maximize.take() {
            let name = if self.maximized {
                "wmde-window-restore-symbolic"
            } else {
                "wmde-window-maximize-symbolic"
            };

            controls.push((name, false, message));
        }

        if let Some(message) = self.on_close.take() {
            controls.push(("wmde-window-close-symbolic", true, message));
        }

        // Only the rightmost button touches the window corner, and only a window
        // with rounded corners needs its fill rounded to match.
        let last = controls.len().wrapping_sub(1);
        let rounded = !self.sharp_corners;
        let focused = self.focused;

        let controls: Vec<Element<'a, Message>> = controls
            .into_iter()
            .enumerate()
            .map(|(i, (name, close, message))| {
                widget::icon::from_name(name)
                    .apply(widget::button::icon)
                    .icon_size(ICON_SIZE)
                    .width(Length::Fixed(WIDTH))
                    .height(Length::Fixed(height))
                    .padding([0.0, (WIDTH - f32::from(ICON_SIZE)) / 2.0])
                    .class(theme::Button::WindowControl {
                        close,
                        round_corner: rounded && i == last,
                    })
                    .selected(focused)
                    .on_press(message)
                    .into()
            })
            .collect();

        widget::row::with_children(controls)
            .align_y(iced::Alignment::Center)
            .into()
    }
}

impl<'a, Message: Clone + 'static> From<HeaderBar<'a, Message>> for Element<'a, Message> {
    fn from(headerbar: HeaderBar<'a, Message>) -> Self {
        headerbar.view()
    }
}
