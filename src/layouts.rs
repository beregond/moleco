use crate::common::Scheme;
use image::{ImageBuffer, Rgba};
use log::trace;
use palette::{FromColor, Hsv, Srgb};

#[derive(clap::ValueEnum, Clone, Default, Debug)]
pub enum Layout {
    #[default]
    Quinqunx,
    Rhombus,
    Stripe,
}

pub struct Picture {
    base_size: u32,
    layout: Layout,
    border_size: u32,
    schemes: Vec<Scheme>,
}

impl Picture {
    pub fn new(base_size: u32, layout: Layout, border_size: u32) -> Self {
        Self {
            base_size,
            layout,
            border_size,
            schemes: Vec::new(),
        }
    }

    pub fn add_scheme(&mut self, scheme: Scheme) {
        self.schemes.push(scheme);
    }

    pub fn generate(&self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let border_color: Srgb<u8> = Srgb::from_color(Hsv::new(0.0, 0.0, 0.1)).into_format();
        let size = self.base_size * 2 + self.border_size * 3;
        let half_border = (self.border_size - 1) / 2;
        let half_size = (self.base_size - 1) / 2;
        let mut buffer = ImageBuffer::new(size, size);
        let primary = self.schemes[0].primary.srgb;
        let first_accent = self.schemes[0].first_accent.srgb;
        let second_accent = self.schemes[0].second_accent.srgb;
        let complementary = self.schemes[0].complementary.srgb;

        let mut layer0: Vec<ShapeType> = Vec::new();
        let mut layers: Vec<Vec<ShapeType>> = Vec::new();

        layer0.push(ShapeType::Rectangle(Rectangle {
            x: self.base_size + self.border_size + half_border,
            y: half_size + self.border_size,
            width: self.base_size,
            height: self.base_size,
            orientation: Orientation::Vertical,
            color: primary,
        }));
        layer0.push(ShapeType::Rectangle(Rectangle {
            x: half_size + self.border_size,
            y: self.base_size + self.border_size + half_border,
            width: self.base_size,
            height: self.base_size,
            orientation: Orientation::Vertical,
            color: first_accent,
        }));
        layer0.push(ShapeType::Rectangle(Rectangle {
            x: self.base_size + half_size + self.border_size * 2,
            y: self.base_size + self.border_size + half_border,
            width: self.base_size,
            height: self.base_size,
            orientation: Orientation::Vertical,
            color: second_accent,
        }));
        layer0.push(ShapeType::Rectangle(Rectangle {
            x: self.base_size + self.border_size + half_border,
            y: self.base_size + half_size + self.border_size * 2,
            width: self.base_size,
            height: self.base_size,
            orientation: Orientation::Vertical,
            color: complementary,
        }));
        layers.push(layer0);

        let mut layer1: Vec<ShapeType> = Vec::new();
        layer1.push(ShapeType::Line(Line {
            x1: half_size + self.border_size,
            y1: half_size + self.border_size,
            x2: self.base_size + half_size + self.border_size * 2,
            y2: self.base_size + half_size + self.border_size * 2,
            border_size: self.border_size,
            color: border_color,
        }));
        layer1.push(ShapeType::Line(Line {
            x1: half_size + self.border_size,
            y1: self.base_size + half_size + self.border_size * 2,
            x2: self.base_size + half_size + self.border_size * 2,
            y2: half_size + self.border_size,
            border_size: self.border_size,
            color: border_color,
        }));
        layer1.push(ShapeType::Line(Line {
            x1: half_border,
            y1: self.base_size + self.border_size + half_border,
            x2: self.base_size + self.border_size + half_border,
            y2: half_border,
            border_size: self.border_size,
            color: border_color,
        }));
        layer1.push(ShapeType::Line(Line {
            x1: self.base_size + self.border_size + half_border,
            y1: half_border,
            x2: self.base_size * 2 + self.border_size * 2 + half_border,
            y2: self.base_size + self.border_size + half_border,
            border_size: self.border_size,
            color: border_color,
        }));
        layer1.push(ShapeType::Line(Line {
            x1: self.base_size * 2 + self.border_size * 2 + half_border,
            y1: self.base_size + self.border_size + half_border,
            x2: self.base_size + self.border_size + half_border,
            y2: self.base_size * 2 + self.border_size * 2 + half_border,
            border_size: self.border_size,
            color: border_color,
        }));
        layer1.push(ShapeType::Line(Line {
            x1: half_border,
            y1: self.base_size + self.border_size + half_border,
            x2: self.base_size + self.border_size + half_border,
            y2: self.base_size * 2 + self.border_size * 2 + half_border,
            border_size: self.border_size,
            color: border_color,
        }));
        layers.push(layer1);

        for layer in &layers {
            for shape in layer {
                match shape {
                    ShapeType::Rectangle(rectangle) => rectangle.draw(&mut buffer),
                    ShapeType::Line(line) => line.draw(&mut buffer),
                }
            }
        }
        buffer
    }
}

