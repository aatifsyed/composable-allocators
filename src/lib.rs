#![no_std]

use allocator_api2::alloc::Allocator;
use core::ptr::NonNull;

#[cfg(feature = "malloc")]
mod malloc;
#[cfg(feature = "malloc")]
pub use malloc::Malloc;
#[cfg(feature = "jemalloc")]
mod jemalloc;
#[cfg(feature = "jemalloc")]
pub use jemalloc::Jemalloc;
#[cfg(feature = "mimalloc")]
mod mimalloc;
#[cfg(feature = "mimalloc")]
pub use mimalloc::Mimalloc;

mod limit;
pub use limit::{CountLimit, SizeLimit};
mod null;
pub use null::Null;
mod or;
pub use or::Or;

mod prelude {
    pub(crate) use crate::*;
    pub(crate) use allocator_api2::alloc::{AllocError, Allocator};
    #[cfg(test)]
    pub(crate) use allocator_api2::boxed::Box;
    pub(crate) use core::{alloc::Layout, ptr::NonNull};
}

/// Whether it is safe to call [`Allocator::deallocate`].
///
/// # Safety
/// - unsafe code may rely on correct implementations
pub unsafe trait Owns {
    fn owns(&self, ptr: NonNull<u8>) -> bool;
}

/// Extension traits for [`Allocator`].
pub trait AllocatorExt: Allocator {
    fn or<A: Allocator>(self, fallback: A) -> Or<Self, A>
    where
        Self: Sized,
    {
        Or {
            primary: self,
            fallback,
        }
    }
    fn limit_size(self, limit: usize) -> SizeLimit<Self>
    where
        Self: Sized,
    {
        SizeLimit {
            inner: self,
            limit: limit.into(),
        }
    }
    fn limit_count(self, limit: usize) -> CountLimit<Self>
    where
        Self: Sized,
    {
        CountLimit {
            inner: self,
            limit: limit.into(),
        }
    }
}
impl<A> AllocatorExt for A where A: Allocator {}
