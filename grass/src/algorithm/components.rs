use std::fmt::{Debug, Formatter, Result};
use std::iter::Enumerate;
use std::{
    cmp::{Ordering, Reverse},
    marker::PhantomData,
};
use std::{
    collections::{BinaryHeap, HashMap},
    hash::Hash,
};

use crate::{
    properties::{WithRegion, WithRegionCore},
    ChromName,
};

pub struct Point<C: ChromName, T: WithRegion<C>> {
    pub is_open: bool,
    pub index: usize,
    pub depth: usize,
    pub value: T,
    _p: PhantomData<C>,
}

impl<C: ChromName, T: WithRegion<C>> Debug for Point<C, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.is_open {
            write!(f, "Open(")?;
        } else {
            write!(f, "Close(")?;
        }

        let (chrom, pos) = self.position();

        write!(f, "{}, {}, {})", chrom.to_string(), pos, self.depth)
    }
}

impl<C: ChromName, T: WithRegion<C>> Point<C, T> {
    pub fn position(&self) -> (C, u32) {
        if self.is_open {
            (self.value.chrom().clone(), self.value.begin())
        } else {
            (self.value.chrom().clone(), self.value.end())
        }
    }
}

impl<C: ChromName, T: WithRegion<C>> PartialEq for Point<C, T> {
    fn eq(&self, other: &Point<C, T>) -> bool {
        self.position() == other.position()
    }
}

impl<C: ChromName, T: WithRegion<C>> PartialOrd for Point<C, T> {
    fn partial_cmp(&self, other: &Point<C, T>) -> Option<Ordering> {
        let ret = self
            .position()
            .cmp(&other.position())
            .then_with(|| self.is_open.cmp(&other.is_open));
        Some(ret)
    }
}

impl<C: ChromName, T: WithRegion<C>> Eq for Point<C, T> {}

impl<C: ChromName, T: WithRegion<C>> Ord for Point<C, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

pub struct ComponentsIter<C, I>
where
    C: ChromName,
    I: Iterator,
    I::Item: WithRegion<C> + Clone,
{
    iter: Enumerate<I>,
    peek_buffer: Option<(usize, I::Item)>,
    heap: BinaryHeap<Reverse<Point<C, I::Item>>>,
}

pub trait Components
where
    Self: Iterator + Sized,
{
    fn components<C: ChromName>(self) -> ComponentsIter<C, Self>
    where
        Self::Item: WithRegion<C> + Clone,
    {
        let mut iter = self.enumerate();
        let peek_buffer = iter.next();
        ComponentsIter {
            iter,
            peek_buffer,
            heap: BinaryHeap::new(),
        }
    }
}

impl<T> Components for T where T: Iterator + Sized {}

impl<C, I> Iterator for ComponentsIter<C, I>
where
    C: ChromName,
    I: Iterator,
    I::Item: WithRegion<C> + Clone,
{
    type Item = Point<C, I::Item>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((index, peek_buffer)) = self.peek_buffer.as_ref() {
            let index = *index;
            if self.heap.peek().map_or(false, |x| {
                x.0.position() < (peek_buffer.chrom().clone(), peek_buffer.begin())
            }) {
                let depth = self.heap.len();
                return self.heap.pop().map(|Reverse(mut x)| {
                    x.depth = depth - 1;
                    x
                });
            }
            let depth = self.heap.len() + 1;

            self.heap.push(Reverse(Point {
                index,
                depth: 0,
                value: peek_buffer.clone(),
                is_open: false,
                _p: PhantomData,
            }));
            let ret = Some(Point {
                index,
                depth,
                is_open: true,
                value: peek_buffer.clone(),
                _p: PhantomData,
            });
            self.peek_buffer = self.iter.next();
            ret
        } else {
            let depth = self.heap.len();
            self.heap.pop().map(|Reverse(mut x)| {
                x.depth = depth - 1;
                x
            })
        }
    }
}

pub struct TaggedComponent<C, I, R, T, F>
where
    C: ChromName,
    I: Iterator<Item = Point<C, R>>,
    R: WithRegion<C> + Clone,
    T: Clone + Hash + Eq,
    F: FnMut(&R) -> T,
{
    tag_func: F,
    state: HashMap<T, usize>,
    component_iter: I,
    _phantom: PhantomData<C>,
}

pub trait TaggedComponentExt<C: ChromName, R>
where
    R: WithRegion<C> + Clone,
    Self: Iterator<Item = Point<C, R>>,
{
    fn with_tag<T, F>(self, tag_func: F) -> TaggedComponent<C, Self, R, T, F>
    where
        T: Clone + Hash + Eq,
        F: FnMut(&R) -> T,
        Self: Sized,
    {
        TaggedComponent {
            tag_func,
            state: HashMap::new(),
            component_iter: self,
            _phantom: Default::default(),
        }
    }
}

impl<T, C, R> TaggedComponentExt<C, R> for T
where
    C: ChromName,
    R: WithRegion<C> + Clone,
    Self: Iterator<Item = Point<C, R>>,
{
}

impl<C, I, R, T, F> Iterator for TaggedComponent<C, I, R, T, F>
where
    C: ChromName,
    I: Iterator<Item = Point<C, R>>,
    R: WithRegion<C> + Clone,
    T: Clone + Hash + Eq,
    F: FnMut(&R) -> T,
{
    type Item = (T, Point<C, R>);
    fn next(&mut self) -> Option<Self::Item> {
        let mut next_comp = self.component_iter.next()?;
        let tag = (self.tag_func)(&next_comp.value);
        let tagged_depth = if next_comp.is_open {
            let cell = self.state.entry(tag.clone()).or_insert(0);
            *cell += 1;
            *cell
        } else {
            let depth = self
                .state
                .get_mut(&tag)
                .map(|depth| {
                    *depth -= 1;
                    *depth
                })
                .unwrap_or(0);
            if depth == 0 {
                self.state.remove(&tag);
            }
            depth
        };
        next_comp.depth = tagged_depth;
        Some((tag, next_comp))
    }
}
