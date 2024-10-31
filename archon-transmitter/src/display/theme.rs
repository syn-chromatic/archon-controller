use embsys::crates::embedded_graphics;

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::pixelcolor::RgbColor;
use embedded_menu::theme::Theme;

#[derive(Clone, Copy)]
pub struct MenuTheme;

impl Theme for MenuTheme {
    type Color = Rgb565;

    fn text_color(&self) -> Self::Color {
        Rgb565::WHITE
    }

    fn selected_text_color(&self) -> Self::Color {
        Rgb565::BLACK
    }

    fn selection_color(&self) -> Self::Color {
        Rgb565::new(51, 255, 51)
    }
}
