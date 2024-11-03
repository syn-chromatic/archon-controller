#![allow(dead_code)]
use super::indicator::DynamicIndicator;

use embsys::crates::embedded_graphics;

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::pixelcolor::RgbColor;

use embedded_menu::interaction::programmed::Programmed;
use embedded_menu::selection_indicator::style::triangle::Triangle;
use embedded_menu::selection_indicator::style::Border;
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
        Rgb565::WHITE
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
pub struct DynamicTheme;

impl Theme for DynamicTheme {
    type Color = Rgb565;

    fn text_color(&self) -> Self::Color {
        Rgb565::WHITE
    }

    fn selected_text_color(&self) -> Self::Color {
        Rgb565::WHITE
    }

    fn selection_color(&self) -> Self::Color {
        Rgb565::new(26, 39, 9)
    }
}

impl DynamicTheme {
    pub fn style<R>() -> MenuStyle<DynamicIndicator, Programmed, AnimatedPosition, R, Self> {
        MenuStyle::new(Self)
            .with_selection_indicator(DynamicIndicator::triangle())
            .with_animated_selection_indicator(3)
    }

    pub fn hidden<R>() -> MenuStyle<DynamicIndicator, Programmed, AnimatedPosition, R, DynamicTheme>
    {
        MenuStyle::new(Self)
            .with_selection_indicator(DynamicIndicator::hidden())
            .with_animated_selection_indicator(3)
    }

    pub fn triangle<R>() -> MenuStyle<DynamicIndicator, Programmed, AnimatedPosition, R, Self> {
        MenuStyle::new(Self)
            .with_selection_indicator(DynamicIndicator::triangle())
            .with_animated_selection_indicator(3)
    }

    pub fn border<R>() -> MenuStyle<DynamicIndicator, Programmed, AnimatedPosition, R, Self> {
        MenuStyle::new(Self)
            .with_selection_indicator(DynamicIndicator::border())
            .with_animated_selection_indicator(3)
    }

    pub fn filled_border<R>() -> MenuStyle<DynamicIndicator, Programmed, AnimatedPosition, R, Self>
    {
        MenuStyle::new(Self)
            .with_selection_indicator(DynamicIndicator::filled_border())
            .with_animated_selection_indicator(3)
    }

    pub fn from_actionable<R>(
        actionable: bool,
    ) -> MenuStyle<DynamicIndicator, Programmed, AnimatedPosition, R, DynamicTheme> {
        if actionable {
            return Self::filled_border();
        }
        Self::triangle()
    }
}

#[derive(Clone, Copy)]
pub struct BorderTheme;

impl Theme for BorderTheme {
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

impl BorderTheme {
    pub fn style<R>() -> MenuStyle<Border, Programmed, AnimatedPosition, R, Self> {
        MenuStyle::new(Self)
            .with_selection_indicator(Border)
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
