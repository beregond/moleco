use crate::common::Scheme;
use crate::tokenize::{generate_compound_hierarchy, Compound, CompoundKind};
use image::{ImageBuffer, Rgba};
use log::trace;
use palette::{FromColor, Hsv, Srgba};

pub struct Picture {
    base_size: u32,
    border_size: u32,
    schemes: Vec<Scheme>,
    bar: Option<(String, String)>,
}

impl Picture {
    pub fn new(base_size: u32, border_size: u32) -> Self {
        Self {
            base_size,
            border_size,
            schemes: Vec::new(),
            //FIXME: such wrong name
            bar: None,
        }
    }

    pub fn add_scheme(&mut self, scheme: Scheme) {
        self.schemes.push(scheme);
    }

    pub fn add_bar(&mut self, indexing: String, concentration: String) {
        self.bar = Some((indexing, concentration));
    }

    fn generate_bar(&self) -> Option<Compound> {
        match &self.bar {
            None => None,
            Some((indexing, concentration)) => {
                Some(generate_compound_hierarchy(indexing, concentration))
            }
        }
    }

    pub fn generate(&self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let border_color: Srgba<u8> = Srgba::from_color(Hsv::new(0.0, 0.0, 0.1)).into_format();
        let eraser = Srgba::new(0, 0, 0, 0);
        let cell_size = self.base_size * 2 + self.border_size * 3;
        let width = cell_size * self.schemes.len() as u32
            - (self.schemes.len() as u32 - 1) * self.border_size;
        let half_border = (self.border_size - 1) / 2;
        let half_size = (self.base_size - 1) / 2;
        let quarter_size = (half_size - 1) / 2;
        let eight_size = (quarter_size - 1) / 2;

        trace!("border_size: {}", self.border_size);
        trace!("half_size: {}", half_border);

        let mut height = cell_size;
        let mut layers: Vec<Vec<ShapeType>> = Vec::new();
        if let Some(compound) = self.generate_bar() {
            let level = compound.calculate_level();
            height += quarter_size + level * eight_size + half_border;
            let y_offset = cell_size + quarter_size;
            layers =
                self.draw_compound_bar(compound, level, width - half_border, y_offset, eight_size);
        }

        let mut buffer = ImageBuffer::new(width, height);
        let mut offset = 0;

        for scheme in &self.schemes {
            let primary = scheme.primary.srgb;
            let first_accent = scheme.first_accent.srgb;
            let second_accent = scheme.second_accent.srgb;
            let complementary = scheme.complementary.srgb;
            let mut base_color: Vec<ShapeType> = Vec::new();

            base_color.push(ShapeType::Square(Square {
                x: offset + self.base_size + self.border_size + half_border,
                y: half_size + self.border_size,
                size: self.base_size,
                orientation: Orientation::Vertical,
                color: primary.into(),
            }));
            base_color.push(ShapeType::Square(Square {
                x: offset + half_size + self.border_size,
                y: self.base_size + self.border_size + half_border,
                size: self.base_size,
                orientation: Orientation::Vertical,
                color: first_accent.into(),
            }));
            base_color.push(ShapeType::Square(Square {
                x: offset + self.base_size + half_size + self.border_size * 2,
                y: self.base_size + self.border_size + half_border,
                size: self.base_size,
                orientation: Orientation::Vertical,
                color: second_accent.into(),
            }));
            base_color.push(ShapeType::Square(Square {
                x: offset + self.base_size + self.border_size + half_border,
                y: self.base_size + half_size + self.border_size * 2,
                size: self.base_size,
                orientation: Orientation::Vertical,
                color: complementary.into(),
            }));
            layers.push(base_color);

            let mut base_lines: Vec<ShapeType> = Vec::new();
            // Cross - left top to right bottom
            base_lines.push(ShapeType::Line(Line {
                x1: offset + half_size + self.border_size,
                y1: half_size + self.border_size,
                x2: offset + self.base_size + half_size + self.border_size * 2,
                y2: self.base_size + half_size + self.border_size * 2,
                border_size: self.border_size,
                color: border_color,
            }));
            // Cross - left bottom to right top
            base_lines.push(ShapeType::Line(Line {
                x1: offset + half_size + self.border_size,
                y1: self.base_size + half_size + self.border_size * 2,
                x2: offset + self.base_size + half_size + self.border_size * 2,
                y2: half_size + self.border_size,
                border_size: self.border_size,
                color: border_color,
            }));
            base_lines.push(ShapeType::Line(Line {
                x1: offset + half_border,
                y1: self.base_size + self.border_size + half_border,
                x2: offset + self.base_size + self.border_size + half_border,
                y2: half_border,
                border_size: self.border_size,
                color: border_color,
            }));
            base_lines.push(ShapeType::Line(Line {
                x1: offset + self.base_size + self.border_size + half_border,
                y1: half_border,
                x2: offset + self.base_size * 2 + self.border_size * 2 + half_border,
                y2: self.base_size + self.border_size + half_border,
                border_size: self.border_size,
                color: border_color,
            }));
            base_lines.push(ShapeType::Line(Line {
                x1: offset + self.base_size * 2 + self.border_size * 2 + half_border,
                y1: self.base_size + self.border_size + half_border,
                x2: offset + self.base_size + self.border_size + half_border,
                y2: self.base_size * 2 + self.border_size * 2 + half_border,
                border_size: self.border_size,
                color: border_color,
            }));
            base_lines.push(ShapeType::Line(Line {
                x1: offset + half_border,
                y1: self.base_size + self.border_size + half_border,
                x2: offset + self.base_size + self.border_size + half_border,
                y2: self.base_size * 2 + self.border_size * 2 + half_border,
                border_size: self.border_size,
                color: border_color,
            }));
            layers.push(base_lines);

            let mut cutouts: Vec<ShapeType> = Vec::new();
            // Central cutout
            cutouts.push(ShapeType::Square(Square {
                x: offset + self.base_size + self.border_size + half_border,
                y: self.base_size + self.border_size + half_border,
                size: quarter_size,
                orientation: Orientation::Horizontal,
                color: eraser,
            }));
            // Left top cutout
            cutouts.push(ShapeType::Square(Square {
                x: offset + half_size + self.border_size,
                y: half_size + self.border_size,
                size: quarter_size,
                orientation: Orientation::Horizontal,
                color: eraser,
            }));
            // Right bottom cutout
            cutouts.push(ShapeType::Square(Square {
                x: offset + self.base_size + half_size + self.border_size * 2,
                y: self.base_size + half_size + self.border_size * 2,
                size: quarter_size,
                orientation: Orientation::Horizontal,
                color: eraser,
            }));
            // Left bottom cutout
            cutouts.push(ShapeType::Square(Square {
                x: offset + half_size + self.border_size,
                y: self.base_size + half_size + self.border_size * 2,
                size: quarter_size,
                orientation: Orientation::Horizontal,
                color: eraser,
            }));
            // Right top cutout
            cutouts.push(ShapeType::Square(Square {
                x: offset + self.base_size + half_size + self.border_size * 2,
                y: half_size + self.border_size,
                size: quarter_size,
                orientation: Orientation::Horizontal,
                color: eraser,
            }));

            layers.push(cutouts);
            let mut inner_border: Vec<ShapeType> = Vec::new();
            // Left top cutout borders
            inner_border.push(ShapeType::Line(Line {
                x1: offset + half_size + self.border_size + eight_size,
                y1: half_size + self.border_size - eight_size,
                x2: offset + half_size + self.border_size + eight_size,
                y2: half_size + self.border_size + eight_size,
                border_size: self.border_size,
                color: border_color,
            }));
            inner_border.push(ShapeType::Line(Line {
                x1: offset + half_size + self.border_size - eight_size,
                y1: half_size + self.border_size + eight_size,
                x2: offset + half_size + self.border_size + eight_size,
                y2: half_size + self.border_size + eight_size,
                border_size: self.border_size,
                color: border_color,
            }));

            // Right bottom cutout borders
            inner_border.push(ShapeType::Line(Line {
                x1: offset + self.base_size + half_size + self.border_size * 2 - eight_size,
                y1: self.base_size + half_size + self.border_size * 2 - eight_size,
                x2: offset + self.base_size + half_size + self.border_size * 2 + eight_size,
                y2: self.base_size + half_size + self.border_size * 2 - eight_size,
                border_size: self.border_size,
                color: border_color,
            }));
            inner_border.push(ShapeType::Line(Line {
                x1: offset + self.base_size + half_size + self.border_size * 2 - eight_size,
                y1: self.base_size + half_size + self.border_size * 2 - eight_size,
                x2: offset + self.base_size + half_size + self.border_size * 2 - eight_size,
                y2: self.base_size + half_size + self.border_size * 2 + eight_size,
                border_size: self.border_size,
                color: border_color,
            }));

            // Left bottom cutout borders
            inner_border.push(ShapeType::Line(Line {
                x1: offset + half_size + self.border_size - eight_size,
                y1: self.base_size + half_size + self.border_size * 2 - eight_size,
                x2: offset + half_size + self.border_size + eight_size,
                y2: self.base_size + half_size + self.border_size * 2 - eight_size,
                border_size: self.border_size,
                color: border_color,
            }));
            inner_border.push(ShapeType::Line(Line {
                x1: offset + half_size + self.border_size + eight_size,
                y1: self.base_size + half_size + self.border_size * 2 - eight_size,
                x2: offset + half_size + self.border_size + eight_size,
                y2: self.base_size + half_size + self.border_size * 2 + eight_size,
                border_size: self.border_size,
                color: border_color,
            }));

            // Right top cutout borders
            inner_border.push(ShapeType::Line(Line {
                x1: offset + self.base_size + half_size + self.border_size * 2 - eight_size,
                y1: half_size + self.border_size - eight_size,
                x2: offset + self.base_size + half_size + self.border_size * 2 - eight_size,
                y2: half_size + self.border_size + eight_size,
                border_size: self.border_size,
                color: border_color,
            }));
            inner_border.push(ShapeType::Line(Line {
                x1: offset + self.base_size + half_size + self.border_size * 2 - eight_size,
                y1: half_size + self.border_size + eight_size,
                x2: offset + self.base_size + half_size + self.border_size * 2 + eight_size,
                y2: half_size + self.border_size + eight_size,
                border_size: self.border_size,
                color: border_color,
            }));

            // Central cutout borders
            inner_border.push(ShapeType::Line(Line {
                x1: offset + self.base_size + self.border_size + half_border - eight_size,
                y1: self.base_size + self.border_size + half_border - eight_size,
                x2: offset + self.base_size + self.border_size + half_border + eight_size,
                y2: self.base_size + self.border_size + half_border - eight_size,
                border_size: self.border_size,
                color: border_color,
            }));
            inner_border.push(ShapeType::Line(Line {
                x1: offset + self.base_size + self.border_size + half_border - eight_size,
                y1: self.base_size + self.border_size + half_border - eight_size,
                x2: offset + self.base_size + self.border_size + half_border - eight_size,
                y2: self.base_size + self.border_size + half_border + eight_size,
                border_size: self.border_size,
                color: border_color,
            }));
            inner_border.push(ShapeType::Line(Line {
                x1: offset + self.base_size + self.border_size + half_border - eight_size,
                y1: self.base_size + self.border_size + half_border + eight_size,
                x2: offset + self.base_size + self.border_size + half_border + eight_size,
                y2: self.base_size + self.border_size + half_border + eight_size,
                border_size: self.border_size,
                color: border_color,
            }));
            inner_border.push(ShapeType::Line(Line {
                x1: offset + self.base_size + self.border_size + half_border + eight_size,
                y1: self.base_size + self.border_size + half_border - eight_size,
                x2: offset + self.base_size + self.border_size + half_border + eight_size,
                y2: self.base_size + self.border_size + half_border + eight_size,
                border_size: self.border_size,
                color: border_color,
            }));

            layers.push(inner_border);

            offset += cell_size - self.border_size;
        }

        for layer in &layers {
            for shape in layer {
                match shape {
                    ShapeType::Square(square) => square.draw(&mut buffer),
                    ShapeType::Line(line) => line.draw(&mut buffer),
                    ShapeType::Rectangle(rectangle) => rectangle.draw(&mut buffer),
                }
            }
        }
        buffer
    }

