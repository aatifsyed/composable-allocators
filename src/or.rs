use crate::prelude::*;

/// An [`Allocator`] which tries `PrimaryT`, and then `FallbackT` if it fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Or<PrimaryT, FallbackT> {
    pub primary: PrimaryT,
    pub fallback: FallbackT,
}

unsafe impl<PrimaryT, FallbackT> Allocator for Or<PrimaryT, FallbackT>
where
    PrimaryT: Allocator + Owns,
    FallbackT: Allocator,
{
    #[inline(always)]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.primary
            .allocate(layout)
            .or_else(|_| self.fallback.allocate(layout))
    }
    #[inline(always)]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        if self.primary.owns(ptr, layout) {
            self.primary.deallocate(ptr, layout)
        } else {
            self.fallback.deallocate(ptr, layout)
        }
    }
    #[inline(always)]
    fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.primary
            .allocate_zeroed(layout)
            .or_else(|_| self.fallback.allocate_zeroed(layout))
    }
    #[inline(always)]
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        if self.primary.owns(ptr, old_layout) {
            self.primary.grow(ptr, old_layout, new_layout)
        } else {
            self.fallback.grow(ptr, old_layout, new_layout)
        }
    }
    #[inline(always)]
    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        if self.primary.owns(ptr, old_layout) {
            self.primary.grow_zeroed(ptr, old_layout, new_layout)
        } else {
            self.fallback.grow_zeroed(ptr, old_layout, new_layout)
        }
    }
    #[inline(always)]
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        if self.primary.owns(ptr, old_layout) {
            self.primary.shrink(ptr, old_layout, new_layout)
        } else {
            self.fallback.shrink(ptr, old_layout, new_layout)
        }
    }
}

unsafe impl<PrimaryT, FallbackT> Owns for Or<PrimaryT, FallbackT>
where
    PrimaryT: Owns,
    FallbackT: Owns,
{
    #[inline(always)]
    fn owns(&self, ptr: NonNull<u8>, layout: Layout) -> bool {
        self.primary.owns(ptr, layout) || self.fallback.owns(ptr, layout)
    }
}

#[test]
fn test() {
    Box::try_new_in(1, Null.or(Null)).unwrap_err();
    #[cfg(feature = "malloc")]
    let _ = Box::new_in(1, Null.or(Malloc));
}
