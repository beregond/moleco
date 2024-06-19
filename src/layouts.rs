use crate::tokenize::{Capacity, Concentration, Content, Ingredient, Mixture};
use crate::Scheme;
use image::{ImageBuffer, Rgba};
use log::debug;
use palette::{FromColor, Hsv, Srgba};
use std::collections::HashMap;

macro_rules! vertical_square {
    ($shapes: expr, $size: expr, $point: expr, $color: expr) => {
        $shapes.push(Shape::Square(Square {
            x: $point.x,
            y: $point.y,
            size: $size,
            orientation: Orientation::Vertical,
            color: $color.into(),
        }));
    };
}

macro_rules! horizontal_square {
    ($shapes: expr, $size: expr, $point: expr, $color: expr) => {
        $shapes.push(Shape::Square(Square {
            x: $point.x,
            y: $point.y,
            size: $size,
            orientation: Orientation::Horizontal,
            color: $color.into(),
        }));
    };
}

macro_rules! line {
    ($shapes: expr, $start: expr, $end: expr, $size: expr, $color: expr) => {
        $shapes.push(Shape::Line(Line {
            x1: $start.x,
            y1: $start.y,
            x2: $end.x,
            y2: $end.y,
            border_size: $size,
            color: $color.into(),
        }));
    };
    ($shapes: expr, $start_x: expr, $start_y: expr, $end_x: expr, $end_y: expr, $size: expr, $color: expr) => {
        $shapes.push(Shape::Line(Line {
            x1: $start_x,
            y1: $start_y,
            x2: $end_x,
            y2: $end_y,
            border_size: $size,
            color: $color.into(),
        }));
    };
}

struct WidthsResult {
    widths: Vec<(String, f32)>,
    unestimated_capacity: bool,
}

pub struct Picture {
    base_size: u32,
    border_size: u32,
    schemes: Vec<Scheme>,
    // Indexing and concentration information combined into tree
    mixture_info: Option<Mixture>,
}

impl Picture {
    pub fn new(
        base_size: u32,
        border_size: u32,
        schemes: Vec<Scheme>,
        mixture_info: Option<Mixture>,
    ) -> Self {
        Self {
            base_size,
            border_size,
            schemes,
            mixture_info,
        }
    }

    pub fn generate(&mut self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let border_color: Srgba<u8> = Srgba::from_color(Hsv::new(0.0, 0.0, 0.1)).into_format();
        let eraser = Srgba::new(0, 0, 0, 0);
        let cell_size = self.base_size * 2 + self.border_size * 3;
        let width = cell_size * self.schemes.len() as u32
            - (self.schemes.len() as u32 - 1) * self.border_size;

        // Subtraction is required as generation is pixel perfect and
        // operates on odd sizes - subtraction assures we don't miss pixels in edge cases.
        let half_border = (self.border_size - 1) / 2;
        let half_size = (self.base_size - 1) / 2;
        let quarter_size = (half_size - 1) / 2;
        let eight_size = (quarter_size - 1) / 2;

        // Height is calculated based on the presence of mixture information.
        let height = match &self.mixture_info.is_some() {
            true => cell_size + half_size,
            false => cell_size,
        };

        // Layers and ordering initializatin. If no mixture information is present
        // layers are empty and ordering is just a sequence of indices.
        // Altering ordering is neat trick to improve readability of the generated image. See
        // readme for details.
        let mut layers: Vec<Vec<Shape>> = Vec::new();
        let ordering;

        // Lots of details (like ordering) depending on the presence of mixture information,
        // that's why all work is started from drawing the mixture bar first.
        match &self.mixture_info {
            Some(mixture) => {
                let widths = calculate_widths(&mixture.ingredients);
                debug!("Mixture basic widths: {:?}", widths.widths);

                let ordered_widths = calculate_ordered_widths(&self.schemes, widths);
                debug!("Mixture ordered widths: {:?}", ordered_widths);

                ordering = self._calculate_ordered_indices(Some(&ordered_widths));
                debug!("Mixture ordering: {:?}", ordering);

                let mut bar_layers: Vec<Shape> = Vec::new();
                let mut line_layers: Vec<Shape> = Vec::new();

                // Offset and width gets '-1' because from this time we are working on the actual pixels,
                // which are indexed from 0.
                self.draw_mixture_bar(
                    ordered_widths,
                    cell_size + quarter_size - 1,
                    0,
                    width - half_border - 1,
                    quarter_size,
                    true, //calculated_widths.unestimated_capacity,
                    &mut bar_layers,
                    &mut line_layers,
                );

                layers.push(bar_layers);
                layers.push(line_layers);
            }
            None => {
                ordering = self._calculate_ordered_indices(None);
            }
        };

        let mut buffer = ImageBuffer::new(width, height);

        let mut offset = 0;
        for index in ordering {
            self.draw_single_swatch(
                &mut layers,
                index,
                offset,
                &border_color,
                &eraser,
                &half_border,
                &half_size,
                &quarter_size,
                &eight_size,
            );

            offset += cell_size - self.border_size;
        }

        for layer in &layers {
            for shape in layer {
                match shape {
                    Shape::Square(square) => square.draw(&mut buffer),
                    Shape::Line(line) => line.draw(&mut buffer),
                    Shape::Rectangle(rectangle) => rectangle.draw(&mut buffer),
                }
            }
        }
        buffer
    }

