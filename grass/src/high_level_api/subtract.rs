use std::{cell::RefCell, iter::Peekable, rc::Rc};

use crate::{
    algorithm::{Components, ComponentsIter, Sorted, SortedIntersect, SortedIntersectIter},
    properties::{WithRegion, WithRegionCore},
    records::Bed3,
    ChromName,
};

pub struct InvertedIter<C, A>
where
    A: Iterator + Sorted,
    C: ChromName + Clone,
    A::Item: WithRegion<C> + Clone,
{
    iter: Peekable<ComponentsIter<C, A>>,
    last_chrom: Option<C>,
}

pub trait InvertExt<C>
where
    Self: Iterator + Sorted + Sized,
    C: ChromName + Clone,
    Self::Item: WithRegion<C> + Clone,
{
    fn invert(self) -> InvertedIter<C, Self> {
        InvertedIter {
            iter: self.components().peekable(),
            last_chrom: None,
        }
    }
}

impl<T, C> InvertExt<C> for T
where
    T: Iterator + Sorted + Sized,
    C: ChromName,
    T::Item: WithRegion<C> + Clone,
{
}

impl<C, A> Iterator for InvertedIter<C, A>
where
    A: Iterator + Sorted,
    C: ChromName,
    A::Item: WithRegion<C> + Clone,
{
    type Item = Bed3<C>;
    fn next(&mut self) -> Option<Self::Item> {
        let next_comp = self.iter.peek()?;
        let is_fresh_chrom = self
            .last_chrom
            .as_ref()
            .map_or(true, |chrom| chrom != next_comp.value.chrom());

        if is_fresh_chrom {
            self.last_chrom = Some(next_comp.value.chrom().clone());
            if next_comp.value.begin() > 0 {
                return Some(Bed3 {
                    chrom: next_comp.value.chrom().clone(),
                    begin: 0u32,
                    end: next_comp.value.begin(),
                });
            }
        }

        let begin = loop {
            let next = self.iter.next()?;
            if next.depth == 0 {
                break next.value.end();
            }
        };

        if let Some(next) = self.iter.peek() {
            let end = if next.value.chrom() == self.last_chrom.as_ref().unwrap() {
                next.value.begin()
            } else {
                u32::MAX
            };

            if end - begin > 0 {
                return Some(Bed3 {
                    chrom: self.last_chrom.clone().unwrap(),
                    begin,
                    end,
                });
            }

            self.next()
        } else {
            Some(Bed3 {
                chrom: self.last_chrom.clone().unwrap(),
                begin,
                end: u32::MAX,
            })
        }
    }
}

struct SubstractData<C, A, B>
where
    A: Iterator + Sorted,
    B: Iterator + Sorted,
    C: ChromName,
    A::Item: WithRegion<C> + Clone,
    B::Item: WithRegion<C> + Clone,
{
    iter_a: Peekable<A>,
    iter_b: Peekable<InvertedIter<C, B>>,
    known_chrom: Vec<C>,
}

pub struct SubstractIterA<C, A, B>
where
    A: Iterator + Sorted,
    B: Iterator + Sorted,
    C: ChromName,
    A::Item: WithRegion<C> + Clone,
    B::Item: WithRegion<C> + Clone,
{
    core: Rc<RefCell<SubstractData<C, A, B>>>,
}

impl<C, A, B> Sorted for SubstractIterA<C, A, B>
where
    A: Iterator + Sorted,
    B: Iterator + Sorted,
    C: ChromName,
    A::Item: WithRegion<C> + Clone,
    B::Item: WithRegion<C> + Clone,
{
}

impl<C, A, B> Iterator for SubstractIterA<C, A, B>
where
    A: Iterator + Sorted,
    B: Iterator + Sorted,
    C: ChromName,
    A::Item: WithRegion<C> + Clone,
    B::Item: WithRegion<C> + Clone,
{
    type Item = A::Item;
    fn next(&mut self) -> Option<Self::Item> {
        let core = &mut self.core.borrow_mut();

        let ret = core.iter_a.next()?;

        if core
            .known_chrom
            .last()
            .map_or(true, |last| last < ret.chrom())
        {
            core.known_chrom.push(ret.chrom().clone());
        }

        if let Some(next) = core.iter_a.peek() {
            let chrom = next.chrom().clone();
            if core.known_chrom.last().map_or(true, |last| last < &chrom) {
                core.known_chrom.push(chrom);
            }
        }

        Some(ret)
    }
}

pub struct SubstractIterB<C, A, B>
where
    A: Iterator + Sorted,
    B: Iterator + Sorted,
    C: ChromName,
    A::Item: WithRegion<C> + Clone,
    B::Item: WithRegion<C> + Clone,
{
    core: Rc<RefCell<SubstractData<C, A, B>>>,
    last_chrom: Option<usize>,
}

impl<C, A, B> Sorted for SubstractIterB<C, A, B>
where
    A: Iterator + Sorted,
    B: Iterator + Sorted,
    C: ChromName,
    A::Item: WithRegion<C> + Clone,
    B::Item: WithRegion<C> + Clone,
{
}

impl<C, A, B> Iterator for SubstractIterB<C, A, B>
where
    A: Iterator + Sorted,
    B: Iterator + Sorted,
    C: ChromName,
    A::Item: WithRegion<C> + Clone,
    B::Item: WithRegion<C> + Clone,
{
    type Item = Bed3<C>;
    fn next(&mut self) -> Option<Self::Item> {
        let core = &mut self.core.borrow_mut();
        let current_chrom = self
            .last_chrom
            .map_or(core.known_chrom.get(0), |id| core.known_chrom.get(id))?
            .clone();
        while let Some(next_chrom) = core.iter_b.peek().map(|x| x.chrom().clone()) {
            if next_chrom < current_chrom {
                core.iter_b.next();
            } else if next_chrom == current_chrom {
                let next = core.iter_b.next()?;
                return Some(next);
            } else {
                break;
            }
        }

        if self.last_chrom.map_or(core.known_chrom.len() > 0, |cid| {
            cid < core.known_chrom.len() - 1
        }) {
            let idx = self.last_chrom.map_or(0, |x| x + 1);
            self.last_chrom = Some(idx);
            return Some(Bed3 {
                chrom: core.known_chrom[idx].clone(),
                begin: 0,
                end: u32::MAX,
            });
        }

        None
    }
}

pub trait SubtractExt<C: ChromName>
where
    Self: Iterator + Sorted + Sized,
    Self::Item: WithRegion<C> + Clone,
{
    fn subtract<T>(
        self,
        other: T,
    ) -> SortedIntersectIter<C, SubstractIterA<C, Self, T>, SubstractIterB<C, Self, T>>
    where
        T: Iterator + Sorted,
        T::Item: WithRegion<C> + Clone,
    {
        let core_data = Rc::new(RefCell::new(SubstractData {
            iter_a: self.peekable(),
            iter_b: other.invert().peekable(),
            known_chrom: vec![],
        }));

        let iter_a = SubstractIterA {
            core: core_data.clone(),
        };

        let iter_b = SubstractIterB {
            core: core_data,
            last_chrom: None,
        };

        iter_a.sorted_intersect(iter_b)
    }
}

impl<C: ChromName, T> SubtractExt<C> for T
where
    Self: Iterator + Sorted,
    Self::Item: WithRegion<C> + Clone,
{
}
