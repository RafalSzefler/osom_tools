#[macro_export]
macro_rules! assert_eq_hex {
    ($left:expr, $right:expr) => {{
        let left_expr = $crate::traits::AsDisplayAsHex::as_display_as_hex(&($left));
        let right_expr = $crate::traits::AsDisplayAsHex::as_display_as_hex(&($right));
        if left_expr.as_slice() != right_expr.as_slice() {
            panic!(
                r#"assertion `left == right` failed
  left: {left_expr}
 right: {right_expr}"#
            )
        }
    }};
}
