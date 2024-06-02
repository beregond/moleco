use moleco::calculate_scheme;

/// Main test to check if the calculate_scheme function works stable.
/// If values changed, it means the hashing algorithm changed.
#[test]
fn test_calculate_scheme() {
    let scheme = calculate_scheme("water".to_string());
    assert_eq!(scheme.primary.hue, 285);
    assert_eq!(scheme.first_accent.hue, 90);
    assert_eq!(scheme.second_accent.hue, 270);
    assert_eq!(scheme.complementary.hue, 105);

    let scheme2 = calculate_scheme("InChI=water".to_string());
    assert_eq!(scheme2.primary.hue, scheme.primary.hue);
    assert_eq!(scheme2.first_accent.hue, scheme.first_accent.hue);
    assert_eq!(scheme2.second_accent.hue, scheme.second_accent.hue);
    assert_eq!(scheme2.complementary.hue, scheme.complementary.hue);
}
