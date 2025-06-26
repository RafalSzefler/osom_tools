use osom_tools_dev::macros::{assert_eq_hex, assert_neq_hex};

#[test]
fn test_assert_eq_hex() {
    let a = [0x01, 0x02, 0x03, 0x04];
    let b = [0x01, 0x02, 0x03, 0x04];

    assert_eq_hex!(a, b);
}

#[test]
#[should_panic]
fn test_not_assert_eq_hex() {
    let a = [0x01, 0x02, 0x03, 0x04];
    let b = [0x05];

    assert_eq_hex!(a, b);
}

#[test]
fn test_assert_neq_hex() {
    let a = [0x01, 0x02, 0x03, 0x04];
    let b = [0x05];

    assert_neq_hex!(a, b);
}

#[test]
#[should_panic]
fn test_not_assert_neq_hex() {
    let a = [0x01, 0x02, 0x03, 0x04];
    let b = [0x01, 0x02, 0x03, 0x04];

    assert_neq_hex!(a, b);
}