    fn draw_compound_bar(
        &self,
        compound: Compound,
        level: u32,
        width: u32,
        y_offset: u32,
        base_bar_size: u32,
    ) -> Vec<Vec<ShapeType>> {
        let mut layers: Vec<Vec<ShapeType>> = Vec::new();
        let mut bar_layers: Vec<ShapeType> = Vec::new();
        let mut line_layers: Vec<ShapeType> = Vec::new();

        //compound.calculate_concentration();

        // Offset and width gets '-1' because from this time we are working on the actual pixels,
        // which are indexed from 0.
        self.draw_components(
            compound.components,
            level,
            y_offset - 1,
            0,
            width - 1,
            base_bar_size,
            &mut bar_layers,
            &mut line_layers,
        );

        layers.push(bar_layers);
        layers.push(line_layers);
        layers
    }

    fn draw_components(
        &self,
        components: Vec<CompoundKind>,
        level: u32,
        y_offset: u32,
        start_x: u32,
        end_x: u32,
        base_bar_size: u32,
        bar_layers: &mut Vec<ShapeType>,
        line_layers: &mut Vec<ShapeType>,
    ) -> () {
        let available_width = end_x - start_x;
        let ww = available_width / components.len() as u32;

        let mut start = start_x;
        let mut end = start_x + ww;

        line_layers.push(ShapeType::Line(Line {
            x1: start_x,
            y1: y_offset,
            x2: end_x,
            y2: y_offset,
            border_size: self.border_size,
            color: Srgba::from_color(Hsv::new(0.0, 0.0, 0.1)).into_format(),
        }));
        line_layers.push(ShapeType::Line(Line {
            x1: start_x,
            y1: y_offset,
            x2: start_x,
            y2: y_offset + base_bar_size * level,
            border_size: self.border_size,
            color: Srgba::from_color(Hsv::new(0.0, 0.0, 0.1)).into_format(),
        }));
        line_layers.push(ShapeType::Line(Line {
            x1: start_x,
            y1: y_offset + base_bar_size * level,
            x2: end_x,
            y2: y_offset + base_bar_size * level,
            border_size: self.border_size,
            color: Srgba::from_color(Hsv::new(0.0, 0.0, 0.1)).into_format(),
        }));
        line_layers.push(ShapeType::Line(Line {
            x1: end_x,
            y1: y_offset,
            x2: end_x,
            y2: y_offset + base_bar_size * level,
            border_size: self.border_size,
            color: Srgba::from_color(Hsv::new(0.0, 0.0, 0.1)).into_format(),
        }));

        for component in components {
            match component {
                CompoundKind::Compound(compound) => {
                    self.draw_components(
                        compound.components,
                        level - 1,
                        y_offset,
                        start,
                        end,
                        base_bar_size,
                        bar_layers,
                        line_layers,
                    );
                    bar_layers.push(ShapeType::Rectangle(Rectangle {
                        x: start,
                        y: y_offset + base_bar_size,
                        width: end - start,
                        height: base_bar_size * (level - 1),
                        color: Srgba::from_color(Hsv::new(0.0, 0.0, 0.8)).into_format(),
                    }));
                    line_layers.push(ShapeType::Line(Line {
                        x1: start,
                        y1: y_offset + base_bar_size,
                        x2: start + end - start,
                        y2: y_offset + base_bar_size,
                        border_size: self.border_size,
                        color: Srgba::from_color(Hsv::new(0.0, 0.0, 0.1)).into_format(),
                    }));
                    line_layers.push(ShapeType::Line(Line {
                        x1: start + end - start,
                        y1: y_offset + base_bar_size,
                        x2: start + end - start,
                        y2: y_offset + base_bar_size * level,
                        border_size: self.border_size,
                        color: Srgba::from_color(Hsv::new(0.0, 0.0, 0.1)).into_format(),
                    }));
                    line_layers.push(ShapeType::Line(Line {
                        x1: start,
                        y1: y_offset + base_bar_size,
                        x2: start,
                        y2: y_offset + base_bar_size * level,
                        border_size: self.border_size,
                        color: Srgba::from_color(Hsv::new(0.0, 0.0, 0.1)).into_format(),
                    }));
                    line_layers.push(ShapeType::Line(Line {
                        x1: start,
                        y1: y_offset + base_bar_size * level,
                        x2: start + end - start,
                        y2: y_offset + base_bar_size * level,
                        border_size: self.border_size,
                        color: Srgba::from_color(Hsv::new(0.0, 0.0, 0.1)).into_format(),
                    }));
                }
                CompoundKind::Substance(substance) => {
                    let index: Result<usize, _> = substance.index.parse();
                    match index {
                        Ok(value) => {
                            bar_layers.push(ShapeType::Rectangle(Rectangle {
                                x: start,
                                y: y_offset,
                                width: ww,
                                height: base_bar_size * level,
                                color: self.schemes[value - 1].primary.srgb.into(),
                            }));
                        }
                        Err(_) => {
                            trace!("Failed to parse index: {}", substance.index);
                        }
                    }
                }
            }
            start = end;
            end += ww;
            line_layers.push(ShapeType::Line(Line {
                x1: start,
                y1: y_offset,
                x2: start,
                y2: y_offset + base_bar_size * level,
                border_size: self.border_size,
                color: Srgba::from_color(Hsv::new(0.0, 0.0, 0.1)).into_format(),
            }));
        }
    }
}

