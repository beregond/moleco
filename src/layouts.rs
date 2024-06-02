use crate::tokenize::{
    generate_compound_tree, Capacity, Compound, Concentration, Content, Ingredient,
};
use crate::Scheme;
use image::{ImageBuffer, Rgba};
use log::trace;
use palette::{FromColor, Hsv, Srgba};
use std::collections::HashMap;

struct WidthsResult {
    widths: Vec<(String, f32)>,
    unestimated_capacity: bool,
}

// TODO no pub?
pub struct Picture {
    base_size: u32,
    border_size: u32,
    schemes: Vec<Scheme>,
    // Indexing and concentration information combined into tree
    compound_info: Option<(String, String)>,
    // TODO add _ to this?
    scheme_ordering: Option<Vec<usize>>,
}

impl Picture {
    pub fn new(
        base_size: u32,
        border_size: u32,
        schemes: Vec<Scheme>,
        compound_info: Option<(String, String)>,
    ) -> Self {
        Self {
            base_size,
            border_size,
            schemes,
            compound_info,
            scheme_ordering: None,
        }
    }

    // TODO: This does not belong here
    fn generate_compound_tree(&self) -> Option<Compound> {
        match &self.compound_info {
            None => None,
            Some((indexing, concentration)) => {
                Some(generate_compound_tree(indexing, concentration))
            }
        }
    }

