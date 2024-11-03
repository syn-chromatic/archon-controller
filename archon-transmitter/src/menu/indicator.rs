#![allow(dead_code)]

use embsys::crates::embedded_graphics;

use embedded_graphics::prelude::DrawTarget;
use embedded_graphics::prelude::PixelColor;
use embedded_graphics::prelude::Point;
use embedded_graphics::prelude::Size;
use embedded_graphics::primitives::triangle::Triangle as TriangleShape;
use embedded_graphics::primitives::ContainsPoint;
use embedded_graphics::primitives::Primitive;
use embedded_graphics::primitives::PrimitiveStyle;
use embedded_graphics::primitives::Rectangle as RectangleShape;
use embedded_graphics::transform::Transform;
use embedded_graphics::Drawable;
use embedded_layout::align::Align;
use embedded_layout::prelude::horizontal::LeftToRight;
use embedded_layout::prelude::vertical::Center;

use embedded_menu::interaction::InputState;
use embedded_menu::margin::Insets;
use embedded_menu::selection_indicator::style::interpolate;
use embedded_menu::selection_indicator::style::IndicatorStyle;
use embedded_menu::theme::Theme;

#[derive(Clone, Copy)]
pub struct Arrow {
    body: RectangleShape,
    tip: TriangleShape,
}

impl Arrow {
    const SHRINK: i32 = 1;

    pub fn new(bounds: RectangleShape, fill_width: u32) -> Self {
        let body = RectangleShape::new(bounds.top_left, Size::new(fill_width, bounds.size.height));

        let tip = TriangleShape::new(
            Point::new(0, Self::SHRINK),
            Point::new(0, Self::tip_width(bounds)),
            Point::new(
                bounds.size.height as i32 / 2 - Self::SHRINK,
                bounds.size.height as i32 / 2,
            ),
        )
        .align_to(&body, LeftToRight, Center)
        // e-layout doesn't align well to 0 area rectangles
        .translate(Point::new(if body.is_zero_sized() { -1 } else { 0 }, 0));

        Self { body, tip }
    }

    pub fn tip_width(bounds: RectangleShape) -> i32 {
        bounds.size.height as i32 - 1 - Self::SHRINK
    }

    pub fn draw<D, C>(&self, color: C, target: &mut D) -> Result<(), D::Error>
    where
        C: PixelColor,
        D: DrawTarget<Color = C>,
    {
        let style = PrimitiveStyle::with_fill(color);

        self.body.into_styled(style).draw(target)?;
        self.tip.into_styled(style).draw(target)?;

        Ok(())
    }
}

#[derive(Copy, Clone)]
pub enum DynamicShape {
    Hidden,
    Triangle(Arrow),
    Border(RectangleShape),
}

#[derive(Copy, Clone)]
pub enum DynamicShapePrimitive {
    Hidden,
    Triangle,
    Border,
    FilledBorder,
}

impl Transform for DynamicShape {
    fn translate(&self, by: embedded_graphics::prelude::Point) -> Self {
        match self {
            DynamicShape::Hidden => *self,
            DynamicShape::Triangle(arrow) => Self::Triangle(Arrow {
                body: arrow.body.translate(by),
                tip: arrow.tip.translate(by),
            }),
            DynamicShape::Border(rectangle) => Self::Border(rectangle.translate(by)),
        }
    }

    fn translate_mut(&mut self, by: embedded_graphics::prelude::Point) -> &mut Self {
        match self {
            DynamicShape::Hidden => self,
            DynamicShape::Triangle(arrow) => {
                arrow.body.translate_mut(by);
                arrow.tip.translate_mut(by);
                self
            }
            DynamicShape::Border(rectangle) => {
                rectangle.translate_mut(by);
                self
            }
        }
    }
}

impl ContainsPoint for DynamicShape {
    fn contains(&self, point: embedded_graphics::prelude::Point) -> bool {
        match self {
            DynamicShape::Hidden => false,
            DynamicShape::Triangle(arrow) => {
                arrow.body.contains(point) || arrow.tip.contains(point)
            }
            DynamicShape::Border(rectangle) => rectangle.contains(point),
        }
    }
}

#[derive(Copy, Clone)]
pub struct DynamicIndicator {
    shape: DynamicShapePrimitive,
}

impl DynamicIndicator {
    pub fn hidden() -> Self {
        Self {
            shape: DynamicShapePrimitive::Hidden,
        }
    }

    pub fn triangle() -> Self {
        Self {
            shape: DynamicShapePrimitive::Triangle,
        }
    }

    pub fn border() -> Self {
        Self {
            shape: DynamicShapePrimitive::Border,
        }
    }

    pub fn filled_border() -> Self {
        Self {
            shape: DynamicShapePrimitive::FilledBorder,
        }
    }
}

impl IndicatorStyle for DynamicIndicator {
    type Shape = DynamicShape;

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
            DynamicShapePrimitive::Hidden => DynamicShape::Hidden,
            DynamicShapePrimitive::Triangle => {
                DynamicShape::Triangle(Arrow::new(bounds, fill_width))
            }
            DynamicShapePrimitive::Border => DynamicShape::Border(RectangleShape::new(
                bounds.top_left,
                Size::new(fill_width, bounds.size.height),
            )),
            DynamicShapePrimitive::FilledBorder => DynamicShape::Border(RectangleShape::new(
                bounds.top_left,
                Size::new(bounds.size.width, bounds.size.height),
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

        let shape: DynamicShape = self.shape(state, display_area, fill_width);

        match shape {
            DynamicShape::Hidden => {}
            DynamicShape::Triangle(arrow) => {
                arrow.draw(theme.selection_color(), display)?;
            }
            DynamicShape::Border(rectangle) => {
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
