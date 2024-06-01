use moleco::tokenize::{Capacity, Concentration, Content};

macro_rules! assert_absolute_capacity {
    ($result: expr, $value: expr) => {
        if let Capacity::Absolute(value) = $result {
            assert_eq!(value, $value);
        } else {
            panic!("Expected absolute capacity, got {:?}", $result);
        }
    };
}

macro_rules! assert_relative_capacity {
    ($result: expr) => {
        if let Capacity::Relative = $result {
            // Ok
        } else {
            panic!("Expected absolute capacity, got {:?}", $result);
        }
    };
}

macro_rules! assert_unestimated_capacity {
    ($result: expr) => {
        if let Capacity::Unestimated = $result {
            // Ok
        } else {
            panic!("Expected absolute capacity, got {:?}", $result);
        }
    };
}

//--- PP

#[test]
fn test_content_pp_1() {
    let content = Content::from_str("6pp1").unwrap();
    assert_eq!(content.value, 6);
    assert_eq!(content.concentration, Concentration::PP);
    assert_eq!(content.magnitude, 1);
    assert_eq!(content.value_at_magnitude(&0), 60);
}

#[test]
fn test_content_pp_2() {
    Content::from_str("6pp2").unwrap();
}

#[test]
fn test_content_capacity_pp_2() {
    assert_absolute_capacity!(Content::calculate_capacity(&Concentration::PP, &1isize), 10);
    assert_absolute_capacity!(
        Content::calculate_capacity(&Concentration::PP, &0isize),
        100
    );
    assert_absolute_capacity!(
        Content::calculate_capacity(&Concentration::PP, &-1isize),
        1000
    );
    assert_absolute_capacity!(
        Content::calculate_capacity(&Concentration::PP, &-2isize),
        10000
    );
    assert_absolute_capacity!(
        Content::calculate_capacity(&Concentration::PP, &-3isize),
        100000
    );
}

#[test]
#[should_panic]
fn test_content_capacity_pp_3() {
    Content::calculate_capacity(&Concentration::PP, &2isize);
}

#[test]
fn test_maximum_viable_magnitude_pp() {
    assert_eq!(
        Content::maximum_viable_magnitude(&Concentration::PP).unwrap(),
        1
    );
}

//--- WV

#[test]
fn test_content_wv_1() {
    let content = Content::from_str("25wv-2").unwrap();
    assert_eq!(content.value, 25);
    assert_eq!(content.concentration, Concentration::WV);
    assert_eq!(content.magnitude, -2);
    assert_eq!(content.value_at_magnitude(&-3), 250);
}

#[test]
fn test_content_wv_2() {
    Content::from_str("23wv2").unwrap();
}

#[test]
fn test_content_capacity_wv_2() {
    assert_absolute_capacity!(
        Content::calculate_capacity(&Concentration::WV, &-1isize),
        10
    );
    assert_absolute_capacity!(
        Content::calculate_capacity(&Concentration::WV, &-2isize),
        100
    );
    assert_absolute_capacity!(
        Content::calculate_capacity(&Concentration::WV, &-3isize),
        1000
    );
    assert_absolute_capacity!(
        Content::calculate_capacity(&Concentration::WV, &-4isize),
        10000
    );
}

#[test]
#[should_panic]
fn test_content_capacity_wv_3() {
    Content::calculate_capacity(&Concentration::WV, &0isize);
}

#[test]
fn test_maximum_viable_magnitude_wv() {
    assert_eq!(
        Content::maximum_viable_magnitude(&Concentration::WV).unwrap(),
        -1
    );
}

//--- WF

#[test]
fn test_content_wf_1() {
    let content = Content::from_str("37wf-3").unwrap();
    assert_eq!(content.value, 37);
    assert_eq!(content.concentration, Concentration::WF);
    assert_eq!(content.magnitude, -3);
    assert_eq!(content.value_at_magnitude(&-4), 370);
}

#[test]
fn test_content_wf_2() {
    Content::from_str("45wf2").unwrap();
}

#[test]
fn test_content_capacity_wf_2() {
    assert_absolute_capacity!(
        Content::calculate_capacity(&Concentration::WF, &-1isize),
        10
    );
    assert_absolute_capacity!(
        Content::calculate_capacity(&Concentration::WF, &-2isize),
        100
    );
    assert_absolute_capacity!(
        Content::calculate_capacity(&Concentration::WF, &-3isize),
        1000
    );
    assert_absolute_capacity!(
        Content::calculate_capacity(&Concentration::WF, &-4isize),
        10000
    );
}

#[test]
#[should_panic]
fn test_content_capacity_wf_3() {
    Content::calculate_capacity(&Concentration::WF, &0isize);
}

#[test]
fn test_maximum_viable_magnitude_wf() {
    assert_eq!(
        Content::maximum_viable_magnitude(&Concentration::WF).unwrap(),
        -1
    );
}

//--- RF