    fn draw_single_swatch(
        &self,
        layers: &mut Vec<Vec<Shape>>,
        index: usize,
        offset: u32,
        border_color: &Srgba<u8>,
        eraser: &Srgba<u8>,
        half_border: &u32,
        half_size: &u32,
        quarter_size: &u32,
        eight_size: &u32,
    ) {
        // Now lets calculate key points in drawing
        //
        //             A
        //           /   \
        //         /       \
        //       B     C     D
        //     /   \       /   \
        //   /       \   /       \
        // E     F     G     H     I
        //   \       /   \       /
        //     \   /       \   /
        //       J     K     L
        //         \       /
        //           X   Y
        //             M

        let a = Point {
            x: offset + self.base_size + self.border_size + half_border,
            y: *half_border,
        };
        let b = Point {
            x: offset + half_size + self.border_size,
            y: half_size + self.border_size,
        };
        let c = Point {
            x: offset + self.base_size + self.border_size + half_border,
            y: half_size + self.border_size,
        };
        let d = Point {
            x: offset + self.base_size + half_size + self.border_size * 2,
            y: half_size + self.border_size,
        };
        let e = Point {
            x: offset + half_border,
            y: self.base_size + self.border_size + half_border,
        };
        let f = Point {
            x: offset + half_size + self.border_size,
            y: self.base_size + self.border_size + half_border,
        };
        let g = Point {
            x: offset + self.base_size + self.border_size + half_border,
            y: self.base_size + self.border_size + half_border,
        };
        let h = Point {
            x: offset + self.base_size + half_size + self.border_size * 2,
            y: self.base_size + self.border_size + half_border,
        };
        let i = Point {
            x: offset + self.base_size * 2 + self.border_size * 2 + half_border,
            y: self.base_size + self.border_size + half_border,
        };
        let j = Point {
            x: offset + half_size + self.border_size,
            y: self.base_size + half_size + self.border_size * 2,
        };
        let k = Point {
            x: offset + self.base_size + self.border_size + half_border,
            y: self.base_size + half_size + self.border_size * 2,
        };
        let l = Point {
            x: offset + self.base_size + half_size + self.border_size * 2,
            y: self.base_size + half_size + self.border_size * 2,
        };
        let m = Point {
            x: offset + self.base_size + self.border_size + half_border,
            y: self.base_size * 2 + self.border_size * 2 + half_border,
        };
        let x = Point {
            x: j.x + (m.x - j.x) * 2 / 3,
            y: j.y + (m.y - j.y) * 2 / 3,
        };
        let y = Point {
            x: m.x + (l.x - m.x) / 3,
            y: x.y,
        };

        let scheme = &self.schemes[index];
        let mut base_colors: Vec<Shape> = Vec::new();
        // Primary color
        vertical_square!(base_colors, self.base_size, c, scheme.primary.srgb);
        // First accent
        vertical_square!(base_colors, self.base_size, f, scheme.first_accent.srgb);
        // Second accent
        vertical_square!(base_colors, self.base_size, h, scheme.second_accent.srgb);
        // Complementary color
        vertical_square!(base_colors, self.base_size, k, scheme.complementary.srgb);
        layers.push(base_colors);

        let mut base_lines: Vec<Shape> = Vec::new();
        // Cross - left top to right bottom
        line!(base_lines, b, l, self.border_size, *border_color);
        // Cross - left bottom to right top
        line!(base_lines, j, d, self.border_size, *border_color);

        // Border - top left
        line!(base_lines, e, a, self.border_size, *border_color);
        // Border - top right
        line!(base_lines, a, i, self.border_size, *border_color);
        // Border - bottom right
        line!(base_lines, i, m, self.border_size, *border_color);
        // Border - bottom left
        line!(base_lines, m, e, self.border_size, *border_color);
        layers.push(base_lines);

        let mut cutouts: Vec<Shape> = Vec::new();
        // Left top cutout
        horizontal_square!(cutouts, *quarter_size, b, *eraser);
        // Right top cutout
        horizontal_square!(cutouts, *quarter_size, d, *eraser);
        // Left bottom cutout
        horizontal_square!(cutouts, *quarter_size, j, *eraser);
        // Right bottom cutout
        horizontal_square!(cutouts, *quarter_size, l, *eraser);
        // Central cutout
        horizontal_square!(cutouts, *quarter_size, g, *eraser);
        layers.push(cutouts);

        // Aliases, to fit declarative code in single lines
        let es = eight_size;
        let size = self.border_size;
        let color = border_color;

        let mut lines: Vec<Shape> = Vec::new();
        // Left top cutout borders
        line!(lines, b.x + es, b.y - es, b.x + es, b.y + es, size, *color);
        line!(lines, b.x - es, b.y + es, b.x + es, b.y + es, size, *color);

        // Right top cutout borders
        line!(lines, d.x - es, d.y - es, d.x - es, d.y + es, size, *color);
        line!(lines, d.x - es, d.y + es, d.x + es, d.y + es, size, *color);

        // Left bottom cutout borders
        line!(lines, j.x - es, j.y - es, j.x + es, j.y - es, size, *color);
        line!(lines, j.x + es, j.y - es, j.x + es, j.y + es, size, *color);

        // Right bottom cutout borders
        line!(lines, l.x - es, l.y - es, l.x + es, l.y - es, size, *color);
        line!(lines, l.x - es, l.y - es, l.x - es, l.y + es, size, *color);

        // Central cutout borders
        line!(lines, g.x - es, g.y - es, g.x + es, g.y - es, size, *color);
        line!(lines, g.x - es, g.y - es, g.x - es, g.y + es, size, *color);
        line!(lines, g.x - es, g.y + es, g.x + es, g.y + es, size, *color);
        line!(lines, g.x + es, g.y - es, g.x + es, g.y + es, size, *color);

        // Orientation mark
        line!(lines, x, y, size, *color);

        layers.push(lines);
    }

