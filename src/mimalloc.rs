use crate::prelude::*;
use core::ffi::c_void;

/// An allocator using [`mimalloc`](https://github.com/microsoft/mimalloc).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Mimalloc;

unsafe impl Allocator for Mimalloc {
    #[inline(always)]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        match NonNull::new(unsafe {
            libmimalloc_sys::mi_aligned_alloc(layout.align(), layout.size())
        }) {
            Some(it) => Ok(NonNull::slice_from_raw_parts(
                it.cast::<u8>(),
                layout.size(),
            )),
            None => Err(AllocError),
        }
    }

    #[inline(always)]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, _: Layout) {
        libmimalloc_sys::mi_free(ptr.as_ptr().cast::<c_void>())
    }
}

unsafe impl Owns for Mimalloc {
    #[inline(always)]
    fn owns(&self, ptr: NonNull<u8>) -> bool {
        // TODO(aatifsyed): what does "default heap of this thread" mean?
        unsafe { libmimalloc_sys::mi_check_owned(ptr.as_ptr().cast::<c_void>()) }
    }
}

#[test]
fn should_succeed() {
    Box::try_new_in(1, Mimalloc).unwrap();
}
