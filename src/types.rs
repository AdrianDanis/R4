//! Custom type wrappers for doing static checking

/// Wrapper for a physical address
#[derive(Ord, Eq, PartialEq, PartialOrd, Debug, Copy, Clone)]
pub struct PAddr(pub usize);
