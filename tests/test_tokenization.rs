use moleco::tokenize::{
    generate_mixture_tree, tokenize_string, Component, Concentration, Content, Group, Ingredient,
    Mixture, Substance, Token,
};

// source: http://molmatinf.com/minchidemo/
// 37% wt. Formaldehyde in Water with 10-15% Methanol
static FORMALDEHYDE: &str =
    "MInChI=0.00.1S/CH2O/c1-2/h1H2&CH4O/c1-2/h2H,1H3&H2O/h1H2/n{{1&3}&2}/g{{37wf-2&}&10:15pp0}";

static LITHIUM_DIISOPROPYLAMIDE_SOLUTION: &str =
    "MInChI=0.00.1S/C4H8O/c1-2-4-5-3-1/h1-4H2&C6H12/c1-6-4-2-3-5-6/h6H,2-5H2,1H3&C6H14/c1-3-5-6-4-2/\
    h3-6H2,1-2H3&C6H14/c1-4-5-6(2)3/h6H,4-5H2,1-3H3&C6H14/c1-4-6(3)5-2/h6H,4-5H2,1-3H3&C6H14N.Li/c1-5\
    (2)7-6(3)4;/h5-6H,1-4H3;/q-1;+1/n{6&{1&{3&2&4&5}}}/g{1mr0&{1vp0&{5:7pp1&1:2pp1&1:5pp0&1:5pp0}7vp0}}";

static DISHWASHING_LIQUID: &str =
    "MInChI=0.00.1S/C12H26O4S.Na/c1-2-3-4-5-6-7-8-9-10-11-12-16-17(13,14)15;/h2-12H2,1H3,(H,13,14,15);\
    /q;+1/p-1&C18H30O3S.Na/c1-2-3-4-5-6-7-8-9-10-11-12-17-13-15-18(16-14-17)22(19,20)21;/h13-16H,2-12H2,\
    1H3,(H,19,20,21);/q;+1/p-1&ClH.Na/h1H;/q;+1/p-1&H2O/h1H2/n{4&{2&4}&&{1&4}&3}/g{807wf-3&{6pp1&4pp1}\
    117wf-3&1wf-2&{27pp0&73pp0}66wf-3&}";

macro_rules! enum_token {
    ($value: expr) => {
        Component::Token(Token {
            value: $value.to_string(),
        })
    };
}

macro_rules! group {
    ($components: expr, $value: expr) => {
        Group {
            components: $components,
            value: $value,
        }
    };
}

macro_rules! enum_group {
    ($components: expr, $value: expr) => {
        Component::Group(group!($components, $value))
    };
}

fn get_ic(payload: &str) -> (&str, &str) {
    let mut chunks: Vec<&str> = payload.split('/').collect();
    let concentration = chunks.pop().unwrap();
    let indexing = chunks.pop().unwrap();
    (indexing, concentration)
}

#[test]
#[should_panic]
fn test_empty_string() {
    tokenize_string("", ' ').unwrap();
}

#[test]
#[should_panic]
fn test_wrong_first_character() {
    tokenize_string("bbb", 'a').unwrap();
}

#[test]
fn test_empty_components_string() {
    let result = tokenize_string("b", 'b').unwrap();
    assert_eq!(result, group!(vec![enum_token!("")], None));
}

#[test]
fn test_tokenization_1() {
    let result = tokenize_string("n1&2", 'n').unwrap();
    assert_eq!(
        result,
        group!(vec![enum_token!("1"), enum_token!("2")], None)
    );
}

#[test]
fn test_tokenization_2() {
    let result = tokenize_string("n1&{2&3}&4", 'n').unwrap();
    assert_eq!(
        result,
        group!(
            vec![
                enum_token!("1"),
                enum_group!(vec![enum_token!("2"), enum_token!("3")], None),
                enum_token!("4")
            ],
            None
        )
    );
}

#[test]
fn test_tokenization_3() {
    let result = tokenize_string("n1&{2&3&{58&67}}&4", 'n').unwrap();
    assert_eq!(
        result,
        group!(
            vec![
                enum_token!("1"),
                enum_group!(
                    vec![
                        enum_token!("2"),
                        enum_token!("3"),
                        enum_group!(vec![enum_token!("58"), enum_token!("67")], None)
                    ],
                    None
                ),
                enum_token!("4")
            ],
            None
        )
    );
}

#[test]
fn test_tokenization_4() {
    assert_eq!(
        tokenize_string("n1&{2&3}&4", 'n'),
        tokenize_string("n{1&{2&3}&4}", 'n'),
    );
}

#[test]
fn test_tokenization_5() {
    let result = tokenize_string("n1&{2&3&}&4", 'n').unwrap();
    assert_eq!(
        result,
        group!(
            vec![
                enum_token!("1"),
                enum_group!(
                    vec![enum_token!("2"), enum_token!("3"), enum_token!("")],
                    None
                ),
                enum_token!("4")
            ],
            None
        )
    );
}

#[test]
fn test_tokenization_6() {
    let result = tokenize_string("n1&{2&3&{58&67}foo}bar&4", 'n').unwrap();
    assert_eq!(
        result,
        group!(
            vec![
                enum_token!("1"),
                enum_group!(
                    vec![
                        enum_token!("2"),
                        enum_token!("3"),
                        enum_group!(
                            vec![enum_token!("58"), enum_token!("67")],
                            Some("foo".to_string())
                        )
                    ],
                    Some("bar".to_string())
                ),
                enum_token!("4")
            ],
            None
        )
    );
}

