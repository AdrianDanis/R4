use core::intrinsics::transmute;
use core::num::Wrapping;
use core::mem::size_of;
use core::slice;

pub unsafe trait VSpaceWindow<'a> {
    fn base(&self) -> usize;
    fn size(&self) -> usize;
    unsafe fn make<T: Sized>(&self, b: usize) -> &'a T {
        if !self.range_valid(b, size_of::<T>()) {
            panic!("Cannot make object at {}", b);
        }
        return transmute(b);
    }
    unsafe fn make_slice<T: Sized>(&self, b:usize, num: usize) -> &'a [T] {
        if !self.range_valid(b, size_of::<T>() * num) {
            panic!("Cannot make array with {} elements at {}", num, b);
        }
        return slice::from_raw_parts(transmute(b), num);
    }
    /* Passing in a window to describe the window you want to create
     * is a little backwards, but it provides the nicest API as far
     * as I can tell */
    unsafe fn subwindow<'i, I, O: ?Sized>(&self, window: I) -> &'a O where O: VSpaceWindow<'a> + Default, I: VSpaceWindow<'i> {
        /* Validate the range for this window */
        if !self.range_valid(window.base(), window.size()) {
            panic!("Cannot construct window with range {} {}, from {} {}");
        }
        /* transmate to extend the lifetime. This is safe as
         * it is a zero sized object */
        transmute(&O::default())
    }
    fn range_valid(&self, b: usize, s: usize) -> bool {
        /* We have moved the `- s` to the other side of the equation
         * so that if `self.base() + self.size()` overflowed
         * then subtracting `s` will either correct if the range is
         * valid, or not correct in which case the range is invalid
         * and the comparison will fail */
        b >= self.base() && b <= (Wrapping(self.base()) + Wrapping(self.size()) - Wrapping(s)).0
    }
}
