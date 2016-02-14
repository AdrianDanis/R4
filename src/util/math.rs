//! Math helper rountines

/// Round a value up by the given alignment
pub fn round_up(val: usize, align: usize) -> usize {
    let diff = val % align;
    if diff == 0 {
        val
    } else {
        val + (align - diff)
    }
}