    /// Normalize indices order. During mixture bar calculation indices may be reordered to improve readability.
    /// This function either parses what was calculated, or in case of no mixture information - returns
    /// initial order, as passed through the constructor.
    fn _calculate_ordered_indices(
        &self,
        ordered_widths: Option<&Vec<(String, f32)>>,
    ) -> Vec<usize> {
        match ordered_widths {
            None => {
                let mut indices: Vec<usize> = Vec::new();
                for i in 0..self.schemes.len() {
                    indices.push(i);
                }
                indices
            }
            Some(value) => {
                let mut ordered_indices: Vec<usize> = vec![];
                for (index, _) in value {
                    match index.parse::<usize>() {
                        Ok(value) => ordered_indices.push(value - 1),
                        Err(_) => {}
                    }
                }
                ordered_indices
            }
        }
    }

    fn draw_mixture_bar(
        &self,
        widths: Vec<(String, f32)>,
        y_offset: u32,
        start_x: u32,
        end_x: u32,
        base_bar_size: u32,
        unestimated_capacity: bool,
        bar_layers: &mut Vec<Shape>,
        line_layers: &mut Vec<Shape>,
    ) {
        let available_width = end_x - start_x;
        let mut indices: Vec<String> = vec![];
        let mut sizes: Vec<f32> = vec![];
        let mut unknown_substance_present = false;
        for (index, width) in &widths {
            if *width == 0f32 {
                continue;
            }
            indices.push(index.clone());
            sizes.push(width.clone());

            if index == "" {
                unknown_substance_present = true;
            }
        }

        // Streching sizes so ln values will be bigger than 10
        while sizes
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
            < &10f32
        {
            sizes = sizes.iter().map(|s| s * 10f32).collect();
        }

        // If the capacity is unestimated - we need to have unknown substance present to indicate
        // this. It may happen that unknown substance is already present, but if not - we need to
        // add a bit of space for it.
        if unestimated_capacity && !unknown_substance_present {
            indices.push("".to_string());
            // Chosen by fair dice roll.
            sizes.push(4f32);
        }

        let ln_sizes = sizes.iter().map(|s| s.ln()).collect::<Vec<f32>>();

        debug!("Mixture sizes after logarithm: {:?}", ln_sizes);

        let ln_sum = ln_sizes.iter().sum::<f32>();

        let mut actual_sizes = ln_sizes
            .iter()
            .map(|s| ((s / ln_sum * available_width as f32) as u32))
            .collect::<Vec<u32>>();

        // Streching, to reclaim pixels lost on rounding
        let mut sizes_sum_diff: isize =
            available_width as isize - actual_sizes.iter().sum::<u32>() as isize;
        while sizes_sum_diff > 0 {
            for i in 0..sizes.len() {
                if sizes_sum_diff == 0 {
                    break;
                }
                actual_sizes[i] += 1;
                sizes_sum_diff -= 1;
            }
        }
        // Shrinking, to remove excess pixels
        while sizes_sum_diff < 0 {
            for i in 0..sizes.len() {
                if sizes_sum_diff == 0 {
                    break;
                }
                actual_sizes[i] -= 1;
                sizes_sum_diff += 1;
            }
        }

        debug!("Mixture actual sizes: {:?}", actual_sizes);

        line_layers.push(Shape::Line(Line {
            x1: start_x,
            y1: y_offset,
            x2: end_x,
            y2: y_offset,
            border_size: self.border_size,
            color: Srgba::from_color(Hsv::new(0.0, 0.0, 0.1)).into_format(),
        }));
        line_layers.push(Shape::Line(Line {
            x1: start_x,
            y1: y_offset,
            x2: start_x,
            y2: y_offset + base_bar_size,
            border_size: self.border_size,
            color: Srgba::from_color(Hsv::new(0.0, 0.0, 0.1)).into_format(),
        }));
        line_layers.push(Shape::Line(Line {
            x1: start_x,
            y1: y_offset + base_bar_size,
            x2: end_x,
            y2: y_offset + base_bar_size,
            border_size: self.border_size,
            color: Srgba::from_color(Hsv::new(0.0, 0.0, 0.1)).into_format(),
        }));

        let mut start = start_x;
        let mut end = start_x;
        let mut step_index = 0;

        for substance_index in indices {
            end += actual_sizes[step_index];

            let color = match substance_index.parse::<usize>() {
                Ok(value) => self.schemes[value - 1].primary.srgb.into(),
                Err(_) => Srgba::from_color(Hsv::new(0.0, 0.0, 0.8)).into_format(),
            };
            bar_layers.push(Shape::Rectangle(Rectangle {
                x: start,
                y: y_offset,
                width: end - start,
                height: base_bar_size,
                color,
            }));

            start = end;
            step_index += 1;
            line_layers.push(Shape::Line(Line {
                x1: start,
                y1: y_offset,
                x2: start,
                y2: y_offset + base_bar_size,
                border_size: self.border_size,
                color: Srgba::from_color(Hsv::new(0.0, 0.0, 0.1)).into_format(),
            }));
        }

        // Indicate "open" end of the bar.
        if unestimated_capacity {
            // Remove last vertical line.
            line_layers.pop();

            let half_height = (base_bar_size - 1) / 2;
            let third_height = base_bar_size / 3;
            let sixth_height = third_height / 2;

            bar_layers.push(Shape::Square(Square {
                x: end_x,
                y: y_offset + sixth_height,
                size: third_height,
                orientation: Orientation::Vertical,
                color: Srgba::new(0, 0, 0, 0),
            }));

            bar_layers.push(Shape::Square(Square {
                x: end_x,
                y: y_offset + half_height,
                size: third_height,
                orientation: Orientation::Vertical,
                color: Srgba::new(0, 0, 0, 0),
            }));

            bar_layers.push(Shape::Square(Square {
                x: end_x,
                y: y_offset + half_height + third_height,
                size: third_height,
                orientation: Orientation::Vertical,
                color: Srgba::new(0, 0, 0, 0),
            }));

            line_layers.push(Shape::Line(Line {
                x1: end_x,
                y1: y_offset,
                x2: end_x - sixth_height,
                y2: y_offset + sixth_height,
                border_size: self.border_size,
                color: Srgba::from_color(Hsv::new(0.0, 0.0, 0.1)).into_format(),
            }));

            line_layers.push(Shape::Line(Line {
                x1: end_x - sixth_height,
                y1: y_offset + sixth_height,
                x2: end_x,
                y2: y_offset + third_height,
                border_size: self.border_size,
                color: Srgba::from_color(Hsv::new(0.0, 0.0, 0.1)).into_format(),
            }));

            line_layers.push(Shape::Line(Line {
                x1: end_x,
                y1: y_offset + third_height,
                x2: end_x - sixth_height,
                y2: y_offset + half_height,
                border_size: self.border_size,
                color: Srgba::from_color(Hsv::new(0.0, 0.0, 0.1)).into_format(),
            }));

            line_layers.push(Shape::Line(Line {
                x1: end_x - sixth_height,
                y1: y_offset + half_height,
                x2: end_x,
                y2: y_offset + half_height + sixth_height,
                border_size: self.border_size,
                color: Srgba::from_color(Hsv::new(0.0, 0.0, 0.1)).into_format(),
            }));

            line_layers.push(Shape::Line(Line {
                x1: end_x,
                y1: y_offset + half_height + sixth_height,
                x2: end_x - sixth_height,
                y2: y_offset + half_height + third_height,
                border_size: self.border_size,
                color: Srgba::from_color(Hsv::new(0.0, 0.0, 0.1)).into_format(),
            }));

            line_layers.push(Shape::Line(Line {
                x1: end_x - sixth_height,
                y1: y_offset + half_height + third_height,
                x2: end_x,
                y2: y_offset + base_bar_size,
                border_size: self.border_size,
                color: Srgba::from_color(Hsv::new(0.0, 0.0, 0.1)).into_format(),
            }));
        }
    }
}

