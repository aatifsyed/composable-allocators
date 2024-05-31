use crate::prelude::*;
use core::sync::atomic::{AtomicUsize, Ordering};

/// An [`Allocator`] which allows `A` to allocate at most [`limit`](Self::limit) bytes.
#[derive(Debug)]
pub struct SizeLimit<A> {
    pub inner: A,
    pub limit: AtomicUsize,
}
unsafe impl<A> Allocator for SizeLimit<A>
where
    A: Allocator,
{
    #[inline(always)]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        match self
            .limit
            .fetch_update(Ordering::Release, Ordering::Acquire, |it| {
                it.checked_sub(layout.size())
            }) {
            Ok(_) => self.inner.allocate(layout),
            Err(_) => Err(AllocError),
        }
    }
    #[inline(always)]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        self.limit.fetch_sub(layout.size(), Ordering::Release);
        self.inner.deallocate(ptr, layout)
    }
}
unsafe impl<A> Owns for SizeLimit<A>
where
    A: Owns,
{
    #[inline(always)]
    fn owns(&self, ptr: NonNull<u8>, layout: Layout) -> bool {
        self.inner.owns(ptr, layout)
    }
}

#[cfg(feature = "malloc")]
#[test]
fn limit() {
    let a = Malloc.limit_size(1);
    let occupied = Box::new_in(1u8, &a);
    Box::try_new_in(1u8, &a).unwrap_err();
    drop(occupied);
    let _ = Box::new_in(1u8, &a);
}

#[derive(Debug)]
/// An [`Allocator`] which allows `A` to allocate at most [`limit`](Self::limit) times.
pub struct CountLimit<A> {
    pub inner: A,
    pub limit: AtomicUsize,
}

unsafe impl<A> Allocator for CountLimit<A>
where
    A: Allocator,
{
    #[inline(always)]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        match self
            .limit
            .fetch_update(Ordering::Release, Ordering::Acquire, |it| it.checked_sub(1))
        {
            Ok(_) => self.inner.allocate(layout),
            Err(_) => Err(AllocError),
        }
    }
    #[inline(always)]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        self.limit.fetch_add(1, Ordering::Release);
        self.inner.deallocate(ptr, layout)
    }
}

unsafe impl<A> Owns for CountLimit<A>
where
    A: Owns,
{
    #[inline(always)]
    fn owns(&self, ptr: NonNull<u8>, layout: Layout) -> bool {
        self.inner.owns(ptr, layout)
    }
}

#[cfg(feature = "malloc")]
#[test]
fn count() {
    let a = Malloc.limit_count(1);
    let occupied = Box::new_in(1, &a);
    Box::try_new_in(1, &a).unwrap_err();
    drop(occupied);
    let _ = Box::new_in(1, &a);
}
