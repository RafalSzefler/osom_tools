#[inline(always)]
pub fn call<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    f()
}
