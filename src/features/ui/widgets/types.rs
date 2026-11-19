//! Stateless widget builder types (`ButtonVariant`, `Button`, `TextInput`, `Toast`).
//! Zero platform `#[cfg]`.

use crate::features::blocks::BlockId;

/// Visual theme preset for buttons.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonVariant {
    Standard,
    Primary,
    Danger,
    Accent,
    Tab { active: bool },
    Flat {
        bg: u32,
        hover_bg: u32,
        border: u32,
        hover_border: u32,
    },
}

/// Stateless button widget configuration.
#[derive(Clone, Copy)]
pub struct Button<'a> {
    pub label: &'a str,
    pub variant: ButtonVariant,
    pub hovered: bool,
    pub focused: bool,
    pub scale: i32,
    pub text_color: Option<u32>,
    pub outer_border: bool,
    pub beveled: bool,
}

impl<'a> Button<'a> {
    #[inline]
    pub const fn new(label: &'a str) -> Self {
        Self {
            label,
            variant: ButtonVariant::Standard,
            hovered: false,
            focused: false,
            scale: 2,
            text_color: None,
            outer_border: false,
            beveled: true,
        }
    }

    #[inline]
    pub const fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    #[inline]
    pub const fn primary(mut self) -> Self {
        self.variant = ButtonVariant::Primary;
        self
    }

    #[inline]
    pub const fn danger(mut self) -> Self {
        self.variant = ButtonVariant::Danger;
        self
    }

    #[inline]
    pub const fn accent(mut self) -> Self {
        self.variant = ButtonVariant::Accent;
        self
    }

    #[inline]
    pub const fn tab(mut self, active: bool) -> Self {
        self.variant = ButtonVariant::Tab { active };
        self
    }

    #[inline]
    pub const fn hovered(mut self, hovered: bool) -> Self {
        self.hovered = hovered;
        self
    }

    #[inline]
    pub const fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    #[inline]
    pub const fn scale(mut self, scale: i32) -> Self {
        self.scale = scale;
        self
    }

    #[inline]
    pub const fn text_color(mut self, color: u32) -> Self {
        self.text_color = Some(color);
        self
    }

    /// Uses a 2px outer border frame (`x-2, y-2, w+4, h+4`) instead of inner 3D bevels.
    #[inline]
    pub const fn outer_frame(mut self) -> Self {
        self.outer_border = true;
        self.beveled = false;
        self
    }

    /// Uses a flat inner border instead of 3D bevels.
    #[inline]
    pub const fn flat_border(mut self) -> Self {
        self.outer_border = false;
        self.beveled = false;
        self
    }
}

/// Stateless text input box widget configuration.
#[derive(Clone, Copy)]
pub struct TextInput<'a> {
    pub text: &'a str,
    pub placeholder: &'a str,
    pub label: Option<&'a str>,
    pub focused: bool,
    pub hovered: bool,
    pub selected: bool,
    pub cursor_visible: bool,
    pub scale: i32,
    pub outer_border: bool,
}

impl<'a> TextInput<'a> {
    #[inline]
    pub const fn new(text: &'a str) -> Self {
        Self {
            text,
            placeholder: "",
            label: None,
            focused: false,
            hovered: false,
            selected: false,
            cursor_visible: false,
            scale: 2,
            outer_border: false,
        }
    }

    #[inline]
    pub const fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    #[inline]
    pub const fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    #[inline]
    pub const fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    #[inline]
    pub const fn hovered(mut self, hovered: bool) -> Self {
        self.hovered = hovered;
        self
    }

    #[inline]
    pub const fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    #[inline]
    pub const fn cursor(mut self, visible: bool) -> Self {
        self.cursor_visible = visible;
        self
    }

    #[inline]
    pub const fn scale(mut self, scale: i32) -> Self {
        self.scale = scale;
        self
    }

    #[inline]
    pub const fn outer_frame(mut self) -> Self {
        self.outer_border = true;
        self
    }
}

/// Toast notification configuration (used by Quests, Achievements, Multiplayer, Music).
#[derive(Clone, Copy)]
pub struct Toast<'a> {
    pub heading: &'a str,
    pub title: &'a str,
    pub subtitle: &'a str,
    pub accent_color: u32,
    pub icon: Option<BlockId>,
    pub y: i32,
}
