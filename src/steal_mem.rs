//! Early memory stealing allocator
//!
//! During kernel bootup some memory may need to be allocated. This allocator
//! provides a way to allocate such memory under the assumption that it will
//! never be freed.
//!
//! It is assumed that any allocations can be fullfilled from memory that
//! resides in one of the early boot windows. The rest of the memory can
//! be iterated over to place into the final kernel window

use ::vspace::VSpaceWindow;
use ::core::marker::PhantomData;
use ::core::ops;
use ::core::mem::{size_of, forget, transmute};
use ::util;
use types::*;

/// Custom box for our returned alloccations
/// This does not implement drop as we do not support freeing these.
/// Has a reference to phantom data to ensure this allocation does not
/// live too long
#[allow(dead_code)]
pub struct StealBox<'a, T> {
    ptr: *const T,
    lifetime: &'a PhantomData<usize>,
}

pub struct StealBoxPlace<'a, T> {
    ptr: *mut T,
    lifetime: &'a PhantomData<usize>,
}

impl<'a, T> ops::Deref for StealBox<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe {&*self.ptr}
    }
}

impl<'a, T> ops::DerefMut for StealBox<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe {&mut *(self.ptr as *mut T)}
    }
}

impl<'a, T> ops::InPlace<T> for StealBoxPlace<'a, T> {
    type Owner = StealBox<'a, T>;
    unsafe fn finalize(self) -> Self::Owner {
        StealBox {ptr: self.ptr, lifetime: self.lifetime}
    }
}

impl<'a, T> ops::Placer<T> for StealBoxPlace<'a, T> {
    type Place = StealBoxPlace<'a, T>;
    /// This implementation makes no attempt to check that we can found a
    /// a valid place.
    fn make_place(self) -> Self {
        self
    }
}

impl<'a, T> ops::Place<T> for StealBoxPlace<'a, T> {
    fn pointer(&mut self) -> *mut T {
        self.ptr
    }
}

/// Abstract implementation of the memory stealing allocator.
///
/// Allocations can be performed by
///
/// ```
/// let steal_mem_impl = ...;
/// let obj = steal_mem_impl <- Obj::default()
/// ```
pub struct StealMem<'a, 'w, I, W>
        where I: Iterator<Item=(PAddr, PAddr)>, W: VSpaceWindow<'a> + 'w {
    iter: I,
    range: (PAddr, PAddr),
    window: &'w W,
    phantom: PhantomData<&'a usize>,
}

impl<'a, 'w, I, W> StealMem<'a, 'w, I, W>
        where I: Iterator<Item=(PAddr, PAddr)>, W:VSpaceWindow<'a> {
    /// Construct a new allocator. Expects to be passed an iterator that will
    /// yield (start, end) pairs of physical address. The return results
    /// should be ordered such that early pairs are (hopefully) valid in the
    /// supplied window such that any early allocations can succeed
    ///
    /// # Safety
    ///
    /// Enough of the early iterations must be valid in the supplied window
    /// for all in place allocations to succeed
    pub unsafe fn new(i: I, w: &'w W) -> StealMem<'a, 'w, I, W> {
        StealMem {iter: i, window: w, phantom: PhantomData, range: (PAddr(0), PAddr(0))}
    }
    /// Return an in place allocator to construct a variable out of
    ///
    /// # Safety
    ///
    /// Can only call alloc for as long as the original iterator returns
    /// memory within the vspace window
    pub unsafe fn alloc<T>(&mut self, align: usize) -> StealBoxPlace<'a, T> {
        /* Technically we should reserve a range here, and then complete
         * the allocation in finalize. But since this is a single threaded
         * allocated that panics if anything goes wrong (in a kernel that
         * panics if anything goes wrong) then it's fine to just alloc
         * directly. */
        let base = self.window.from_paddr(self.alloc_raw(size_of::<T>(), align));
        /* grab the raw range from the window */
        let slice: &'a [u8] = self.window.make_slice(base, size_of::<T>()).unwrap();
        /* pull out the pointer and forget about the slice */
        let pointer = slice.as_ptr();
        forget(slice);
        /* return the raw pointer to the start of the block, constructing
         * a fake lifetime that is equivalent to the original slice lifetime
         */
        StealBoxPlace { ptr: pointer as *mut T, lifetime: transmute(&PhantomData::<usize>) }
    }
    /// Internal function allocates a range with the given alignment
    fn alloc_raw(&mut self, size: usize, align: usize) -> PAddr {
        /* round the current range up by the alignment */
        let next_base = PAddr(util::round_up((self.range.0).0, align));
        /* see if this fits */
        if next_base.0 + size <= (self.range.1).0 {
            (self.range.0).0 = next_base.0 + size;
            return next_base;
        }
        /* look for another range */
        match self.iter.next() {
            Some(range) => self.range = range,
            None => panic!("Out of memory allocated {} bytes with {} align", size, align),
        }
        self.alloc_raw(size, align)
    }
}
