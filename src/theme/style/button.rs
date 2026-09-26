// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Contains stylesheet implementation for [`crate::widget::button`].

use cosmic_theme::Component;
use iced_core::{Background, Color};

use crate::theme::TRANSPARENT_COMPONENT;
use crate::widget::button::{Catalog, Style};

#[derive(Default)]
pub enum Button {
    AppletIcon,
    AppletMenu,
    Custom {
        active: Box<dyn Fn(bool, &crate::Theme) -> Style>,
        disabled: Box<dyn Fn(&crate::Theme) -> Style>,
        hovered: Box<dyn Fn(bool, &crate::Theme) -> Style>,
        pressed: Box<dyn Fn(bool, &crate::Theme) -> Style>,
    },
    Destructive,
    HeaderBar,
    Icon,
    IconVertical,
    Image,
    Link,
    LinkActive,
    ListItem([f32; 4]),
    MenuFolder,
    MenuItem,
    MenuRoot,
    NavToggle,
    #[default]
    Standard,
    Suggested,
    Text,
    Transparent,
    /// WMDE: Windows-style caption button in the header bar.
    ///
    /// Window focus is passed in through the button's `selected` flag, so an
    /// unfocused window dims its glyphs the way the title text already does.
    WindowControl {
        /// The close button takes the red Windows hover/pressed fill.
        close: bool,
        /// Set on the rightmost button of a window with rounded corners: its
        /// top-right corner follows the window radius so the fill does not poke
        /// out past the corner.
        round_corner: bool,
    },
}

/// WMDE: Windows 11 close-button fill (#C42B1C), measured off the reference
/// screenshots in `w11/`.
const CLOSE_FILL: Color = Color {
    r: 0.769,
    g: 0.169,
    b: 0.110,
    a: 1.0,
};

/// WMDE: builds the appearance of a caption button (see [`Button::WindowControl`]).
///
/// `fill` is the button background: `None` in the resting state, a translucent
/// wash on hover/press, or the red fill for close.
fn window_control(
    theme: &crate::Theme,
    close: bool,
    round_corner: bool,
    window_focused: bool,
    fill: Option<Color>,
) -> Style {
    use crate::ext::ColorExt;

    let cosmic = theme.cosmic();
    let container = cosmic.background(theme.transparent);

    // Same focused/unfocused split the header bar uses for its title text.
    let glyph = if fill.is_some() && close {
        Color::WHITE
    } else if window_focused {
        Color::from(container.on)
    } else {
        Color::from(container.component.on).blend_alpha(container.base.into(), 0.5)
    };

    // Ensures visually aligned radii for content and window corners
    let window_corner_radius = cosmic.radius_s().map(|x| if x < 4.0 { x } else { x + 4.0 });
    // The compositor paints the window frame over the outermost pixel of the window
    // geometry (a 1px IndicatorShader ring in cosmic-comp), so this fill sits one pixel
    // inside it. Concentric insets need the SMALLER radius: reusing the window's own
    // radius flattens the fill's arc relative to the frame, and the diagonal shows 2-3px
    // of frame where the straight edges show 1.
    const WINDOW_BORDER: f32 = 1.0;
    let top_right = if round_corner {
        (window_corner_radius[1] - WINDOW_BORDER).max(0.0)
    } else {
        0.0
    };

    Style {
        background: fill.map(Background::Color),
        border_radius: [0.0, top_right, 0.0, 0.0].into(),
        icon_color: Some(glyph),
        text_color: Some(glyph),
        ..Style::new()
    }
}

/// WMDE: the translucent wash a non-close caption button paints on hover/press.
/// Windows lightens on dark title bars and darkens on light ones.
fn window_control_wash(theme: &crate::Theme, alpha: f32) -> Color {
    let base = if theme.cosmic().is_dark {
        Color::WHITE
    } else {
        Color::BLACK
    };

    Color { a: alpha, ..base }
}

