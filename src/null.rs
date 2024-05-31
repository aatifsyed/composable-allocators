use allocator_api2::alloc::{AllocError, Allocator};
#[cfg(test)]
use allocator_api2::boxed::Box;
use core::{alloc::Layout, ptr::NonNull};

/// An [`Allocator`] which never allocates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Null;

unsafe impl Allocator for Null {
    #[inline(always)]
    fn allocate(&self, _: Layout) -> Result<NonNull<[u8]>, AllocError> {
        Err(AllocError)
    }
    #[inline(always)]
    unsafe fn deallocate(&self, _: NonNull<u8>, _: Layout) {
        panic!("Null allocator should never be asked to deallocate")
    }
}

unsafe impl crate::Owns for Null {
    #[inline(always)]
    fn owns(&self, _: NonNull<u8>, _: Layout) -> bool {
        false
    }
}

#[test]
fn should_fail() {
    Box::try_new_in(1, Null).unwrap_err();
}
