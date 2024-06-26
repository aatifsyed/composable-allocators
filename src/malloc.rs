use crate::prelude::*;
use core::{cmp, mem, ptr};
use libc::c_void;

/// An allocator using the OS-provided [`malloc`](https://man7.org/linux/man-pages/man3/malloc.3.html).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Malloc;

unsafe impl Allocator for Malloc {
    #[inline(always)]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let mut memptr = ptr::null_mut::<c_void>();
        match unsafe {
            libc::posix_memalign(
                &mut memptr,
                cmp::max(layout.align(), mem::size_of::<usize>()),
                layout.size(),
            )
        } {
            0 => match NonNull::new(memptr.cast::<u8>()) {
                Some(malloc) => Ok(NonNull::slice_from_raw_parts(malloc, layout.size())),
                None => unreachable!(),
            },
            libc::EINVAL => unreachable!(),
            libc::ENOMEM => Err(AllocError),
            _undocumented => Err(AllocError),
        }
    }

    #[inline(always)]
    unsafe fn deallocate(&self, free: NonNull<u8>, _: Layout) {
        libc::free(free.as_ptr().cast::<c_void>())
    }
}

#[test]
fn should_succeed() {
    let _ = Box::new_in(1, Malloc);
}