pub fn appearance(
    theme: &crate::Theme,
    focused: bool,
    selected: bool,
    disabled: bool,
    style: &Button,
    color: impl Fn(&Component) -> (Color, Option<Color>, Option<Color>),
) -> Style {
    let cosmic = theme.cosmic();
    let mut corner_radii = &cosmic.corner_radii.radius_xl;
    let mut appearance = Style::new();
    let hc = theme.theme_type.is_high_contrast();
    match style {
        Button::Standard
        | Button::Text
        | Button::Suggested
        | Button::Destructive
        | Button::Transparent => {
            let style_component = match style {
                Button::Standard => &cosmic.button,
                Button::Text => &cosmic.text_button,
                Button::Suggested => &cosmic.accent_button,
                Button::Destructive => &cosmic.destructive_button,
                Button::Transparent => &TRANSPARENT_COMPONENT,
                _ => return appearance,
            };

            let (background, text, icon) = color(style_component);
            appearance.background = Some(Background::Color(background));
            if !matches!(style, Button::Standard) {
                appearance.text_color = text;
                appearance.icon_color = icon;
            } else if hc {
                appearance.border_color = style_component.border.into();
                appearance.border_width = 1.;
            }
        }

        Button::Icon | Button::IconVertical | Button::HeaderBar | Button::NavToggle => {
            if matches!(style, Button::IconVertical) {
                corner_radii = &cosmic.corner_radii.radius_m;
                if selected {
                    appearance.overlay = Some(Background::Color(Color::from(
                        cosmic.icon_button.selected_state_color(),
                    )));
                }
            }
            if matches!(style, Button::NavToggle) {
                corner_radii = &cosmic.corner_radii.radius_s;
            }

            let (background, text, icon) = color(&cosmic.icon_button);
            appearance.background = Some(Background::Color(background));
            // Only override icon button colors when it is disabled
            appearance.icon_color = if disabled { icon } else { None };
            appearance.text_color = if disabled { text } else { None };
        }

        Button::Image => {
            appearance.background = None;
            appearance.text_color = Some(cosmic.accent_text_color().into());
            appearance.icon_color = Some(cosmic.accent.base.into());

            corner_radii = &cosmic.corner_radii.radius_s;
            appearance.border_radius = (*corner_radii).into();

            if focused || selected {
                appearance.border_width = 2.0;
                appearance.border_color = cosmic.accent.base.into();
            } else if hc {
                appearance.border_color = theme.current_container().component.divider.into();
                appearance.border_width = 1.;
            }

            return appearance;
        }

        Button::Link => {
            appearance.background = None;
            appearance.icon_color = Some(cosmic.accent_text_color().into());
            appearance.text_color = Some(cosmic.accent_text_color().into());
            corner_radii = &cosmic.corner_radii.radius_0;
        }

        Button::LinkActive => {
            appearance.background = Some(Background::Color(cosmic.text_button.hover.into()));
            appearance.icon_color = Some(cosmic.accent_text_color().into());
            appearance.text_color = Some(cosmic.accent_text_color().into());
            // WMDE: square like `Link` - this fill is a selection, and selections are square
            // (.doc/appearance/05-rules.md).
            corner_radii = &cosmic.corner_radii.radius_0;
        }

        Button::Custom { .. } => (),
        // WMDE: the per-state fills live in the `Catalog` impl below, which never
        // routes a caption button through here; this is the resting appearance.
        Button::WindowControl {
            close,
            round_corner,
        } => return window_control(theme, *close, *round_corner, selected, None),
        Button::AppletMenu => {
            let (background, _, _) = color(&cosmic.text_button);
            appearance.background = Some(Background::Color(background));

            appearance.icon_color = Some(cosmic.background(theme.transparent).on.into());
            appearance.text_color = Some(cosmic.background(theme.transparent).on.into());
            corner_radii = &cosmic.corner_radii.radius_0;
        }
        Button::AppletIcon => {
            let (background, _, _) = color(&cosmic.text_button);
            appearance.background = Some(Background::Color(background));

            appearance.icon_color = Some(cosmic.background(theme.transparent).on.into());
            appearance.text_color = Some(cosmic.background(theme.transparent).on.into());
        }
        Button::MenuFolder => {
            // Menu folders cannot be disabled, ignore customized icon and text color
            let component = &cosmic.background(theme.transparent).component;
            let (background, _, _) = color(component);
            appearance.background = Some(Background::Color(background));
            appearance.icon_color = Some(component.on.into());
            appearance.text_color = Some(component.on.into());
            corner_radii = &cosmic.corner_radii.radius_s;
        }
        Button::ListItem(radii) => {
            corner_radii = radii;
            let (background, text, icon) = color(&cosmic.list_button);

            if selected {
                appearance.background = Some(Background::Color(
                    cosmic.primary(theme.transparent).component.hover.into(),
                ));
                appearance.icon_color = Some(cosmic.accent.base.into());
                appearance.text_color = Some(cosmic.accent_text_color().into());
            } else {
                appearance.background = Some(Background::Color(background));
                appearance.icon_color = icon;
                appearance.text_color = text;
            }
        }
        Button::MenuItem => {
            let (background, text, icon) = color(&cosmic.background(theme.transparent).component);
            appearance.background = Some(Background::Color(background));
            appearance.icon_color = icon;
            appearance.text_color = text;
            corner_radii = &cosmic.corner_radii.radius_s;
        }
        Button::MenuRoot => {
            appearance.background = None;
            appearance.icon_color = None;
            appearance.text_color = None;
        }
    }

    appearance.border_radius = (*corner_radii).into();

    if focused {
        appearance.outline_width = 1.0;
        appearance.outline_color = cosmic.accent.base.into();
        appearance.border_width = 2.0;
        appearance.border_color = Color::TRANSPARENT;
    }

    appearance
}

