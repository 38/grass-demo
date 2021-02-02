use crate::properties::WithRegion;

pub trait Sorted: Iterator
where
    Self::Item: WithRegion,
{
}

pub trait AssumeSorted: Iterator + Sized
where
    Self::Item: WithRegion,
{
    fn assume_sorted(self) -> AssumingSortedIter<Self> {
        AssumingSortedIter { inner: self }
    }
}

impl<T: Iterator> AssumeSorted for T where T::Item: WithRegion {}

pub struct AssumingSortedIter<T: Iterator>
where
    T::Item: WithRegion,
{
    inner: T,
}

impl<T: Iterator> Iterator for AssumingSortedIter<T>
where
    T::Item: WithRegion,
{
    type Item = T::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<T: Iterator> Sorted for AssumingSortedIter<T> where T::Item: WithRegion {}

impl<T: Iterator + Sorted, P> Sorted for std::iter::Filter<T, P>
where
    T::Item: WithRegion,
    P: Fn(&T::Item) -> bool,
{
}
