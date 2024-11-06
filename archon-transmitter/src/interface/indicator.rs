#![allow(dead_code)]

use embsys::crates::embedded_graphics;

use embedded_graphics::prelude::Size;
use embedded_graphics::primitives::ContainsPoint;
use embedded_graphics::primitives::Primitive;
use embedded_graphics::primitives::PrimitiveStyle;
use embedded_graphics::primitives::Rectangle as RectangleShape;
use embedded_graphics::transform::Transform;
use embedded_graphics::Drawable;

use embedded_menu::interaction::InputState;
use embedded_menu::margin::Insets;
use embedded_menu::selection_indicator::style::interpolate;
use embedded_menu::selection_indicator::style::triangle::Arrow;
use embedded_menu::selection_indicator::style::IndicatorStyle;
use embedded_menu::theme::Theme;

#[derive(Copy, Clone)]
pub enum DynShape {
    Hidden,
    Triangle,
    Border,
    FilledBorder,
    Line,
}

#[derive(Copy, Clone)]
pub enum DynIndicatorShape {
    Hidden,
    Arrow(Arrow),
    Rectangle(RectangleShape),
}

impl Transform for DynIndicatorShape {
    fn translate(&self, by: embedded_graphics::prelude::Point) -> Self {
        match self {
            DynIndicatorShape::Hidden => *self,
            DynIndicatorShape::Arrow(arrow) => Self::Arrow(arrow.translate(by)),
            DynIndicatorShape::Rectangle(rectangle) => Self::Rectangle(rectangle.translate(by)),
        }
    }

    fn translate_mut(&mut self, by: embedded_graphics::prelude::Point) -> &mut Self {
        match self {
            DynIndicatorShape::Hidden => self,
            DynIndicatorShape::Arrow(arrow) => {
                arrow.translate_mut(by);
                self
            }
            DynIndicatorShape::Rectangle(rectangle) => {
                rectangle.translate_mut(by);
                self
            }
        }
    }
}

impl ContainsPoint for DynIndicatorShape {
    fn contains(&self, point: embedded_graphics::prelude::Point) -> bool {
        match self {
            DynIndicatorShape::Hidden => false,
            DynIndicatorShape::Arrow(arrow) => arrow.contains(point),
            DynIndicatorShape::Rectangle(rectangle) => rectangle.contains(point),
        }
    }
}

#[derive(Copy, Clone)]
pub struct DynIndicator {
    shape: DynShape,
}

impl DynIndicator {
    pub fn new(shape: DynShape) -> Self {
        Self { shape }
    }

    pub fn change(&mut self, shape: DynShape) {
        self.shape = shape;
    }
}

impl IndicatorStyle for DynIndicator {
    type Shape = DynIndicatorShape;
    type State = ();

    fn padding(&self, _state: &Self::State, _height: i32) -> Insets {
        Insets {
            left: 10,
            top: 1,
            right: 2,
            bottom: 1,
        }
    }

    fn shape(&self, _state: &Self::State, bounds: RectangleShape, fill_width: u32) -> Self::Shape {
        match self.shape {
            DynShape::Hidden => DynIndicatorShape::Hidden,
            DynShape::Triangle => DynIndicatorShape::Arrow(Arrow::new(bounds, fill_width)),
            DynShape::Border => DynIndicatorShape::Rectangle(RectangleShape::new(
                bounds.top_left,
                Size::new(fill_width, bounds.size.height),
            )),
            DynShape::FilledBorder => DynIndicatorShape::Rectangle(RectangleShape::new(
                bounds.top_left,
                Size::new(bounds.size.width, bounds.size.height),
            )),
            DynShape::Line => DynIndicatorShape::Rectangle(RectangleShape::new(
                bounds.top_left,
                Size::new(fill_width.max(1), bounds.size.height),
            )),
        }
    }

    fn draw<T, D>(
        &self,
        state: &Self::State,
        input_state: InputState,
        theme: &T,
        display: &mut D,
    ) -> Result<Self::Shape, D::Error>
    where
        T: Theme,
        D: embedded_graphics::prelude::DrawTarget<Color = T::Color>,
    {
        let display_area: RectangleShape = display.bounding_box();

        let fill_width: u32 = if let InputState::InProgress(progress) = input_state {
            interpolate(progress as u32, 0, 255, 0, display_area.size.width)
        } else {
            0
        };

        let shape: DynIndicatorShape = self.shape(state, display_area, fill_width);

        match shape {
            DynIndicatorShape::Hidden => {}
            DynIndicatorShape::Arrow(arrow) => {
                arrow.draw(theme.selection_color(), display)?;
            }
            DynIndicatorShape::Rectangle(rectangle) => {
                display_area
                    .into_styled(PrimitiveStyle::with_stroke(theme.selection_color(), 1))
                    .draw(display)?;

                rectangle
                    .into_styled(PrimitiveStyle::with_fill(theme.selection_color()))
                    .draw(display)?;
            }
        }

        Ok(shape)
    }
}