#[test]
fn test_content_rf_1() {
    let content = Content::from_str("42rf-2").unwrap();
    assert_eq!(content.value, 42);
    assert_eq!(content.concentration, Concentration::RF);
    assert_eq!(content.magnitude, -2);
    assert_eq!(content.value_at_magnitude(&-3), 420);
}

#[test]
fn test_content_rf_2() {
    Content::from_str("45rf2").unwrap();
}

#[test]
fn test_content_capacity_rf_2() {
    assert_absolute_capacity!(
        Content::calculate_capacity(&Concentration::RF, &-1isize),
        10
    );
    assert_absolute_capacity!(
        Content::calculate_capacity(&Concentration::RF, &-2isize),
        100
    );
    assert_absolute_capacity!(
        Content::calculate_capacity(&Concentration::RF, &-3isize),
        1000
    );
    assert_absolute_capacity!(
        Content::calculate_capacity(&Concentration::RF, &-4isize),
        10000
    );
}

#[test]
#[should_panic]
fn test_content_capacity_rf_3() {
    Content::calculate_capacity(&Concentration::RF, &0isize);
}

#[test]
fn test_maximum_viable_magnitude_rf() {
    assert_eq!(
        Content::maximum_viable_magnitude(&Concentration::RF).unwrap(),
        -1
    );
}

//--- MF

#[test]
fn test_content_mf_1() {
    let content = Content::from_str("3mf1").unwrap();
    assert_eq!(content.value, 3);
    assert_eq!(content.concentration, Concentration::MF);
    assert_eq!(content.magnitude, 1);
    assert_eq!(content.value_at_magnitude(&0), 30);
}

#[test]
fn test_content_mf_2() {
    Content::from_str("6mf3").unwrap();
}

#[test]
fn test_content_capacity_mf_2() {
    assert_absolute_capacity!(Content::calculate_capacity(&Concentration::MF, &1isize), 10);
    assert_absolute_capacity!(
        Content::calculate_capacity(&Concentration::MF, &0isize),
        100
    );
    assert_absolute_capacity!(
        Content::calculate_capacity(&Concentration::MF, &-1isize),
        1000
    );
    assert_absolute_capacity!(
        Content::calculate_capacity(&Concentration::MF, &-2isize),
        10000
    );
    assert_absolute_capacity!(
        Content::calculate_capacity(&Concentration::MF, &-3isize),
        100000
    );
}

#[test]
#[should_panic]
fn test_content_capacity_mf_3() {
    Content::calculate_capacity(&Concentration::MF, &2isize);
}

//--- VP

#[test]
fn test_content_vp_1() {
    let content = Content::from_str("5vp3").unwrap();
    assert_eq!(content.value, 5);
    assert_eq!(content.concentration, Concentration::VP);
    assert_eq!(content.magnitude, 3);
    assert_eq!(content.value_at_magnitude(&0), 5000);
}

#[test]
fn test_content_vp_2() {
    Content::from_str("6vp-3").unwrap();
}

#[test]
fn test_content_capacity_vp_2() {
    assert_relative_capacity!(Content::calculate_capacity(&Concentration::VP, &1isize));
}

//--- MR

#[test]
fn test_content_mr_1() {
    let content = Content::from_str("3mr0").unwrap();
    assert_eq!(content.value, 3);
    assert_eq!(content.concentration, Concentration::MR);
    assert_eq!(content.magnitude, 0);
    assert_eq!(content.value_at_magnitude(&0), 3);
}

#[test]
fn test_content_mr_2() {
    Content::from_str("6mr3").unwrap();
}

#[test]
fn test_content_capacity_mr_2() {
    assert_unestimated_capacity!(Content::calculate_capacity(&Concentration::MR, &1isize));
    assert_unestimated_capacity!(Content::calculate_capacity(&Concentration::MR, &0isize));
    assert_unestimated_capacity!(Content::calculate_capacity(&Concentration::MR, &-1isize));
    assert_unestimated_capacity!(Content::calculate_capacity(&Concentration::MR, &-2isize));
    assert_unestimated_capacity!(Content::calculate_capacity(&Concentration::MR, &-3isize));
}

//--- MB

#[test]
fn test_content_mb_1() {
    let content = Content::from_str("3mb0").unwrap();
    assert_eq!(content.value, 3);
    assert_eq!(content.concentration, Concentration::MB);
    assert_eq!(content.magnitude, 0);
    assert_eq!(content.value_at_magnitude(&0), 3);
}

#[test]
fn test_content_mb_2() {
    Content::from_str("6mb3").unwrap();
}

#[test]
fn test_content_capacity_mb_2() {
    assert_unestimated_capacity!(Content::calculate_capacity(&Concentration::MB, &1isize));
    assert_unestimated_capacity!(Content::calculate_capacity(&Concentration::MB, &0isize));
    assert_unestimated_capacity!(Content::calculate_capacity(&Concentration::MB, &-1isize));
    assert_unestimated_capacity!(Content::calculate_capacity(&Concentration::MB, &-2isize));
    assert_unestimated_capacity!(Content::calculate_capacity(&Concentration::MB, &-3isize));
}
