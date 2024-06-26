use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Group {
    pub components: Vec<Component>,
    pub value: Option<String>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Token {
    pub value: String,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Component {
    Token(Token),
    Group(Group),
}

macro_rules! parse_result {
    ($result:expr, $payload:expr) => {
        match $result {
            Ok(v) => Ok(v),
            Err(_) => Err(format!(
                "Invalid content notation, invalid value - {:?}",
                $payload
            )),
        }
    };
}

pub fn tokenize_string(input: &str, start: char) -> Result<Group, String> {
    if input.is_empty() {
        return Err("Empty group passed to tokenization".to_string());
    }
    // Prefix removal
    let mut iter = input.chars();
    if iter.next().unwrap() != start {
        return Err(format!(
            "Wrong first character, expected {} in input {}",
            start, input
        ));
    }
    let mut new_input: String = iter.collect();

    // Check if first and last character is '{' and '}'
    let len = new_input.len();
    let ends_with_paren = match len {
        l if l > 0 => match new_input.get(len - 1..) {
            Some("}") => true,
            _ => false,
        },
        _ => false,
    };
    let mut iter = new_input.chars().peekable();
    let starts_with_paren = match iter.peek() {
        Some(&'{') => true,
        _ => false,
    };

    // Check if parentheses are matching
    // Also check if the first group is covering the entire payload (and thus is obsolete)
    let mut first_group_is_covering_entire_payload = true;
    let mut level_indicator = 0;
    while let Some(&c) = iter.peek() {
        let mut decreased = false;
        match c {
            '{' => {
                level_indicator += 1;
            }
            '}' => {
                level_indicator -= 1;
                decreased = true;
            }
            _ => {}
        }
        if level_indicator < 0 {
            return Err(format!("Unmatching parentheses in input {}", input));
        }
        iter.next();
        // Check if first group closed before the end of the payload
        if decreased && level_indicator == 0 && iter.peek().is_some() {
            first_group_is_covering_entire_payload = false;
        }
    }
    if level_indicator != 0 {
        return Err(format!("Unmatching parentheses in input {}", input));
    }

    // Remove the first and last character if they are '{' and '}' and the group is covering the entire payload
    if starts_with_paren && ends_with_paren && first_group_is_covering_entire_payload {
        new_input = new_input.get(1..len - 1).unwrap().to_string();
    }

    let mut iter = new_input.chars().peekable();
    Ok(parse_group(&mut iter))
}

fn parse_group(iter: &mut Peekable<Chars>) -> Group {
    let mut components = Vec::new();
    let mut current_token = String::new();

    match iter.peek() {
        // None - case for the empty string
        // Some(&) - case for & in the beginning of the group or payload (e.g. "{&&&)}")
        Some('&') | None => {
            components.push(Component::Token(Token {
                value: "".to_string(),
            }));
        }
        _ => {}
    }

    while let Some(&c) = iter.peek() {
        match c {
            '&' => {
                if !current_token.is_empty() {
                    components.push(Component::Token(Token {
                        value: current_token,
                    }));
                    current_token = String::new();
                }
                iter.next();

                match iter.peek() {
                    // Case for & at the end of the group or payload (e.g. "n{1&{2&}}")
                    Some(&'}') | None => {
                        components.push(Component::Token(Token {
                            value: "".to_string(),
                        }));
                    }
                    // Case for groupped &&
                    Some(&'&') => {
                        components.push(Component::Token(Token {
                            value: "".to_string(),
                        }));
                    }
                    _ => {}
                }
            }
            '{' => {
                iter.next();
                components.push(Component::Group(parse_group(iter)));
            }
            '}' => {
                iter.next();
                if !current_token.is_empty() {
                    components.push(Component::Token(Token {
                        value: current_token,
                    }));
                    current_token = String::new();
                }
                // Parsing the value of the group if it exists, like "{group}value&other"
                while let Some(&c) = iter.peek() {
                    match c {
                        '}' | '&' => {
                            break;
                        }
                        _ => {
                            current_token.push(c);
                            iter.next();
                        }
                    }
                }
                let value = match current_token.is_empty() {
                    true => None,
                    false => Some(current_token),
                };
                return Group { components, value };
            }
            _ => {
                current_token.push(c);
                iter.next();
            }
        }
    }

    // Add any remaining token
    if !current_token.is_empty() {
        components.push(Component::Token(Token {
            value: current_token,
        }));
    }

    Group {
        components,
        value: None,
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Concentration {
    // To get idea what those infixes do, check cauculate_capacity function
    /// Percentage points, 51pp0 equals 51 percent, 5pp1 equals 50 percent
    PP,
    /// Weight to total volume ratio (in percent), ~25wr-3 equals to 2.5% of weight of solution
    WV,
    /// Weight to total weight ratio (in percent), 37ww-2 equals 37 grams per 100 grams of solution (~37%)
    WF,
    /// Volume to total volume ratio (in percent), 87rf-3 equals 8.7 milliliters per 100 milliliters of solution (~8.7%)
    RF,
    /// Mole to total mole ratio (in percent), 12mf0 equals 12 moles per 100 moles of solution (~12%)
    MF,
    /// Ratio of two volumes, 37vp0&28vp0 equals 37:28 ratio
    VP,
    /// Mole per liter of total solution, 17mr-1 equals 1.7 moles per liter of solution
    MR,
    /// Mole per kilogram of solvent, 55mb-1 equals 5.5 moles per kilogram of solution
    MB,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Capacity {
    Absolute(usize),
    Relative,
    Unestimated,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Content {
    pub value: usize,
    pub concentration: Concentration,
    pub magnitude: isize,
}

impl Content {
    pub fn from_str(payload: &str) -> Result<Self, String> {
        let (value, concentration, magnitude) = split_payload(payload)?;
        Ok(Self {
            value,
            concentration,
            magnitude,
        })
    }

    pub fn value_at_magnitude(&self, magnitude: &isize) -> usize {
        if magnitude == &self.magnitude {
            return self.value;
        } else if magnitude > &self.magnitude {
            // TODO: test this
            // It's not like it is impossible to calculate size at higher magnitude,
            // but it makes no sense in this context, so this is defensive check against it.
            // (The flow will always choose lowest available magnitude to avoid float caltulations,
            // so this should never be triggered)
            unreachable!("Calculating size at higher magnitude is blocked");
        } else {
            return self.value * 10usize.pow((self.magnitude - magnitude) as u32);
        }
    }

    /// Calculate capacity of concenration type at given magnitude.
    /// This is part of the calculation of value of the content - to avoid float calculations,
    /// the smallest magnitude is always selected (during rendering of mixture bar), and then the
    /// capacity (aka max possible value) is calculated at that specific magnitude.
    ///
    /// Imagine that you have 35pp0, 15pp0 and 5pp1 - the smallest magnitude is 0, so max capacity for PP
    /// at magnitude 0 is 100. Combine it with value_at_magnitude and you get 35, 15 and 50 that
    /// sums up to (maximum capacity) 100.
    ///
    /// For 5pp1, 2pp1 and 3pp1 values will be 5, 2 and 3, so they will sum up to 10 (max capacity at magnitude 1).
    pub fn calculate_capacity(concentration: &Concentration, magnitude: &isize) -> Capacity {
        match concentration {
            Concentration::PP | Concentration::MF => {
                if magnitude > &1isize {
                    unreachable!("Magnitude too big");
                }
                Capacity::Absolute(10usize.pow(-(magnitude - 2) as u32))
            }
            Concentration::WV | Concentration::WF | Concentration::RF => {
                if magnitude > &-1isize {
                    unreachable!("Magnitude too big");
                }
                Capacity::Absolute(10usize.pow(-magnitude as u32))
            }
            Concentration::VP => Capacity::Relative,
            Concentration::MR | Concentration::MB => Capacity::Unestimated,
        }
    }

    /// Check what is maximum magnitude that makes sense for calculations.
    /// In other words at which level we can still talk about meaningful parts, and not values that
    /// are above 100% of content.
    pub fn maximum_viable_magnitude(concentration: &Concentration) -> Option<isize> {
        match concentration {
            Concentration::PP | Concentration::MF => Some(1),
            Concentration::WV | Concentration::WF | Concentration::RF => Some(-1),
            Concentration::MR | Concentration::MB => Some(0),
            Concentration::VP => None,
        }
    }
}

fn split_payload(payload: &str) -> Result<(usize, Concentration, isize), String> {
    let (concentration, split) = match payload {
        s if s.contains("pp") => (Concentration::PP, payload.split("pp")),
        s if s.contains("wf") => (Concentration::WF, payload.split("wf")),
        s if s.contains("wv") => (Concentration::WV, payload.split("wv")),
        s if s.contains("rf") => (Concentration::RF, payload.split("rf")),
        s if s.contains("mf") => (Concentration::MF, payload.split("mf")),
        s if s.contains("vp") => (Concentration::VP, payload.split("vp")),
        s if s.contains("mr") => (Concentration::MR, payload.split("mr")),
        s if s.contains("mb") => (Concentration::MB, payload.split("mb")),
        _ => {
            return Err(format!(
                "Invalid content notation, unrecognized content infix idetifier - {:?}",
                payload
            ))
        }
    };

    let chunks: Vec<&str> = split.collect();

    if chunks.len() != 2 {
        return Err(format!(
            "Invalid content notation, too many parts - {:?}",
            payload
        ));
    }

    let value = match chunks[0] {
        s if s.starts_with("<=") || s.starts_with(">=") => {
            parse_result!(s[2..].parse::<usize>(), payload)?
        }
        s if s.starts_with('~') || s.starts_with('<') || s.starts_with('>') => {
            parse_result!(s[1..].parse::<usize>(), payload)?
        }
        s if s.contains(":") => {
            let parts: Vec<&str> = s.split(":").collect();
            if parts.len() != 2 {
                return Err(format!(
                    "Invalid content notation, too many parts - {:?}",
                    payload
                ));
            }

            // To be honest first parsing isnt really needed, but lets validate it anyway
            // TODO: describe why bigger part it taken only
            parse_result!(parts[0].parse::<usize>(), payload)?;
            parse_result!(parts[1].parse::<usize>(), payload)?
        }
        s => parse_result!(s.parse::<usize>(), payload)?,
    };

    let magnitude = parse_result!(chunks[1].parse::<isize>(), payload)?;

    Ok((value, concentration, magnitude))
}

#[derive(Debug, Eq, PartialEq)]
pub struct Mixture {
    pub ingredients: Vec<Ingredient>,
    pub content: Option<Content>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Substance {
    pub index: Option<String>,
    pub content: Option<Content>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Ingredient {
    Mixture(Mixture),
    Substance(Substance),
}

pub fn generate_mixture_tree(indexing: &str, concentration: &str) -> Result<Mixture, String> {
    if indexing.is_empty() {
        return Err("Empty indexing part, you must pass at least \"/n/\"".to_string());
    }
    if concentration.is_empty() {
        return Err("Empty concentration part, you must pass at least \"/g/\"".to_string());
    }
    let i_tree = tokenize_string(indexing, 'n')?;
    let c_tree = tokenize_string(concentration, 'g')?;
    combine_groups(&i_tree, &c_tree)
}

fn combine_groups(indexing_group: &Group, concentration_group: &Group) -> Result<Mixture, String> {
    Ok(Mixture {
        ingredients: combine_components(
            &indexing_group.components,
            &concentration_group.components,
        )?,
        content: match &concentration_group.value {
            Some(v) => Some(Content::from_str(&v)?),
            None => None,
        },
    })
}

fn combine_components(
    indexing_components: &Vec<Component>,
    concentration_components: &Vec<Component>,
) -> Result<Vec<Ingredient>, String> {
    let mut combined_components = Vec::new();
    if indexing_components.len() != concentration_components.len() {
        return Err(format!(
            "Mismatched components, found {} and {} items (\"{}\" and \"{}\")",
            indexing_components.len(),
            concentration_components.len(),
            stringify_group(&Group {
                components: indexing_components.clone(),
                value: None
            }),
            stringify_group(&Group {
                components: concentration_components.clone(),
                value: None
            }),
        ));
    }
    for i in 0..indexing_components.len() {
        match (&indexing_components[i], &concentration_components[i]) {
            (Component::Token(t1), Component::Token(t2)) => {
                combined_components.push(Ingredient::Substance(create_substance(t1, t2)?));
            }
            (Component::Group(g1), Component::Group(g2)) => {
                combined_components.push(Ingredient::Mixture(combine_groups(g1, g2)?));
            }
            _ => return Err(format!("Mismatched components, found mixture and substance on corresponding positions in indexing and concentration notation"))
        }
    }
    Ok(combined_components)
}

fn stringify_group(group: &Group) -> String {
    let mut result = "{".to_string();
    let len = group.components.len();
    let mut i = 1;
    for component in &group.components {
        match component {
            Component::Token(t) => result.push_str(&t.value),
            Component::Group(g) => result.push_str(&stringify_group(g)),
        }
        if i != len {
            result.push('&');
        }
        i += 1;
    }
    result.push('}');
    match &group.value {
        Some(v) => result.push_str(v),
        None => {}
    }
    result
}

fn create_substance(indexing: &Token, concentration: &Token) -> Result<Substance, String> {
    Ok(Substance {
        index: match &indexing.value {
            c if *c == "".to_string() => None,
            _ => Some(indexing.value.clone()),
        },
        content: match concentration.value.clone() {
            c if c == "".to_string() => None,
            _ => Some(Content::from_str(&concentration.value)?),
        },
    })
}
