use moleco::tokenize::{
    generate_compound_hierarchy, tokenize_string, ComponentKind, Compound, Group, Token,
};

// source: http://molmatinf.com/minchidemo/
// 37% wt. Formaldehyde in Water with 10-15% Methanol
static FORMALDEHYDE: &str =
    "MInChI=0.00.1S/CH2O/c1-2/h1H2&CH4O/c1-2/h2H,1H3&H2O/h1H2/n{{1&3}&2}/g{{37wf-2&}&10:15pp0}";

fn t(value: &str) -> ComponentKind {
    ComponentKind::Token(Token {
        value: value.to_string(),
    })
}

fn g(components: Vec<ComponentKind>, value: Option<String>) -> Group {
    Group { components, value }
}

fn gk(components: Vec<ComponentKind>, value: Option<String>) -> ComponentKind {
    ComponentKind::Group(Group { components, value })
}

fn get_formaldehyde_ic() -> (&'static str, &'static str) {
    let mut chunks: Vec<&str> = FORMALDEHYDE.split('/').collect();
    let concentration = chunks.pop().unwrap();
    let indexing = chunks.pop().unwrap();
    (indexing, concentration)
}

#[test]
#[should_panic]
fn test_empty_string() {
    tokenize_string("", ' ');
}

#[test]
#[should_panic]
fn test_wrong_first_character() {
    tokenize_string("bbb", 'a');
}

#[test]
fn test_empty_components_string() {
    let result = tokenize_string("b", 'b');
    assert_eq!(result, g(vec![t("")], None));
}

#[test]
fn test_tokenization_1() {
    let result = tokenize_string("n1&2", 'n');
    assert_eq!(result, g(vec![t("1"), t("2")], None));
}

#[test]
fn test_tokenization_2() {
    let result = tokenize_string("n1&{2&3}&4", 'n');
    assert_eq!(
        result,
        g(vec![t("1"), gk(vec![t("2"), t("3")], None), t("4")], None)
    );
}

#[test]
fn test_tokenization_3() {
    let result = tokenize_string("n1&{2&3&{58&67}}&4", 'n');
    assert_eq!(
        result,
        g(
            vec![
                t("1"),
                gk(vec![t("2"), t("3"), gk(vec![t("58"), t("67")], None)], None),
                t("4")
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
    let result = tokenize_string("n1&{2&3&}&4", 'n');
    assert_eq!(
        result,
        g(
            vec![t("1"), gk(vec![t("2"), t("3"), t("")], None), t("4")],
            None
        )
    );
}
#[test]
fn test_hierarchy_1() {
    let (indexing, concentration) = get_formaldehyde_ic();
    let hierarchy = generate_compound_hierarchy(indexing, concentration);
    assert_eq!(
        hierarchy,
        Compound {
            components: vec![],
            content: Some("100".to_string()),
        }
    );
}
