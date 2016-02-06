use core::intrinsics::transmute;

pub unsafe trait VSpaceWindow<'a> {
    fn base(&self) -> usize;
    fn size(&self) -> usize;
    unsafe fn make<T>(&self) -> &'a T {
        unimplemented!()
    }
    /* Passing in a window to describe the window you want to create
     * is a little backwards, but it provides the nicest API as far
     * as I can tell */
    unsafe fn subwindow<'i, I, O: ?Sized>(&self, window: I) -> &'a O where O: VSpaceWindow<'a> + Default, I: VSpaceWindow<'i> {
        /* transmate to extend the lifetime. This is safe as
         * it is a zero sized object */
        transmute(&O::default())
    }
}
