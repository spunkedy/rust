use crate::iter::adapters::{
    zip::try_get_unchecked, SourceIter, TrustedRandomAccess, TrustedRandomAccessNoCoerce,
};
use crate::iter::{FusedIterator, InPlaceIterable, TrustedLen};
use crate::ops::{Deref, DerefMut, Try};

/// An adapter that converts an iterator over `&T` to an iterator over
/// `&T::Target`, coercing each item using [`Deref`].
///
/// This `struct` is created by the [`as_deref`] method on [`Iterator`]. See its
/// documentation for more.
///
/// [`as_deref`]: Iterator::as_deref
/// [`Iterator`]: trait.Iterator.html
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[unstable(feature = "iter_as_deref", reason = "recently added", issue = "none")]
#[derive(Clone, Debug)]
pub struct AsDeref<I> {
    iter: I,
}

impl<I> AsDeref<I> {
    #[inline]
    pub(in crate::iter) fn new(iter: I) -> AsDeref<I> {
        AsDeref { iter }
    }
}

fn deref_fold<'a, B: Deref + ?Sized + 'a, Acc>(
    mut g: impl FnMut(Acc, &'a B::Target) -> Acc,
) -> impl FnMut(Acc, &'a B) -> Acc {
    move |acc, elt| g(acc, &**elt)
}

fn deref_try_fold<'a, B: Deref + ?Sized + 'a, Acc, R>(
    mut g: impl FnMut(Acc, &'a B::Target) -> R,
) -> impl FnMut(Acc, &'a B) -> R {
    move |acc, elt| g(acc, &**elt)
}

#[unstable(feature = "iter_as_deref", reason = "recently added", issue = "none")]
impl<'a, B, I> Iterator for AsDeref<I>
where
    I: Iterator<Item = &'a B>,
    B: Deref + ?Sized + 'a,
{
    type Item = &'a B::Target;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(v) => Some(&**v),
            None => None,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn try_fold<Acc, G, R>(&mut self, init: Acc, g: G) -> R
    where
        Self: Sized,
        G: FnMut(Acc, Self::Item) -> R,
        R: Try<Output = Acc>,
    {
        self.iter.try_fold(init, deref_try_fold(g))
    }

    fn fold<Acc, G>(self, init: Acc, g: G) -> Acc
    where
        G: FnMut(Acc, Self::Item) -> Acc,
    {
        self.iter.fold(init, deref_fold(g))
    }

    #[doc(hidden)]
    #[inline]
    unsafe fn __iterator_get_unchecked(&mut self, idx: usize) -> Self::Item
    where
        Self: TrustedRandomAccessNoCoerce,
    {
        // SAFETY: the caller must uphold the contract for
        // `Iterator::__iterator_get_unchecked`.
        unsafe { &**try_get_unchecked(&mut self.iter, idx) }
    }
}

#[unstable(feature = "iter_as_deref", reason = "recently added", issue = "none")]
impl<'a, B, I> DoubleEndedIterator for AsDeref<I>
where
    I: DoubleEndedIterator<Item = &'a B>,
    B: Deref + ?Sized + 'a,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.iter.next_back() {
            Some(v) => Some(&**v),
            None => None,
        }
    }

    fn try_rfold<Acc, G, R>(&mut self, init: Acc, g: G) -> R
    where
        Self: Sized,
        G: FnMut(Acc, Self::Item) -> R,
        R: Try<Output = Acc>,
    {
        self.iter.try_rfold(init, deref_try_fold(g))
    }

    fn rfold<Acc, G>(self, init: Acc, g: G) -> Acc
    where
        G: FnMut(Acc, Self::Item) -> Acc,
    {
        self.iter.rfold(init, deref_fold(g))
    }
}

#[unstable(feature = "iter_as_deref", reason = "recently added", issue = "none")]
impl<'a, B, I> ExactSizeIterator for AsDeref<I>
where
    I: ExactSizeIterator<Item = &'a B>,
    B: Deref + ?Sized + 'a,
{
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.iter.is_empty()
    }
}

#[unstable(feature = "iter_as_deref", reason = "recently added", issue = "none")]
impl<'a, B, I> FusedIterator for AsDeref<I>
where
    B: Deref + ?Sized + 'a,
    I: FusedIterator<Item = &'a B>,
{
}

#[unstable(feature = "trusted_len", issue = "37572")]
unsafe impl<'a, B, I> TrustedLen for AsDeref<I>
where
    I: TrustedLen<Item = &'a B>,
    B: Deref + ?Sized + 'a,
{
}

#[doc(hidden)]
#[unstable(feature = "trusted_random_access", issue = "none")]
unsafe impl<I> TrustedRandomAccess for AsDeref<I> where I: TrustedRandomAccess {}

#[doc(hidden)]
#[unstable(feature = "trusted_random_access", issue = "none")]
unsafe impl<I: TrustedRandomAccessNoCoerce> TrustedRandomAccessNoCoerce for AsDeref<I> {
    const MAY_HAVE_SIDE_EFFECT: bool = true;
}

#[unstable(issue = "none", feature = "inplace_iteration")]
unsafe impl<I> SourceIter for AsDeref<I>
where
    I: SourceIter,
{
    type Source = I::Source;

    #[inline]
    unsafe fn as_inner(&mut self) -> &mut I::Source {
        // SAFETY: unsafe function forwarding to unsafe function with the same requirements
        unsafe { SourceIter::as_inner(&mut self.iter) }
    }
}

#[unstable(issue = "none", feature = "inplace_iteration")]
unsafe impl<'a, B, I> InPlaceIterable for AsDeref<I>
where
    B: Deref + ?Sized + 'a,
    I: InPlaceIterable<Item = &'a B>,
{
}