    pub fn generate(&mut self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let border_color: Srgba<u8> = Srgba::from_color(Hsv::new(0.0, 0.0, 0.1)).into_format();
        let eraser = Srgba::new(0, 0, 0, 0);
        let cell_size = self.base_size * 2 + self.border_size * 3;
        let width = cell_size * self.schemes.len() as u32
            - (self.schemes.len() as u32 - 1) * self.border_size;

        // Note to myself - subtraction is required as generation is pixel perfect and
        // operates on odd sizes - subtraction assures we don't miss pixels in edge cases.
        let half_border = (self.border_size - 1) / 2;
        let half_size = (self.base_size - 1) / 2;
        let quarter_size = (half_size - 1) / 2;
        let eight_size = (quarter_size - 1) / 2;

        trace!("border_size: {}", self.border_size);
        trace!("half_size: {}", half_border);

        let mut height = cell_size;
        let mut layers: Vec<Vec<Shape>> = Vec::new();
        if let Some(compound) = self.generate_compound_tree() {
            height += quarter_size * 2;
            let y_offset = cell_size + quarter_size;
            layers = self.draw_compound_bar(compound, width - half_border, y_offset, quarter_size);
        }

        let mut buffer = ImageBuffer::new(width, height);
        let mut offset = 0;

        let ordering = match &self.scheme_ordering {
            Some(ordering) => ordering.clone(),
            None => {
                let mut ord: Vec<usize> = Vec::new();
                for i in 0..self.schemes.len() {
                    ord.push(i);
                }
                ord
            }
        };

        for index in ordering {
            let scheme = &self.schemes[index];
            let primary = scheme.primary.srgb;
            let first_accent = scheme.first_accent.srgb;
            let second_accent = scheme.second_accent.srgb;
            let complementary = scheme.complementary.srgb;
            let mut base_color: Vec<Shape> = Vec::new();

            base_color.push(Shape::Square(Square {
                x: offset + self.base_size + self.border_size + half_border,
                y: half_size + self.border_size,
                size: self.base_size,
                orientation: Orientation::Vertical,
                color: primary.into(),
            }));
            base_color.push(Shape::Square(Square {
                x: offset + half_size + self.border_size,
                y: self.base_size + self.border_size + half_border,
                size: self.base_size,
                orientation: Orientation::Vertical,
                color: first_accent.into(),
            }));
            base_color.push(Shape::Square(Square {
                x: offset + self.base_size + half_size + self.border_size * 2,
                y: self.base_size + self.border_size + half_border,
                size: self.base_size,
                orientation: Orientation::Vertical,
                color: second_accent.into(),
            }));
            base_color.push(Shape::Square(Square {
                x: offset + self.base_size + self.border_size + half_border,
                y: self.base_size + half_size + self.border_size * 2,
                size: self.base_size,
                orientation: Orientation::Vertical,
                color: complementary.into(),
            }));
            layers.push(base_color);

            let mut base_lines: Vec<Shape> = Vec::new();
            // Cross - left top to right bottom
            base_lines.push(Shape::Line(Line {
                x1: offset + half_size + self.border_size,
                y1: half_size + self.border_size,
                x2: offset + self.base_size + half_size + self.border_size * 2,
                y2: self.base_size + half_size + self.border_size * 2,
                border_size: self.border_size,
                color: border_color,
            }));
            // Cross - left bottom to right top
            base_lines.push(Shape::Line(Line {
                x1: offset + half_size + self.border_size,
                y1: self.base_size + half_size + self.border_size * 2,
                x2: offset + self.base_size + half_size + self.border_size * 2,
                y2: half_size + self.border_size,
                border_size: self.border_size,
                color: border_color,
            }));
            base_lines.push(Shape::Line(Line {
                x1: offset + half_border,
                y1: self.base_size + self.border_size + half_border,
                x2: offset + self.base_size + self.border_size + half_border,
                y2: half_border,
                border_size: self.border_size,
                color: border_color,
            }));
            base_lines.push(Shape::Line(Line {
                x1: offset + self.base_size + self.border_size + half_border,
                y1: half_border,
                x2: offset + self.base_size * 2 + self.border_size * 2 + half_border,
                y2: self.base_size + self.border_size + half_border,
                border_size: self.border_size,
                color: border_color,
            }));
            base_lines.push(Shape::Line(Line {
                x1: offset + self.base_size * 2 + self.border_size * 2 + half_border,
                y1: self.base_size + self.border_size + half_border,
                x2: offset + self.base_size + self.border_size + half_border,
                y2: self.base_size * 2 + self.border_size * 2 + half_border,
                border_size: self.border_size,
                color: border_color,
            }));
            base_lines.push(Shape::Line(Line {
                x1: offset + half_border,
                y1: self.base_size + self.border_size + half_border,
                x2: offset + self.base_size + self.border_size + half_border,
                y2: self.base_size * 2 + self.border_size * 2 + half_border,
                border_size: self.border_size,
                color: border_color,
            }));
            layers.push(base_lines);

            let mut cutouts: Vec<Shape> = Vec::new();
            // Central cutout
            cutouts.push(Shape::Square(Square {
                x: offset + self.base_size + self.border_size + half_border,
                y: self.base_size + self.border_size + half_border,
                size: quarter_size,
                orientation: Orientation::Horizontal,
                color: eraser,
            }));
            // Left top cutout
            cutouts.push(Shape::Square(Square {
                x: offset + half_size + self.border_size,
                y: half_size + self.border_size,
                size: quarter_size,
                orientation: Orientation::Horizontal,
                color: eraser,
            }));
            // Right bottom cutout
            cutouts.push(Shape::Square(Square {
                x: offset + self.base_size + half_size + self.border_size * 2,
                y: self.base_size + half_size + self.border_size * 2,
                size: quarter_size,
                orientation: Orientation::Horizontal,
                color: eraser,
            }));
            // Left bottom cutout
            cutouts.push(Shape::Square(Square {
                x: offset + half_size + self.border_size,
                y: self.base_size + half_size + self.border_size * 2,
                size: quarter_size,
                orientation: Orientation::Horizontal,
                color: eraser,
            }));
            // Right top cutout
            cutouts.push(Shape::Square(Square {
                x: offset + self.base_size + half_size + self.border_size * 2,
                y: half_size + self.border_size,
                size: quarter_size,
                orientation: Orientation::Horizontal,
                color: eraser,
            }));

            layers.push(cutouts);
            let mut inner_border: Vec<Shape> = Vec::new();
            // Left top cutout borders
            inner_border.push(Shape::Line(Line {
                x1: offset + half_size + self.border_size + eight_size,
                y1: half_size + self.border_size - eight_size,
                x2: offset + half_size + self.border_size + eight_size,
                y2: half_size + self.border_size + eight_size,
                border_size: self.border_size,
                color: border_color,
            }));
            inner_border.push(Shape::Line(Line {
                x1: offset + half_size + self.border_size - eight_size,
                y1: half_size + self.border_size + eight_size,
                x2: offset + half_size + self.border_size + eight_size,
                y2: half_size + self.border_size + eight_size,
                border_size: self.border_size,
                color: border_color,
            }));

            // Right bottom cutout borders
            inner_border.push(Shape::Line(Line {
                x1: offset + self.base_size + half_size + self.border_size * 2 - eight_size,
                y1: self.base_size + half_size + self.border_size * 2 - eight_size,
                x2: offset + self.base_size + half_size + self.border_size * 2 + eight_size,
                y2: self.base_size + half_size + self.border_size * 2 - eight_size,
                border_size: self.border_size,
                color: border_color,
            }));
            inner_border.push(Shape::Line(Line {
                x1: offset + self.base_size + half_size + self.border_size * 2 - eight_size,
                y1: self.base_size + half_size + self.border_size * 2 - eight_size,
                x2: offset + self.base_size + half_size + self.border_size * 2 - eight_size,
                y2: self.base_size + half_size + self.border_size * 2 + eight_size,
                border_size: self.border_size,
                color: border_color,
            }));

            // Left bottom cutout borders
            inner_border.push(Shape::Line(Line {
                x1: offset + half_size + self.border_size - eight_size,
                y1: self.base_size + half_size + self.border_size * 2 - eight_size,
                x2: offset + half_size + self.border_size + eight_size,
                y2: self.base_size + half_size + self.border_size * 2 - eight_size,
                border_size: self.border_size,
                color: border_color,
            }));
            inner_border.push(Shape::Line(Line {
                x1: offset + half_size + self.border_size + eight_size,
                y1: self.base_size + half_size + self.border_size * 2 - eight_size,
                x2: offset + half_size + self.border_size + eight_size,
                y2: self.base_size + half_size + self.border_size * 2 + eight_size,
                border_size: self.border_size,
                color: border_color,
            }));

            // Right top cutout borders
            inner_border.push(Shape::Line(Line {
                x1: offset + self.base_size + half_size + self.border_size * 2 - eight_size,
                y1: half_size + self.border_size - eight_size,
                x2: offset + self.base_size + half_size + self.border_size * 2 - eight_size,
                y2: half_size + self.border_size + eight_size,
                border_size: self.border_size,
                color: border_color,
            }));
            inner_border.push(Shape::Line(Line {
                x1: offset + self.base_size + half_size + self.border_size * 2 - eight_size,
                y1: half_size + self.border_size + eight_size,
                x2: offset + self.base_size + half_size + self.border_size * 2 + eight_size,
                y2: half_size + self.border_size + eight_size,
                border_size: self.border_size,
                color: border_color,
            }));

            // Central cutout borders
            inner_border.push(Shape::Line(Line {
                x1: offset + self.base_size + self.border_size + half_border - eight_size,
                y1: self.base_size + self.border_size + half_border - eight_size,
                x2: offset + self.base_size + self.border_size + half_border + eight_size,
                y2: self.base_size + self.border_size + half_border - eight_size,
                border_size: self.border_size,
                color: border_color,
            }));
            inner_border.push(Shape::Line(Line {
                x1: offset + self.base_size + self.border_size + half_border - eight_size,
                y1: self.base_size + self.border_size + half_border - eight_size,
                x2: offset + self.base_size + self.border_size + half_border - eight_size,
                y2: self.base_size + self.border_size + half_border + eight_size,
                border_size: self.border_size,
                color: border_color,
            }));
            inner_border.push(Shape::Line(Line {
                x1: offset + self.base_size + self.border_size + half_border - eight_size,
                y1: self.base_size + self.border_size + half_border + eight_size,
                x2: offset + self.base_size + self.border_size + half_border + eight_size,
                y2: self.base_size + self.border_size + half_border + eight_size,
                border_size: self.border_size,
                color: border_color,
            }));
            inner_border.push(Shape::Line(Line {
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
                    Shape::Square(square) => square.draw(&mut buffer),
                    Shape::Line(line) => line.draw(&mut buffer),
                    Shape::Rectangle(rectangle) => rectangle.draw(&mut buffer),
                }
            }
        }
        buffer
    }

    fn draw_compound_bar(
        &mut self,
        compound: Compound,
        width: u32,
        y_offset: u32,
        base_bar_size: u32,
    ) -> Vec<Vec<Shape>> {
        let mut layers: Vec<Vec<Shape>> = Vec::new();
        let mut bar_layers: Vec<Shape> = Vec::new();
        let mut line_layers: Vec<Shape> = Vec::new();

        let calculated_widths = self.calculate_widths(&compound.ingredients);
        trace!("calc: {:?}", calculated_widths.widths);

        let mut sums: HashMap<String, f32> = HashMap::new();

        // Sum up widths for each index, while checking if the index is valid
        for (index, width) in calculated_widths.widths {
            let actual_index = match index.parse::<usize>() {
                Ok(value) => match self.schemes.get(value - 1) {
                    Some(_) => index,
                    None => {
                        trace!("Unknown index: {}", index);
                        "".to_string()
                    }
                },
                Err(_) => {
                    trace!("Unknown index: {}", index);
                    "".to_string()
                }
            };

            let sum = sums.entry(actual_index).or_insert(0f32);
            *sum += width;
        }
        trace!("sums: {:?}", sums);

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

        for key in 0..self.schemes.len() {
            // FIXME again off by one, should be done only once
            let str_key = (key + 1).to_string();
            match ordered_widths_map.get(&str_key) {
                Some(value) => ordered_widths.push((str_key, *value)),
                None => {}
            }
        }

        for key in 0..self.schemes.len() {
            let str_key = (key + 1).to_string();
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

        trace!("organized widths: {:?}", ordered_widths);

        let mut ordered_indices: Vec<usize> = vec![];
        for (index, _) in &ordered_widths {
            match index.parse::<usize>() {
                // #FIXME: Im subtracting this in two places, should be done only once
                Ok(value) => ordered_indices.push(value - 1),
                Err(_) => {}
            }
        }

        self.scheme_ordering = Some(ordered_indices);

        // Offset and width gets '-1' because from this time we are working on the actual pixels,
        // which are indexed from 0.
        self.draw_components(
            ordered_widths,
            y_offset - 1,
            0,
            width - 1,
            base_bar_size,
            calculated_widths.unestimated_capacity,
            &mut bar_layers,
            &mut line_layers,
        );

        layers.push(bar_layers);
        layers.push(line_layers);
        layers
    }

    fn draw_components(
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

        trace!("entry_sizes: {:?}", sizes);

        let ln_sizes = sizes.iter().map(|s| s.ln()).collect::<Vec<f32>>();

        trace!("ln sizes: {:?}", ln_sizes);

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

        trace!("actual_sizes: {:?}", actual_sizes);

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

    fn calculate_widths(&self, components: &Vec<Ingredient>) -> WidthsResult {
        let mut unestimated_capacity = false;
        let mut magnitudes: Vec<isize> = vec![];
        let mut concentrations: Vec<&Concentration> = vec![];
        let mut unknown = 0;
        for component in components {
            match component {
                Ingredient::Compound(compound) => match &compound.content {
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
                Ingredient::Compound(compound) => match &compound.content {
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

        trace!("capacity: {}", capacity);
        trace!("min_magnitude: {}", min_magnitude);
        trace!("values: {:?}", values);
        trace!("sum: {}", sum);
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
                Ingredient::Compound(compound) => {
                    let size = match &compound.content {
                        Some(content) => content.value_at_magnitude(min_magnitude) as f32,
                        None => default_width,
                    };
                    let size = size / sum;
                    let calculated = self.calculate_widths(&compound.ingredients);
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
