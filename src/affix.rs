use crate::prelude::*;
use core::{marker::PhantomData, ptr};

/// ```text
/// ┌─────────────────────────────────────────┐
/// │ outer                                   │
/// ├──────────────┬────────────────┬─────────┤
/// │ PrefixT      │ body           │ SuffixT │
/// ├──────────────┼────────────────┴─────────┘
/// ├─body_offset─►│                :         :
/// │              :                :         :
/// └────────────────suffix_offset─►│         :
/// :                                         :
/// ├─outer.size()───────────────────────────►│
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct AffixLayout {
    pub body_offset: usize,
    pub suffix_offset: usize,
    pub outer: Layout,
}

impl AffixLayout {
    pub fn new<PrefixT, SuffixT>(body: Layout) -> Option<Self> {
        let (outer, body_offset) = Layout::new::<PrefixT>().extend(body).ok()?;
        let (outer, suffix_offset) = outer.extend(Layout::new::<SuffixT>()).ok()?;
        let outer = outer.pad_to_align();
        Some(AffixLayout {
            body_offset,
            suffix_offset,
            outer,
        })
    }
    /// # Safety
    /// - `outer` must be from an [`Allocator::allocate`] call with [`Self::outer`].
    pub unsafe fn narrow(&self, outer: NonNull<[u8]>) -> NonNull<[u8]> {
        let ptr = outer.as_ptr().cast::<u8>().byte_add(self.body_offset);
        NonNull::slice_from_raw_parts(NonNull::new_unchecked(ptr), self.suffix_offset)
    }
    /// # Safety
    /// - `body` must be from a call to [`Affix::affix_allocate`].
    pub unsafe fn broaden(&self, body: NonNull<u8>) -> (NonNull<u8>, NonNull<u8>) {
        let prefix = NonNull::new_unchecked(body.as_ptr().byte_sub(self.body_offset));
        (
            prefix,
            NonNull::new_unchecked(prefix.as_ptr().byte_add(self.suffix_offset)),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Affix<A, PrefixT, SuffixT> {
    pub inner: A,
    pub prefix: PhantomData<fn() -> PrefixT>,
    pub suffix: PhantomData<fn() -> SuffixT>,
}

impl<A, PrefixT, SuffixT> Affix<A, PrefixT, SuffixT>
where
    A: Allocator,
{
    #[allow(clippy::type_complexity)]
    pub fn affix_allocate(
        &self,
        body: Layout,
    ) -> Result<(NonNull<u8>, NonNull<[u8]>, NonNull<u8>), AllocError> {
        let affix_layout = AffixLayout::new::<PrefixT, SuffixT>(body).ok_or(AllocError)?;
        let outer = self.inner.allocate(affix_layout.outer)?;
        debug_assert!(outer.len() >= affix_layout.outer.size());
        let body = unsafe { affix_layout.narrow(outer) };
        let (prefix, suffix) = unsafe { affix_layout.broaden(body.cast::<u8>()) };
        Ok((prefix, body, suffix))
    }
    /// # Safety
    /// - `body` must be from a call to [`Self::affix_allocate`].
    pub unsafe fn affix_get(body: NonNull<u8>, layout: Layout) -> (NonNull<u8>, NonNull<u8>) {
        AffixLayout::new::<PrefixT, SuffixT>(layout)
            .unwrap_unchecked()
            .broaden(body)
    }
}

unsafe impl<A, PrefixT, SuffixT> Allocator for Affix<A, PrefixT, SuffixT>
where
    A: Allocator,
{
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let (_, body, _) = self.affix_allocate(layout)?;
        Ok(body)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        let affix_layout = AffixLayout::new::<PrefixT, SuffixT>(layout).unwrap_unchecked();
        let (start, _) = affix_layout.broaden(ptr);
        self.inner.deallocate(start, affix_layout.outer)
    }
}

unsafe impl<A, PrefixT, SuffixT> Owns for Affix<A, PrefixT, SuffixT>
where
    A: Owns,
{
    fn owns(&self, ptr: NonNull<u8>, layout: Layout) -> bool {
        match AffixLayout::new::<PrefixT, SuffixT>(layout) {
            Some(affix_layout) => {
                // BUG(aatifsyed): this is bad
                let (ptr, _) = unsafe { affix_layout.broaden(ptr) };
                self.inner.owns(ptr, affix_layout.outer)
            }
            None => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Guard<A, PrefixT, SuffixT> {
    pub inner: Affix<A, PrefixT, SuffixT>,
    pub prefix: PrefixT,
    pub suffix: SuffixT,
}

unsafe impl<A, PrefixT, SuffixT> Allocator for Guard<A, PrefixT, SuffixT>
where
    A: Allocator,
    PrefixT: Copy + PartialEq,
    SuffixT: Copy + PartialEq,
{
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let (prefix, body, suffix) = self.inner.affix_allocate(layout)?;
        unsafe { ptr::write(prefix.as_ptr().cast::<PrefixT>(), self.prefix) };
        unsafe { ptr::write(suffix.as_ptr().cast::<SuffixT>(), self.suffix) };
        Ok(body)
    }

    unsafe fn deallocate(&self, body: NonNull<u8>, layout: Layout) {
        let affix_layout = AffixLayout::new::<PrefixT, SuffixT>(layout).unwrap_unchecked();
        let (prefix, suffix) = affix_layout.broaden(body);
        let prefix = ptr::read(prefix.cast::<PrefixT>().as_ptr());
        let suffix = ptr::read(suffix.cast::<SuffixT>().as_ptr());
        if prefix != self.prefix {
            panic!("prefix guard doesn't match")
        }
        if suffix != self.suffix {
            panic!("suffix guard doesn't match")
        }
        self.inner.deallocate(body, layout)
    }
}

#[cfg(feature = "malloc")]
#[test]
fn guard() {
    let _ = Box::new_in(1, Malloc.zero().guard([0xFF_u8; 3], [0xEE_u8; 3]));
}
