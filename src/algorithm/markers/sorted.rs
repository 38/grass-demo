use crate::properties::WithRegion;
use crate::ChromName;

pub trait Sorted: Iterator{}

pub trait AssumeSorted: Iterator + Sized {
    fn assume_sorted<C: ChromName>(self) -> AssumingSortedIter<Self> 
    where
        Self::Item : WithRegion<C>
    {
        AssumingSortedIter { inner: self }
    }
}

impl<T: Iterator> AssumeSorted for T {}

pub struct AssumingSortedIter<T: Iterator> {
    inner: T,
}

impl<T: Iterator> Iterator for AssumingSortedIter<T> {
    type Item = T::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<T: Iterator> Sorted for AssumingSortedIter<T> {}

impl<T: Iterator + Sorted, P> Sorted for std::iter::Filter<T, P>
where
    P: Fn(&T::Item) -> bool,
{
}
