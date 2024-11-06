use super::indicator::DynShape;
use super::style::DynMenuStyle;

use embedded_menu::theme::Theme;

pub trait ActionableSelect {
    fn is_actionable(&self) -> bool;
    fn set_indicator<T, R>(&self, style: &mut DynMenuStyle<T, R>)
    where
        T: Theme,
        R: Clone,
    {
        match self.is_actionable() {
            true => style.set_indicator(DynShape::FilledBorder),
            false => style.set_indicator(DynShape::Triangle),
        }
    }
}