// TODO test this?
fn calculate_ordered_widths(
    schemes: &Vec<Scheme>,
    calculated_widths: WidthsResult,
) -> Vec<(String, f32)> {
    let mut sums: HashMap<String, f32> = HashMap::new();

    // Sum up widths for each index, while checking if the index is valid
    for (index, width) in calculated_widths.widths {
        let actual_index = match index.parse::<usize>() {
            Ok(value) => match schemes.get(value - 1) {
                Some(_) => index,
                None => {
                    debug!("Unknown index: '{}'", index);
                    "".to_string()
                }
            },
            Err(_) => {
                debug!("Unknown index: '{}'", index);
                "".to_string()
            }
        };

        let sum = sums.entry(actual_index).or_insert(0f32);
        *sum += width;
    }

    let mut empty_keys: Vec<String> = vec![];
    for (key, value) in sums.iter() {
        if *value <= 0f32 {
            empty_keys.push(key.clone());
        }
    }

    let mut ordered_widths_map: HashMap<String, f32> = HashMap::new();
    let mut empty_map: HashMap<String, f32> = HashMap::new();
    let mut unknown: Option<(String, f32)> = None;

    for (key, value) in sums.iter() {
        if key == "" {
            unknown = Some((key.clone(), *value));
        } else if *value == 0f32 {
            empty_map.insert(key.clone(), *value);
        } else {
            ordered_widths_map.insert(key.clone(), *value);
        }
    }

    let mut ordered_widths: Vec<(String, f32)> = vec![];
    let mut empty: Vec<(String, f32)> = vec![];

    // Key is not equal index, beware off-by-one error.
    for key in 1..=schemes.len() {
        let str_key = key.to_string();
        match ordered_widths_map.get(&str_key) {
            Some(value) => ordered_widths.push((str_key, *value)),
            None => {}
        }
    }

    // Same as above.
    for key in 1..=schemes.len() {
        let str_key = key.to_string();
        match empty_map.get(&str_key) {
            Some(value) => empty.push((str_key, *value)),
            None => {}
        }
    }

    for (key, value) in empty {
        ordered_widths.push((key, value));
    }

    if let Some((key, value)) = unknown {
        ordered_widths.push((key, value));
    }

    ordered_widths
}