impl Catalog for crate::Theme {
    type Class = Button;

    fn active(&self, focused: bool, selected: bool, style: &Self::Class) -> Style {
        if let Button::Custom { active, .. } = style {
            return active(focused, self);
        }

        if let Button::WindowControl {
            close,
            round_corner,
        } = style
        {
            return window_control(self, *close, *round_corner, selected, None);
        }

        appearance(self, focused, selected, false, style, move |component| {
            let text_color = if matches!(
                style,
                Button::Icon | Button::IconVertical | Button::HeaderBar
            ) && selected
            {
                Some(self.cosmic().accent_text_color().into())
            } else {
                Some(component.on.into())
            };

            (component.base.into(), text_color, text_color)
        })
    }

    fn disabled(&self, style: &Self::Class) -> Style {
        if let Button::Custom { disabled, .. } = style {
            return disabled(self);
        }

        appearance(self, false, false, true, style, |component| {
            let mut background = Color::from(component.base);
            if !matches!(
                style,
                Button::MenuFolder | Button::MenuItem | Button::MenuRoot
            ) {
                background.a *= 0.5;
            }
            (
                background,
                Some(component.on_disabled.into()),
                Some(component.on_disabled.into()),
            )
        })
    }

    fn drop_target(&self, style: &Self::Class) -> Style {
        self.active(false, false, style)
    }

    fn hovered(&self, focused: bool, selected: bool, style: &Self::Class) -> Style {
        if let Button::Custom { hovered, .. } = style {
            return hovered(focused, self);
        }

        if let Button::WindowControl {
            close,
            round_corner,
        } = style
        {
            let fill = if *close {
                CLOSE_FILL
            } else {
                window_control_wash(self, 0.06)
            };

            return window_control(self, *close, *round_corner, selected, Some(fill));
        }

        appearance(
            self,
            focused || matches!(style, Button::Image),
            selected,
            false,
            style,
            |component| {
                let text_color = if matches!(
                    style,
                    Button::Icon | Button::IconVertical | Button::HeaderBar
                ) && selected
                {
                    Some(self.cosmic().accent_text_color().into())
                } else {
                    Some(component.on.into())
                };

                (component.hover.into(), text_color, text_color)
            },
        )
    }

    fn pressed(&self, focused: bool, selected: bool, style: &Self::Class) -> Style {
        if let Button::Custom { pressed, .. } = style {
            return pressed(focused, self);
        }

        // WMDE deviation from Windows: the pressed fill is stronger than hover,
        // not weaker, so the click reads as a click.
        if let Button::WindowControl {
            close,
            round_corner,
        } = style
        {
            let fill = if *close {
                Color {
                    r: CLOSE_FILL.r * 0.82,
                    g: CLOSE_FILL.g * 0.82,
                    b: CLOSE_FILL.b * 0.82,
                    a: 1.0,
                }
            } else {
                window_control_wash(self, 0.12)
            };

            return window_control(self, *close, *round_corner, selected, Some(fill));
        }

        appearance(self, focused, selected, false, style, |component| {
            let text_color = if matches!(
                style,
                Button::Icon | Button::IconVertical | Button::HeaderBar
            ) && selected
            {
                Some(self.cosmic().accent_text_color().into())
            } else {
                Some(component.on.into())
            };

            (component.pressed.into(), text_color, text_color)
        })
    }

    fn selection_background(&self) -> Background {
        Background::Color(self.cosmic().primary(self.transparent).base.into())
    }
}
