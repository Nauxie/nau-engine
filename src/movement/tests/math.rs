use super::super::smoothing_factor;

#[test]
fn smoothing_factor_never_overshoots() {
    assert!((0.0..=1.0).contains(&smoothing_factor(8.0, 0.5)));
    assert!((0.0..=1.0).contains(&smoothing_factor(8.0, 3.0)));
}
