use embsys::crates::embedded_graphics;

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::pixelcolor::RgbColor;

use embedded_menu::interaction::programmed::Programmed;
use embedded_menu::selection_indicator::style::triangle::Triangle;
use embedded_menu::selection_indicator::style::Line;
use embedded_menu::selection_indicator::AnimatedPosition;
use embedded_menu::selection_indicator::StaticPosition;
use embedded_menu::theme::Theme;
use embedded_menu::MenuStyle;

#[derive(Clone, Copy)]
pub struct StandardTheme;

impl Theme for StandardTheme {
    type Color = Rgb565;

    fn text_color(&self) -> Self::Color {
        Rgb565::WHITE
    }

    fn selected_text_color(&self) -> Self::Color {
        Rgb565::BLACK
    }

    fn selection_color(&self) -> Self::Color {
        Rgb565::new(9, 34, 20)
    }
}

impl StandardTheme {
    pub fn style<R>() -> MenuStyle<Triangle, Programmed, AnimatedPosition, R, Self> {
        MenuStyle::new(Self)
            .with_selection_indicator(Triangle)
            .with_animated_selection_indicator(3)
    }
}

#[derive(Clone, Copy)]
pub struct HiddenSelectorTheme;

impl Theme for HiddenSelectorTheme {
    type Color = Rgb565;

    fn text_color(&self) -> Self::Color {
        Rgb565::WHITE
    }

    fn selected_text_color(&self) -> Self::Color {
        Rgb565::BLACK
    }

    fn selection_color(&self) -> Self::Color {
        Rgb565::BLACK
    }
}

impl HiddenSelectorTheme {
    pub fn style<R>() -> MenuStyle<Line, Programmed, StaticPosition, R, Self> {
        MenuStyle::new(Self)
    }
}
