use crate::prelude::*;
use core::{cmp, ffi::c_void, mem, ptr};

/// An allocator using [`jemalloc`](https://jemalloc.net/).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Jemalloc;

unsafe impl Allocator for Jemalloc {
    #[inline(always)]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let mut memptr = ptr::null_mut::<c_void>();
        match unsafe {
            tikv_jemalloc_sys::posix_memalign(
                &mut memptr,
                cmp::max(layout.align(), mem::size_of::<usize>()),
                layout.size(),
            )
        } {
            0 => match NonNull::new(memptr.cast::<u8>()) {
                Some(it) => Ok(NonNull::slice_from_raw_parts(it, layout.size())),
                None => unreachable!(),
            },
            libc::EINVAL => unreachable!(),
            libc::ENOMEM => Err(AllocError),
            _undocumented => Err(AllocError),
        }
    }

    #[inline(always)]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, _: Layout) {
        tikv_jemalloc_sys::free(ptr.as_ptr().cast::<c_void>())
    }
}

#[test]
fn should_succeed() {
    let _ = Box::new_in(1, Jemalloc);
}
