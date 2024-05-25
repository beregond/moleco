use moleco::tokenize::{Content, ContentKind};

#[test]
fn test_content_pp_1() {
    let content = Content::from_str("6pp1").unwrap();
    assert_eq!(content.value, 6);
    assert_eq!(content.kind, ContentKind::PP);
    assert_eq!(content.magnitude, 1);
    assert_eq!(content.value_at_magnitude(&0), 60);
}

#[test]
fn test_content_pp_2() {
    Content::from_str("6pp2").unwrap();
}

#[test]
fn test_content_capacity_pp_2() {
    assert_eq!(
        Content::calculate_capacity(&ContentKind::PP, &1isize).unwrap(),
        10
    );
    assert_eq!(
        Content::calculate_capacity(&ContentKind::PP, &0isize).unwrap(),
        100
    );
    assert_eq!(
        Content::calculate_capacity(&ContentKind::PP, &-1isize).unwrap(),
        1000
    );
    assert_eq!(
        Content::calculate_capacity(&ContentKind::PP, &-2isize).unwrap(),
        10000
    );
    assert_eq!(
        Content::calculate_capacity(&ContentKind::PP, &-3isize).unwrap(),
        100000
    );
}

#[test]
#[should_panic]
fn test_content_capacity_pp_3() {
    Content::calculate_capacity(&ContentKind::PP, &2isize).unwrap();
}

#[test]
fn test_maximum_viable_magnitude_pp() {
    assert_eq!(
        Content::maximum_viable_magnitude(&ContentKind::PP).unwrap(),
        1
    );
}

#[test]
fn test_content_wv_1() {
    let content = Content::from_str("25wv-2").unwrap();
    assert_eq!(content.value, 25);
    assert_eq!(content.kind, ContentKind::WV);
    assert_eq!(content.magnitude, -2);
    assert_eq!(content.value_at_magnitude(&-3), 250);
}

#[test]
fn test_content_wv_2() {
    Content::from_str("23wv2").unwrap();
}

#[test]
fn test_content_capacity_wv_2() {
    assert_eq!(
        Content::calculate_capacity(&ContentKind::WV, &-1isize).unwrap(),
        10
    );
    assert_eq!(
        Content::calculate_capacity(&ContentKind::WV, &-2isize).unwrap(),
        100
    );
    assert_eq!(
        Content::calculate_capacity(&ContentKind::WV, &-3isize).unwrap(),
        1000
    );
    assert_eq!(
        Content::calculate_capacity(&ContentKind::WV, &-4isize).unwrap(),
        10000
    );
}

#[test]
#[should_panic]
fn test_content_capacity_wv_3() {
    Content::calculate_capacity(&ContentKind::WV, &0isize).unwrap();
}

#[test]
fn test_maximum_viable_magnitude_wv() {
    assert_eq!(
        Content::maximum_viable_magnitude(&ContentKind::WV).unwrap(),
        -1
    );
}

#[test]
fn test_content_wf_1() {
    let content = Content::from_str("37wf-3").unwrap();
    assert_eq!(content.value, 37);
    assert_eq!(content.kind, ContentKind::WF);
    assert_eq!(content.magnitude, -3);
    assert_eq!(content.value_at_magnitude(&-4), 370);
}

#[test]
fn test_content_wf_2() {
    Content::from_str("45wf2").unwrap();
}

#[test]
fn test_content_capacity_wf_2() {
    assert_eq!(
        Content::calculate_capacity(&ContentKind::WF, &-1isize).unwrap(),
        10
    );
    assert_eq!(
        Content::calculate_capacity(&ContentKind::WF, &-2isize).unwrap(),
        100
    );
    assert_eq!(
        Content::calculate_capacity(&ContentKind::WF, &-3isize).unwrap(),
        1000
    );
    assert_eq!(
        Content::calculate_capacity(&ContentKind::WF, &-4isize).unwrap(),
        10000
    );
}

#[test]
#[should_panic]
fn test_content_capacity_wf_3() {
    Content::calculate_capacity(&ContentKind::WF, &0isize).unwrap();
}

#[test]
fn test_maximum_viable_magnitude_wf() {
    assert_eq!(
        Content::maximum_viable_magnitude(&ContentKind::WF).unwrap(),
        -1
    );
}