trait Shape {
    fn draw(&self, buffer: &mut ImageBuffer<Rgba<u8>, Vec<u8>>);
}

#[derive(Debug)]
enum Orientation {
    /// □
    Horizontal,
    /// ◇
    Vertical,
}

#[derive(Debug)]
struct Rectangle {
    x: u32,
    y: u32,
    // Width and height are the with respect to gric, so if rectangle is vertical - those are not
    // actual width and height of the rectangle, but borders of the rectangle.
    width: u32,
    height: u32,
    orientation: Orientation,
    color: Srgb<u8>,
}

impl Rectangle {
    fn pixel_belongs(&self, x: u32, y: u32) -> bool {
        match self.orientation {
            Orientation::Horizontal => self.pixel_belongs_horizontal(x, y),
            Orientation::Vertical => self.pixel_belongs_vertical(x, y),
        }
    }

    fn pixel_belongs_horizontal(&self, x: u32, y: u32) -> bool {
        let distance_from_x = abs(self.x, x);
        let distance_from_y = abs(self.y, y);
        distance_from_x < self.width / 2 && distance_from_y < self.height / 2
    }

    fn pixel_belongs_vertical(&self, x: u32, y: u32) -> bool {
        let distance_from_x = abs(self.x, x);
        let distance_from_y = abs(self.y, y);

        let sum = distance_from_y + distance_from_x;
        let line = (self.width - 1) / 2;
        return sum <= line;
    }
}

impl Shape for Rectangle {
    fn draw(&self, buffer: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) {
        let half_size = (self.width - 1) / 2;
        let start_x = self.x - half_size;
        let end_x = self.x + self.width / 2;
        let start_y = self.y - half_size;
        let end_y = self.y + self.height / 2;
        for x in start_x..=end_x {
            for y in start_y..=end_y {
                if self.pixel_belongs(x, y) {
                    buffer.put_pixel(
                        x,
                        y,
                        Rgba([self.color.red, self.color.green, self.color.blue, 255u8]),
                    );
                }
            }
        }
    }
}

fn abs(a: u32, b: u32) -> u32 {
    (a as i32 - b as i32).abs() as u32
}

struct Line {
    x1: u32,
    y1: u32,
    x2: u32,
    y2: u32,
    border_size: u32,
    color: Srgb<u8>,
}

impl Shape for Line {
    fn draw(&self, buffer: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) {
        // Decide which point is the starting point
        let (x1, x2, y1, y2) = match (self.x1, self.x2, self.y1, self.y2) {
            (x1, x2, y1, y2) if x1 > x2 => (x2 as i32, x1 as i32, y2 as i32, y1 as i32),
            (x1, x2, y1, y2) => (x1 as i32, x2 as i32, y1 as i32, y2 as i32),
        };

        let half_border = ((self.border_size - 1) / 2) as i32;

        let max_width = buffer.width() as i32;
        let max_height = buffer.height() as i32;
        for x in x1..x2 + 1 {
            let y = y1 + (y2 - y1) * (x - x1) / (x2 - x1);
            for i in -half_border..half_border + 1 {
                for j in -half_border..half_border + 1 {
                    if x + i < max_width && y + j < max_height && x + i >= 0 && y + j >= 0 {
                        buffer.put_pixel(
                            (x + i) as u32,
                            (y + j) as u32,
                            Rgba([self.color.red, self.color.green, self.color.blue, 255u8]),
                        );
                    }
                }
            }
        }
    }
}

enum ShapeType {
    Rectangle(Rectangle),
    Line(Line),
}