#[derive(Debug)]
enum Orientation {
    /// □
    Horizontal,
    /// ◇
    Vertical,
}

// The square is drawn in very specific way, check draw method to understand how it works.
#[derive(Debug)]
struct Square {
    x: u32,
    y: u32,
    size: u32,
    orientation: Orientation,
    color: Srgba<u8>,
}

impl Square {
    fn pixel_belongs(&self, x: u32, y: u32) -> bool {
        match self.orientation {
            Orientation::Horizontal => self.pixel_belongs_in_horizontal(x, y),
            Orientation::Vertical => self.pixel_belongs_in_vertical(x, y),
        }
    }

    fn pixel_belongs_in_horizontal(&self, x: u32, y: u32) -> bool {
        let distance_from_x = abs(self.x, x);
        let distance_from_y = abs(self.y, y);

        let line = (self.size - 1) / 2;
        distance_from_x <= line && distance_from_y <= line
    }

    fn pixel_belongs_in_vertical(&self, x: u32, y: u32) -> bool {
        let distance_from_x = abs(self.x, x);
        let distance_from_y = abs(self.y, y);

        let sum = distance_from_y + distance_from_x;
        let line = (self.size - 1) / 2;
        return sum <= line;
    }

    // Insteado of square beeing anchored in top left corner, like in 99% of drawing libs, it is
    // anchored in the middle of the square. This is done to simplify calculation of key points in
    // whole image, but makes this implementation a bit magic.
    // It can also be drawn as diamond, but then size is not the size of the diamond, but the size
    // of the square that would fit the diamond (the size is actually size of the diagonals of the
    // diamond).
    // Be warned.
    fn draw(&self, buffer: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) {
        let half_size = (self.size - 1) / 2;
        let start_x = self.x - half_size;
        let end_x = self.x + self.size / 2;
        let start_y = self.y - half_size;
        let end_y = self.y + self.size / 2;
        for x in start_x..=end_x {
            for y in start_y..=end_y {
                if self.pixel_belongs(x, y) {
                    buffer.put_pixel(
                        x,
                        y,
                        Rgba([
                            self.color.red,
                            self.color.green,
                            self.color.blue,
                            self.color.alpha,
                        ]),
                    );
                }
            }
        }
    }
}

