//! Abstraction for a vspace window
//!
//! This vspace window abstraction is meant to allow for the current virtual
//! address layout to be described, and then ensure that only objects with
//! virtual addresses inside that window are created.
//!
//! *TODO*: Implement windows that have 'maybe' address ranges, such that
//! the corrspending objects have a type that forces them to be used in
//! a context where there is an appropriate fault handler setup to catch
//! invalid accesses
use ::core::intrinsics::transmute;
use ::core::num::Wrapping;
use ::core::mem::size_of;
use ::core::slice;
use ::core::fmt::Debug;
use ::core::ops::Deref;
use types::*;

/// Describes a single window into the virtual address space in the form
/// of a base:limit
/// The allocation of objects is not done with in place allocation syntax
/// as you may want the contents of the objects that are already here.
/// Use the `early_mem` allocator if you want to construct initialized
/// objects
///
/// # Safety
///
/// Windows must be created with an appropriate lifetime that describes
/// the actual length of the window. This also means only windows that are
/// actually described by the current address space root should be created
///
/// There is no restriction on creating multiple duplicate overlapping
/// windows, provided the restrictions on lifetimes is preserved
pub unsafe trait VSpaceWindow<'a> where Self::Addr: Copy + Clone + Debug + Deref<Target=usize>{
    /// An address whose type says it is valid in this window
    type Addr;
    /// Get the base address of the window
    fn base(&self) -> usize;
    /// Get the limit of the window
    fn size(&self) -> usize;
    /// Create a reference to an existing object in the virtual address
    /// space. The returned reference has the same lifetime as the window
    ///
    /// Care should be taken in understanding what the `drop` semantics of
    /// the requested object are, as Rust will believe this is a constructed
    /// object and will call `drop` when it goes out of scope. If you are
    /// expected to be able to re `make` this object in the future then
    /// your object should be of a simple type such that the `drop`
    /// implementation is a no-op
    ///
    /// # Panics
    ///
    /// This does not return an error, rather if the requested object would
    /// fall even partially outside the window a panic is raised.
    ///
    /// # Safety
    ///
    /// It is assumed that there is a valid initialized object of type `T`
    /// at the address provided.
    ///
    /// Whilst indirectly implied by the previous point, this means that
    /// even if the overall trait safety requirements are fullfilled and
    /// this window is currently valid, that just means that the virtual
    /// addresses are valid, it is the requirement of the caller of this
    /// function to ensure something real exists at the physical address
    /// of this mapping.
    unsafe fn make<T: Sized>(&self, b: Self::Addr) -> &'a T {
        if !self.addr_range_valid(b, size_of::<T>()) {
            panic!("Cannot make object at {:?}", b);
        }
        return transmute(*b);
    }
    /// This function is very similar to `make` except that it constructs
    /// a slice containing potentially multiple objects of type `T`.
    /// Otherwise everything that applies to `make` applies to this
    ///
    /// # Panics
    ///
    /// See `make`
    ///
    /// # Safety
    ///
    /// See `make`
    unsafe fn make_slice<T: Sized>(&self, b: Self::Addr, num: usize) -> &'a [T] {
        match self.maybe_make_slice(b, num) {
            None => panic!("Cannot make array with {} elements at {:?}", num, b),
            Some(slice) => slice,
        }
    }
    /// Similar to `make_slice`, but can return `None`
    ///
    /// # Safety
    ///
    /// See `make`
    unsafe fn maybe_make_slice<T: Sized>(&self, b: Self::Addr, num: usize) -> Option<&'a [T]> {
        match self.addr_range_valid(b, size_of::<T>() * num) {
            true => Some(slice::from_raw_parts(transmute(*b), num)),
            false => None,
        }
    }
    /// Creates a new window that is a subwindow of this one. The way of
    /// of describing the new window is to pass in an already constructed
    /// window. This is a slightly strange API, but is the nicest one I could
    /// think of, although it is the reason this function is marked as unsafe.
    ///
    /// # Panics
    ///
    /// This does not return an error, rather if the new window is outside
    /// the range of this one a panic is raised
    ///
    /// # Safety
    ///
    /// Subwindows should only be created outside of this function to be
    /// passed into this function
    unsafe fn subwindow<'i, I, O: ?Sized>(&self, window: I) -> &'a O where O: VSpaceWindow<'a>, I: VSpaceWindow<'i> {
        /* Validate the range for this window */
        if !self.range_valid(window.base(), window.size()) {
            panic!("Cannot construct window with range {} {}, from {} {}");
        }
        /* transmate to extend the lifetime. This is safe as
         * it is a zero sized object */
        transmute(&O::default())
    }
    /// Tests if a range of bytes would be valid in this window
    fn range_valid(&self, b: usize, s: usize) -> bool {
        /* We have moved the `- s` to the other side of the equation
         * so that if `self.base() + self.size()` overflowed
         * then subtracting `s` will either correct if the range is
         * valid, or not correct in which case the range is invalid
         * and the comparison will fail */
        b >= self.base() && b <= (Wrapping(self.base()) + Wrapping(self.size()) - Wrapping(s)).0
    }
    /// Tests if a range of bytes would be valid in this window. Takes
    /// as input a base address that is typed to be in the window, so is
    /// really testing if the size would put it out of the window
    fn addr_range_valid(&self, b: Self::Addr, s:usize) -> bool {
        self.range_valid(*b, s)
    }
    /// Return the default construction of a window
    ///
    /// # Safety
    ///
    /// Only windows that are actually valid should be constructed.
    /// Invalid windows can be constructed, but must not be used until
    /// they become valid
    unsafe fn default() -> Self;
    /// Translate a physical address to be within the range of the window
    ///
    /// # Safety
    ///
    /// Should only be called with a value that will end up within the range
    /// of this window
    unsafe fn from_paddr(&self, paddr: PAddr) -> Self::Addr;
    /// Translate an address from this window into a physical address.
    ///
    /// # Safety
    ///
    /// Should only be called on a value that is within this window
    unsafe fn to_paddr(&self, addr: Self::Addr) -> PAddr;
    /// Convert some virtual address into a an address for this window
    ///
    /// # Safety
    ///
    /// Address should be from this window
    unsafe fn to_addr(&self, addr: usize) -> Self::Addr;
}