#[test]
fn test_tokenization_7() {
    assert_eq!(
        tokenize_string("n{1&2}&{3&4}", 'n'),
        tokenize_string("n{{1&2}&{3&4}}", 'n'),
    );
}

#[test]
#[should_panic]
fn test_tokenization_8() {
    tokenize_string("n{1&2}&{3&4", 'n').unwrap();
}

#[test]
#[should_panic]
fn test_tokenization_9() {
    tokenize_string("n{1&2}}&{3&4}", 'n').unwrap();
}

#[test]
fn test_tokenization_10() {
    let result = tokenize_string("n{&&&}", 'n').unwrap();
    assert_eq!(
        result,
        group!(
            vec![
                enum_token!(""),
                enum_token!(""),
                enum_token!(""),
                enum_token!(""),
            ],
            None
        )
    );
}

#[test]
fn test_tokenization_11() {
    let result = tokenize_string(
        "g{807wf-3&{6pp1&4pp1}117wf-3&1wf-2&{27pp0&73pp0}66wf-3&}",
        'g',
    )
    .unwrap();
    assert_eq!(
        result,
        group!(
            vec![
                enum_token!("807wf-3"),
                enum_group!(
                    vec![enum_token!("6pp1"), enum_token!("4pp1")],
                    Some("117wf-3".to_string())
                ),
                enum_token!("1wf-2"),
                enum_group!(
                    vec![enum_token!("27pp0"), enum_token!("73pp0")],
                    Some("66wf-3".to_string())
                ),
                enum_token!(""),
            ],
            None
        )
    );
}

fn m(ingredients: Vec<Ingredient>, content: Option<String>) -> Ingredient {
    let content = match content {
        Some(c) => Some(Content::from_str(&c).unwrap()),
        None => None,
    };
    Ingredient::Mixture(Mixture {
        ingredients,
        content,
    })
}

fn s(index: Option<String>, content: Option<String>) -> Ingredient {
    let content = match content {
        Some(c) => Some(Content::from_str(&c).unwrap()),
        None => None,
    };
    Ingredient::Substance(Substance { index, content })
}

#[test]
fn test_tree_1() {
    let (indexing, concentration) = get_ic(&FORMALDEHYDE);
    let tree = generate_mixture_tree(indexing, concentration).unwrap();
    assert_eq!(
        tree,
        Mixture {
            ingredients: vec![
                m(
                    vec![
                        s(Some("1".to_string()), Some("37wf-2".to_string())),
                        s(Some("3".to_string()), None)
                    ],
                    None
                ),
                s(Some("2".to_string()), Some("10:15pp0".to_string()))
            ],
            content: None,
        }
    );
}

#[test]
fn test_tree_2() {
    let (indexing, concentration) = get_ic(&LITHIUM_DIISOPROPYLAMIDE_SOLUTION);
    let tree = generate_mixture_tree(indexing, concentration).unwrap();
    assert_eq!(
        tree,
        Mixture {
            ingredients: vec![
                s(Some("6".to_string()), Some("1mr0".to_string())),
                m(
                    vec![
                        s(Some("1".to_string()), Some("1vp0".to_string())),
                        m(
                            vec![
                                s(Some("3".to_string()), Some("5:7pp1".to_string())),
                                s(Some("2".to_string()), Some("1:2pp1".to_string())),
                                s(Some("4".to_string()), Some("1:5pp0".to_string())),
                                s(Some("5".to_string()), Some("1:5pp0".to_string())),
                            ],
                            Some("7vp0".to_string())
                        ),
                    ],
                    None
                )
            ],

            content: None,
        }
    );
}

#[test]
fn test_content_parsing() {
    let content = Content::from_str("66wf-3").unwrap();
    assert_eq!(
        content,
        Content {
            value: 66,
            concentration: Concentration::WF,
            magnitude: -3,
        }
    );
}

#[test]
fn test_content_parsing_range() {
    let content = Content::from_str("5:7wf-3").unwrap();
    assert_eq!(
        content,
        Content {
            value: 7,
            concentration: Concentration::WF,
            magnitude: -3,
        }
    );
}

#[test]
fn test_content_parsing_range_2() {
    let content = Content::from_str("2:5pp0").unwrap();
    assert_eq!(
        content,
        Content {
            value: 5,
            concentration: Concentration::PP,
            magnitude: 0,
        }
    );
}

#[test]
fn test_mixture_tree() {
    let (indexing, concentration) = get_ic(&DISHWASHING_LIQUID);
    let tree = generate_mixture_tree(indexing, concentration).unwrap();
    assert_eq!(
        tree,
        Mixture {
            ingredients: vec![
                s(Some("4".to_string()), Some("807wf-3".to_string())),
                m(
                    vec![
                        s(Some("2".to_string()), Some("6pp1".to_string())),
                        s(Some("4".to_string()), Some("4pp1".to_string())),
                    ],
                    Some("117wf-3".to_string())
                ),
                s(None, Some("1wf-2".to_string())),
                m(
                    vec![
                        s(Some("1".to_string()), Some("27pp0".to_string())),
                        s(Some("4".to_string()), Some("73pp0".to_string())),
                    ],
                    Some("66wf-3".to_string())
                ),
                s(Some("3".to_string()), None),
            ],
            content: None,
        }
    );
}