#[derive(Debug)]
struct Rectangle {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: Srgba<u8>,
}

impl Rectangle {
    fn pixel_belongs(&self, x: u32, y: u32, max_width: u32, max_height: u32) -> bool {
        x >= self.x
            && x < self.x + self.width
            && y >= self.y
            && y < self.y + self.height
            && x < max_width
            && y < max_height
    }

    fn draw(&self, buffer: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) {
        let max_width = buffer.width();
        let max_height = buffer.height();

        for x in self.x..=self.x + self.width {
            for y in self.y..=self.y + self.height {
                if self.pixel_belongs(x, y, max_width, max_height) {
                    buffer.put_pixel(
                        x,
                        y,
                        Rgba([
                            self.color.red,
                            self.color.green,
                            self.color.blue,
                            self.color.alpha,
                        ]),
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
    color: Srgba<u8>,
}

impl Line {
    fn draw(&self, buffer: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) {
        // Decide which point is the starting point
        let (x1, x2, y1, y2) = match (self.x1, self.x2, self.y1, self.y2) {
            (x1, x2, y1, y2) if x1 > x2 => (x2 as i32, x1 as i32, y2 as i32, y1 as i32),
            (x1, x2, y1, y2) => (x1 as i32, x2 as i32, y1 as i32, y2 as i32),
        };

        let half_border = ((self.border_size - 1) / 2) as i32;

        let max_width = buffer.width() as i32;
        let max_height = buffer.height() as i32;

        let mut pixels = Vec::new();
        if x1 == x2 {
            for y in y1..y2 + 1 {
                pixels.push((x1, y));
            }
        } else {
            for x in x1..x2 + 1 {
                let y = y1 + (y2 - y1) * (x - x1) / (x2 - x1);
                pixels.push((x, y));
            }
        }

        for (x, y) in pixels {
            for i in -half_border..half_border + 1 {
                for j in -half_border..half_border + 1 {
                    if x + i < max_width && y + j < max_height && x + i >= 0 && y + j >= 0 {
                        buffer.put_pixel(
                            (x + i) as u32,
                            (y + j) as u32,
                            Rgba([
                                self.color.red,
                                self.color.green,
                                self.color.blue,
                                self.color.alpha,
                            ]),
                        );
                    }
                }
            }
        }
    }
}

enum ShapeType {
    Square(Square),
    Rectangle(Rectangle),
    Line(Line),
}