// TODO test this
/// Calculate widths of components in the mixture in percent.
/// Result is denormalized and (un)estimated, so it is just base for later calculations.
fn calculate_widths(components: &Vec<Ingredient>) -> WidthsResult {
    let mut unestimated_capacity = false;
    let mut magnitudes: Vec<isize> = vec![];
    let mut concentrations: Vec<&Concentration> = vec![];
    let mut unknown = 0;
    for component in components {
        match component {
            Ingredient::Mixture(mixture) => match &mixture.content {
                Some(content) => {
                    concentrations.push(&content.concentration);
                    magnitudes.push(content.magnitude);
                }
                None => {
                    unknown += 1;
                }
            },
            Ingredient::Substance(substance) => match &substance.content {
                Some(content) => {
                    concentrations.push(&content.concentration);
                    magnitudes.push(content.magnitude);
                }
                None => {
                    unknown += 1;
                }
            },
        }
    }

    let mut seen_concentrations: Vec<&Concentration> = vec![];
    for concentration in &concentrations {
        if seen_concentrations.contains(concentration) {
            continue;
        }
        seen_concentrations.push(concentration);
    }
    if seen_concentrations.len() > 1 {
        panic!(
            "Different concentrations in one mixture, only one is allowed - {:?}",
            seen_concentrations
        );
    }
    // TODO Where is check if there is at least one?
    let concentration = seen_concentrations[0];

    match Content::maximum_viable_magnitude(concentration) {
        Some(max_level) => {
            magnitudes.push(max_level);
        }
        None => {}
    }

    let min_magnitude = magnitudes.iter().min().unwrap();

    let mut values: Vec<usize> = vec![];
    for component in components {
        match component {
            Ingredient::Mixture(mixture) => match &mixture.content {
                Some(content) => values.push(content.value_at_magnitude(min_magnitude)),
                None => {}
            },
            Ingredient::Substance(substance) => match &substance.content {
                Some(content) => values.push(content.value_at_magnitude(min_magnitude)),
                None => {}
            },
        }
    }
    let sum = values.iter().sum::<usize>();
    let capacity = match Content::calculate_capacity(concentration, min_magnitude) {
        Capacity::Absolute(capacity) => capacity,
        Capacity::Relative => sum,
        Capacity::Unestimated => {
            // As long as this app is not calculating molar mass/volume - it is always
            // impossible to actually know proportions of mixture.
            unestimated_capacity = true;
            sum
        }
    };

    debug!("capacity: {}", capacity);
    debug!("min_magnitude: {}", min_magnitude);
    debug!("values: {:?}", values);
    debug!("sum: {}", sum);
    let mut default_width = 0f32;
    let mut result: Vec<(String, f32)> = vec![];
    match (unknown, sum, capacity) {
        // If taken capacity is more than 100% - every substance without content specified will
        // be treated as addition with **no representation** in the bar. (Actually its size
        // will simply be 0).
        (_, s, c) if s >= c => {}
        // There is some volume left for known SINGLE substance, lets assign it to that substance.
        (u, s, c) if u == 1 && s < c => {
            default_width = (c - s) as f32 / u as f32;
        }
        // There is some unknown substance(s) with known volume OR there are more than one
        // known substances, lets append unknown substance do the end of the bar as indication.
        // In case of multiple known substances - let's not try to guess - treat this as not
        // provided.
        (u, s, c) if u != 1 && s < c => {
            result.push(("".to_string(), (c - s) as f32 / s as f32));
        }
        (_, _, _) => {
            panic!("Unknown case");
        }
    }
    let sum = sum as f32;

    // Final, recursive calculation of widths for each component.
    for component in components {
        match component {
            Ingredient::Mixture(mixture) => {
                let size = match &mixture.content {
                    Some(content) => content.value_at_magnitude(min_magnitude) as f32,
                    None => default_width,
                };
                let size = size / sum;
                let calculated = calculate_widths(&mixture.ingredients);
                for (index, width) in calculated.widths {
                    result.push((index, width * size));
                }
                if calculated.unestimated_capacity {
                    unestimated_capacity = true;
                }
            }
            Ingredient::Substance(substance) => {
                let size = match &substance.content {
                    Some(content) => content.value_at_magnitude(min_magnitude) as f32,
                    None => default_width,
                };
                let index = match &substance.index {
                    Some(index) => index.clone(),
                    None => "".to_string(),
                };
                result.push((index, size / sum as f32));
            }
        }
    }
    WidthsResult {
        widths: result,
        unestimated_capacity,
    }
}

#[derive(Debug)]
struct Point {
    x: u32,
    y: u32,
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

    // Instead of square being anchored in top left corner, like in 99% of drawing libs, it is
    // anchored in the middle of the square. This is done to simplify calculation of key points in
    // whole image, but makes this implementation a bit magic.
    // It can also be drawn as diamond, but then size is not the size of the side of the diamond, but the size
    // of the square that would fit the diamond (the size is actually size of its diagonals).
    //
    // Be warned.
    fn draw(&self, buffer: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) {
        let max_width = buffer.width();
        let max_height = buffer.height();

        let half_size = (self.size - 1) / 2;
        let start_x = self.x - half_size;
        let end_x = self.x + self.size / 2;
        let start_y = self.y - half_size;
        let end_y = self.y + self.size / 2;
        for x in start_x..=end_x {
            for y in start_y..=end_y {
                if self.pixel_belongs(x, y) {
                    if x < max_width && y < max_height {
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

enum Shape {
    Square(Square),
    Rectangle(Rectangle),
    Line(Line),
}
