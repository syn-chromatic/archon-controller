use embsys::crates::embedded_graphics;

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::pixelcolor::RgbColor;
use embedded_menu::theme::Theme;

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
        Rgb565::new(26, 39, 9)
    }
}
