use super::indicator::DynIndicator;
use super::indicator::DynShape;

use embsys::crates::embedded_graphics;

use embedded_graphics::mono_font::iso_8859_1::FONT_6X12;

use embedded_menu::interaction::programmed::Programmed;
use embedded_menu::selection_indicator::AnimatedPosition;
use embedded_menu::theme::Theme;
use embedded_menu::MenuStyle;

#[derive(Copy, Clone)]
pub struct DynMenuStyle<T, R>
where
    T: Theme,
    R: Clone,
{
    theme: T,
    indicator: DynIndicator,
    style: MenuStyle<DynIndicator, Programmed, AnimatedPosition, R, T>,
}

impl<T, R> DynMenuStyle<T, R>
where
    T: Theme,
    R: Clone,
{
    fn create_style(
        theme: T,
        indicator: DynIndicator,
    ) -> MenuStyle<DynIndicator, Programmed, AnimatedPosition, R, T> {
        MenuStyle::new(theme)
            .with_title_font(&FONT_6X12)
            .with_font(&FONT_6X12)
            .with_selection_indicator(indicator)
            .with_animated_selection_indicator(3)
    }
}

impl<T, R> DynMenuStyle<T, R>
where
    T: Theme,
    R: Clone,
{
    pub fn new(theme: T, shape: DynShape) -> Self {
        let indicator: DynIndicator = DynIndicator::new(shape);
        Self {
            theme,
            indicator,
            style: Self::create_style(theme, indicator),
        }
    }

    pub fn set_theme(&mut self, theme: T) {
        self.theme = theme;
    }

    pub fn set_indicator(&mut self, shape: DynShape) {
        self.indicator.change(shape);
        self.style = Self::create_style(self.theme, self.indicator);
    }
}

impl<R, T> From<DynMenuStyle<T, R>> for MenuStyle<DynIndicator, Programmed, AnimatedPosition, R, T>
where
    T: Theme,
    R: Clone,
{
    fn from(value: DynMenuStyle<T, R>) -> Self {
        value.style
    }
}

impl<T, R> core::ops::Deref for DynMenuStyle<T, R>
where
    T: Theme,
    R: Clone,
{
    type Target = MenuStyle<DynIndicator, Programmed, AnimatedPosition, R, T>;

    fn deref(&self) -> &Self::Target {
        &self.style
    }
}
