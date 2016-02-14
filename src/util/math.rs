//! Math helper rountines

/// Round a value up by the given alignment
pub fn round_up(val: usize, align: usize) -> usize {
    val + (align - (val % align))
}
