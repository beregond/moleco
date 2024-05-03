use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Eq, PartialEq)]
pub struct Group {
    pub components: Vec<ComponentKind>,
    pub value: Option<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Token {
    pub value: String,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ComponentKind {
    Token(Token),
    Group(Group),
}

pub fn tokenize_string(input: &str, start: char) -> Group {
    if input.is_empty() {
        panic!("Empty input");
    }
    let mut iter = input.chars();
    if iter.next().unwrap() != start {
        panic!(
            "Wrong first character, expected {} in input {}",
            start, input
        )
    }
    let mut new_input: String = iter.collect();

    // Magic to remove the first and last character if they are '{' and '}'
    let len = new_input.len();
    if len > 1 {
        let first_char = new_input.get(0..1).unwrap();
        let last_char = new_input.get(len - 1..).unwrap();
        if first_char == "{" && last_char == "}" {
            new_input = new_input.get(1..len - 1).unwrap().to_string();
        }
    }

    let mut iter = new_input.chars().peekable();
    return parse_group(&mut iter);
}

fn parse_group(iter: &mut Peekable<Chars>) -> Group {
    let mut components = Vec::new();
    let mut current_token = String::new();

    // Case fot the empty string
    if iter.peek().is_none() {
        components.push(ComponentKind::Token(Token {
            value: "".to_string(),
        }));
    }

    while let Some(&c) = iter.peek() {
        match c {
            '&' => {
                if !current_token.is_empty() {
                    components.push(ComponentKind::Token(Token {
                        value: current_token,
                    }));
                    current_token = String::new();
                }
                iter.next();

                match iter.peek() {
                    // Case for & at the end of the group or payload (e.g. "n{1&{2&}}")
                    Some(&'}') | None => {
                        components.push(ComponentKind::Token(Token {
                            value: "".to_string(),
                        }));
                    }
                    // Case for groupped &&
                    Some(&'&') => {
                        components.push(ComponentKind::Token(Token {
                            value: "".to_string(),
                        }));
                    }
                    _ => {}
                }
            }
            '{' => {
                iter.next();
                components.push(ComponentKind::Group(parse_group(iter)));
            }
            '}' => {
                iter.next();
                if !current_token.is_empty() {
                    components.push(ComponentKind::Token(Token {
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
        components.push(ComponentKind::Token(Token {
            value: current_token,
        }));
    }

    Group {
        components,
        value: None,
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Compound {
    pub components: Vec<CompoundKind>,
    pub content: Option<String>,
}

impl Compound {
    pub fn calculate_level(&self) -> u32 {
        let mut max = 0;
        for component in &self.components {
            match component {
                CompoundKind::Compound(c) => {
                    let level = c.calculate_level();
                    if level > max {
                        max = level;
                    }
                }
                _ => {}
            }
        }
        return max + 1;
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ContentKind {
    PP,
    WV,
    WF,
    RF,
    MF,
    VP,
    MR,
    WR,
    MB,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Content {
    pub value: usize,
    pub kind: ContentKind,
    pub cardinality: isize,
}

impl Content {
    pub fn from_str(payload: &str) -> Result<Self, &str> {
        let (value, kind, cardinality) = split_payload(payload).unwrap();
        Ok(Self {
            value,
            kind,
            cardinality,
        })
    }
}

fn split_payload(payload: &str) -> Result<(usize, ContentKind, isize), String> {
    let (kind, split) = match payload {
        s if s.contains("pp") => (ContentKind::PP, payload.split("pp")),
        s if s.contains("wf") => (ContentKind::WF, payload.split("wf")),
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
        s if s.contains(":") => {
            let parts: Vec<&str> = s.split(":").collect();
            if parts.len() != 2 {
                return Err(format!(
                    "Invalid content notation, too many parts - {:?}",
                    payload
                ));
            }

            (parts[0].parse::<usize>().unwrap() + parts[1].parse::<usize>().unwrap()) / 2
        }
        s => s.parse::<usize>().unwrap(),
    };

    let cardinality = match chunks[1].parse::<isize>() {
        Ok(c) => c,
        Err(_) => {
            return Err(format!(
                "Invalid content notation, invalid cardinality - {:?}",
                payload
            ))
        }
    };

    Ok((value, kind, cardinality))
}

#[derive(Debug, Eq, PartialEq)]
pub struct Substance {
    pub index: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CompoundKind {
    Compound(Compound),
    Substance(Substance),
}

pub fn generate_compound_hierarchy(indexing: &str, concentration: &str) -> Compound {
    let i_tree = tokenize_string(indexing, 'n');
    let c_tree = tokenize_string(concentration, 'g');
    combine_groups(&i_tree, &c_tree)
}

fn combine_groups(indexing_group: &Group, concentration_group: &Group) -> Compound {
    Compound {
        components: combine_components(&indexing_group.components, &concentration_group.components),
        content: concentration_group.value.clone(),
    }
}

fn combine_components(
    indexing_components: &Vec<ComponentKind>,
    concentration_components: &Vec<ComponentKind>,
) -> Vec<CompoundKind> {
    let mut combined_components = Vec::new();
    assert_eq!(indexing_components.len(), concentration_components.len());
    for i in 0..indexing_components.len() {
        match (&indexing_components[i], &concentration_components[i]) {
            (ComponentKind::Token(t1), ComponentKind::Token(t2)) => {
                combined_components.push(CompoundKind::Substance(create_substance(t1, t2)));
            }
            (ComponentKind::Group(g1), ComponentKind::Group(g2)) => {
                combined_components.push(CompoundKind::Compound(combine_groups(g1, g2)));
            }
            _ => panic!("Mismatched components"),
        }
    }
    combined_components
}

fn create_substance(indexing: &Token, concentration: &Token) -> Substance {
    Substance {
        index: match &indexing.value {
            c if *c == "".to_string() => None,
            _ => Some(indexing.value.clone()),
        },
        content: match concentration.value.clone() {
            c if c == "".to_string() => None,
            _ => Some(concentration.value.clone()),
        },
    }
}