#[test]
fn test_content_rf_1() {
    let content = Content::from_str("42rf-2").unwrap();
    assert_eq!(content.value, 42);
    assert_eq!(content.kind, ContentKind::RF);
    assert_eq!(content.magnitude, -2);
    assert_eq!(content.value_at_magnitude(&-3), 420);
}

#[test]
fn test_content_rf_2() {
    Content::from_str("45rf2").unwrap();
}

#[test]
fn test_content_capacity_rf_2() {
    assert_eq!(
        Content::calculate_capacity(&ContentKind::RF, &-1isize).unwrap(),
        10
    );
    assert_eq!(
        Content::calculate_capacity(&ContentKind::RF, &-2isize).unwrap(),
        100
    );
    assert_eq!(
        Content::calculate_capacity(&ContentKind::RF, &-3isize).unwrap(),
        1000
    );
    assert_eq!(
        Content::calculate_capacity(&ContentKind::RF, &-4isize).unwrap(),
        10000
    );
}

#[test]
#[should_panic]
fn test_content_capacity_rf_3() {
    Content::calculate_capacity(&ContentKind::RF, &0isize).unwrap();
}

#[test]
fn test_maximum_viable_magnitude_rf() {
    assert_eq!(
        Content::maximum_viable_magnitude(&ContentKind::RF).unwrap(),
        -1
    );
}

#[test]
fn test_content_mf_1() {
    let content = Content::from_str("3mf1").unwrap();
    assert_eq!(content.value, 3);
    assert_eq!(content.kind, ContentKind::MF);
    assert_eq!(content.magnitude, 1);
    assert_eq!(content.value_at_magnitude(&0), 30);
}

#[test]
fn test_content_mf_2() {
    Content::from_str("6mf3").unwrap();
}

#[test]
fn test_content_capacity_mf_2() {
    assert_eq!(
        Content::calculate_capacity(&ContentKind::MF, &1isize).unwrap(),
        10
    );
    assert_eq!(
        Content::calculate_capacity(&ContentKind::MF, &0isize).unwrap(),
        100
    );
    assert_eq!(
        Content::calculate_capacity(&ContentKind::MF, &-1isize).unwrap(),
        1000
    );
    assert_eq!(
        Content::calculate_capacity(&ContentKind::MF, &-2isize).unwrap(),
        10000
    );
    assert_eq!(
        Content::calculate_capacity(&ContentKind::MF, &-3isize).unwrap(),
        100000
    );
}

#[test]
#[should_panic]
fn test_content_capacity_mf_3() {
    Content::calculate_capacity(&ContentKind::MF, &2isize).unwrap();
}

#[test]
fn test_content_vp_1() {
    let content = Content::from_str("5vp3").unwrap();
    assert_eq!(content.value, 5);
    assert_eq!(content.kind, ContentKind::VP);
    assert_eq!(content.magnitude, 3);
    assert_eq!(content.value_at_magnitude(&0), 5000);
}

#[test]
fn test_content_vp_2() {
    Content::from_str("6vp-3").unwrap();
}

#[test]
fn test_content_capacity_vp_2() {
    assert_eq!(Content::calculate_capacity(&ContentKind::VP, &1isize), None);
}
//-----------------------------------------------
#[test]
fn test_content_mr_1() {
    let content = Content::from_str("3mr0").unwrap();
    assert_eq!(content.value, 3);
    assert_eq!(content.kind, ContentKind::MR);
    assert_eq!(content.magnitude, 0);
    assert_eq!(content.value_at_magnitude(&0), 3);
}

#[test]
fn test_content_mr_2() {
    Content::from_str("6mr3").unwrap();
}

#[test]
fn test_content_capacity_mr_2() {
    assert_eq!(
        Content::calculate_capacity(&ContentKind::MF, &1isize).unwrap(),
        10
    );
    assert_eq!(
        Content::calculate_capacity(&ContentKind::MF, &0isize).unwrap(),
        100
    );
    assert_eq!(
        Content::calculate_capacity(&ContentKind::MF, &-1isize).unwrap(),
        1000
    );
    assert_eq!(
        Content::calculate_capacity(&ContentKind::MF, &-2isize).unwrap(),
        10000
    );
    assert_eq!(
        Content::calculate_capacity(&ContentKind::MF, &-3isize).unwrap(),
        100000
    );
}

#[test]
#[should_panic]
fn test_content_capacity_mr_3() {
    Content::calculate_capacity(&ContentKind::MF, &2isize).unwrap();
}
