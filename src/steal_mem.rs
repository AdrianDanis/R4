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

/// Custom box for our returned alloccations
/// This does not implement drop as we do not support freeing these.
/// Has a reference to phantom data to ensure this allocation does not
/// live too long
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
        where I: Iterator<Item=(usize,usize)>, W: VSpaceWindow<'a> + 'w {
    iter: I,
    range: (usize, usize),
    window: &'w W,
    phantom: PhantomData<&'a usize>,
}

impl<'a, 'w, I, W> StealMem<'a, 'w, I, W>
        where I: Iterator<Item=(usize, usize)>, W:VSpaceWindow<'a> {
    /// Construct a new allocator. Expects to be passed an iterator that will
    /// yield (start, end) pairs. The return results should be ordered such
    /// that early pairs are (hopefully) valid in the supplied window such
    /// that any early allocations can succeed
    ///
    /// # Safety
    ///
    /// Enough of the early iterations must be valid in the supplied window
    /// for all in place allocations to succeed
    pub unsafe fn new(i: I, w: &'w W) -> StealMem<'a, 'w, I, W> {
        StealMem {iter: i, window: w, phantom: PhantomData, range: (0, 0)}
    }
    /// Return an in place allocator to construct a variable out of
    pub unsafe fn alloc<T>(&mut self, align: usize) -> StealBoxPlace<'a, T> {
        /* Technically we should reserve a range here, and then complete
         * the allocation in finalize. But since this is a single threaded
         * allocated that panics if anything goes wrong (in a kernel that
         * panics if anything goes wrong) then it's fine to just alloc
         * directly. */
        let base = self.alloc_raw(size_of::<T>(), size_of::<T>());
        /* grab the raw range from the window */
        let slice: &'a [u8] = self.window.make_slice(1, size_of::<T>());
        /* pull out the pointer and forget about the slice */
        let pointer = slice.as_ptr();
        forget(slice);
        /* return the raw pointer to the start of the block, constructing
         * a fake lifetime that is equivalent to the original slice lifetime
         */
        StealBoxPlace { ptr: pointer as *mut T, lifetime: transmute(&PhantomData::<usize>) }
    }
    /// Internal function allocates a range with the given alignment
    fn alloc_raw(&mut self, size: usize, align: usize) -> usize {
        unimplemented!()
    }
}