/// An adapter that converts an iterator over `&mut T` to an iterator over
/// `&mut T::Target`, coercing each item using [`DerefMut`].
///
/// This `struct` is created by the [`as_deref_mut`] method on [`Iterator`]. See
/// its documentation for more.
///
/// [`as_deref_mut`]: Iterator::as_deref_mut
/// [`Iterator`]: trait.Iterator.html
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[unstable(feature = "iter_as_deref", reason = "recently added", issue = "none")]
#[derive(Clone, Debug)]
pub struct AsDerefMut<I> {
    iter: I,
}

impl<I> AsDerefMut<I> {
    #[inline]
    pub(in crate::iter) fn new(iter: I) -> AsDerefMut<I> {
        AsDerefMut { iter }
    }
}

fn deref_mut_fold<'a, B: DerefMut + ?Sized + 'a, Acc>(
    mut g: impl FnMut(Acc, &'a mut B::Target) -> Acc,
) -> impl FnMut(Acc, &'a mut B) -> Acc {
    move |acc, elt| g(acc, &mut **elt)
}

fn deref_mut_try_fold<'a, B: DerefMut + ?Sized + 'a, Acc, R>(
    mut g: impl FnMut(Acc, &'a mut B::Target) -> R,
) -> impl FnMut(Acc, &'a mut B) -> R {
    move |acc, elt| g(acc, &mut **elt)
}

#[unstable(feature = "iter_as_deref", reason = "recently added", issue = "none")]
impl<'a, B, I> Iterator for AsDerefMut<I>
where
    I: Iterator<Item = &'a mut B>,
    B: DerefMut + ?Sized + 'a,
{
    type Item = &'a mut B::Target;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(v) => Some(&mut **v),
            None => None,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn try_fold<Acc, G, R>(&mut self, init: Acc, g: G) -> R
    where
        Self: Sized,
        G: FnMut(Acc, Self::Item) -> R,
        R: Try<Output = Acc>,
    {
        self.iter.try_fold(init, deref_mut_try_fold(g))
    }

    fn fold<Acc, G>(self, init: Acc, g: G) -> Acc
    where
        G: FnMut(Acc, Self::Item) -> Acc,
    {
        self.iter.fold(init, deref_mut_fold(g))
    }

    #[doc(hidden)]
    #[inline]
    unsafe fn __iterator_get_unchecked(&mut self, idx: usize) -> Self::Item
    where
        Self: TrustedRandomAccessNoCoerce,
    {
        // SAFETY: the caller must uphold the contract for
        // `Iterator::__iterator_get_unchecked`.
        unsafe { &mut **try_get_unchecked(&mut self.iter, idx) }
    }
}

#[unstable(feature = "iter_as_deref", reason = "recently added", issue = "none")]
impl<'a, B, I> DoubleEndedIterator for AsDerefMut<I>
where
    I: DoubleEndedIterator<Item = &'a mut B>,
    B: DerefMut + ?Sized + 'a,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.iter.next_back() {
            Some(v) => Some(&mut **v),
            None => None,
        }
    }

    fn try_rfold<Acc, G, R>(&mut self, init: Acc, g: G) -> R
    where
        Self: Sized,
        G: FnMut(Acc, Self::Item) -> R,
        R: Try<Output = Acc>,
    {
        self.iter.try_rfold(init, deref_mut_try_fold(g))
    }

    fn rfold<Acc, G>(self, init: Acc, g: G) -> Acc
    where
        G: FnMut(Acc, Self::Item) -> Acc,
    {
        self.iter.rfold(init, deref_mut_fold(g))
    }
}

#[unstable(feature = "iter_as_deref", reason = "recently added", issue = "none")]
impl<'a, B, I> ExactSizeIterator for AsDerefMut<I>
where
    I: ExactSizeIterator<Item = &'a mut B>,
    B: DerefMut + ?Sized + 'a,
{
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.iter.is_empty()
    }
}

#[unstable(feature = "iter_as_deref", reason = "recently added", issue = "none")]
impl<'a, B, I> FusedIterator for AsDerefMut<I>
where
    B: DerefMut + ?Sized + 'a,
    I: FusedIterator<Item = &'a mut B>,
{
}

#[unstable(feature = "trusted_len", issue = "37572")]
unsafe impl<'a, B, I> TrustedLen for AsDerefMut<I>
where
    I: TrustedLen<Item = &'a mut B>,
    B: DerefMut + ?Sized + 'a,
{
}

#[doc(hidden)]
#[unstable(feature = "trusted_random_access", issue = "none")]
unsafe impl<I> TrustedRandomAccess for AsDerefMut<I> where I: TrustedRandomAccess {}

#[doc(hidden)]
#[unstable(feature = "trusted_random_access", issue = "none")]
unsafe impl<I: TrustedRandomAccessNoCoerce> TrustedRandomAccessNoCoerce for AsDerefMut<I> {
    const MAY_HAVE_SIDE_EFFECT: bool = true;
}

#[unstable(issue = "none", feature = "inplace_iteration")]
unsafe impl<I> SourceIter for AsDerefMut<I>
where
    I: SourceIter,
{
    type Source = I::Source;

    #[inline]
    unsafe fn as_inner(&mut self) -> &mut I::Source {
        // SAFETY: unsafe function forwarding to unsafe function with the same requirements
        unsafe { SourceIter::as_inner(&mut self.iter) }
    }
}

#[unstable(issue = "none", feature = "inplace_iteration")]
unsafe impl<'a, B, I> InPlaceIterable for AsDerefMut<I>
where
    B: DerefMut + ?Sized + 'a,
    I: InPlaceIterable<Item = &'a mut B>,
{
}
