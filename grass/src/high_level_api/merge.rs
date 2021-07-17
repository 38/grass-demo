use std::{collections::HashMap, hash::Hash, rc::Rc};

use crate::{
    algorithm::{Components, ComponentsIter, Point, Sorted, TaggedComponent, TaggedComponentExt},
    chromset::LexicalChromRef,
    properties::{WithRegion, WithRegionCore},
    records::{Bed3, Bed4},
};

//use super::{Components, ComponentsIter, Point, Sorted};

pub struct Merger<I: Iterator + Sorted>
where
    I::Item: WithRegion<LexicalChromRef> + Clone,
{
    peek: Option<Point<LexicalChromRef, I::Item>>,
    iter: ComponentsIter<LexicalChromRef, I>,
}

impl<I: Iterator + Sorted> Iterator for Merger<I>
where
    I::Item: WithRegion<LexicalChromRef> + Clone,
{
    type Item = Bed3<LexicalChromRef>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut begin = None;
        let mut end;
        while self.peek.as_ref().map_or(false, |peek| peek.depth > 0) {
            let mut buf = self.iter.next();
            std::mem::swap(&mut buf, &mut self.peek);
            end = Some(buf.unwrap().value.clone());
            if begin.is_none() {
                begin = end.clone();
            }
        }
        end = self.peek.take().map(|x| x.value);
        self.peek = self.iter.next();
        if let Some(ret) = begin {
            let mut ret = Bed3::new(ret);
            ret.end = end.unwrap().end();
            Some(ret)
        } else {
            None
        }
    }
}

pub struct TaggedMerger<I, T> {
    iter: I,
    chrom: Option<LexicalChromRef>,
    begins: HashMap<T, u32>,
}

impl<I, R, T> Iterator for TaggedMerger<I, T>
where
    I: Iterator<Item = (T, Point<LexicalChromRef, R>)>,
    R: WithRegion<LexicalChromRef> + Clone,
    T: ToString + Eq + Hash,
{
    type Item = Bed4<LexicalChromRef>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some((tag, comp)) = self.iter.next() {
            let (chr, pos) = comp.position();
            if Some(&chr) != self.chrom.as_ref() {
                self.begins.clear();
            }
            self.chrom = Some(chr);
            if let Some(&begin) = self.begins.get(&tag) {
                if !comp.is_open {
                    let core = Bed3 {
                        chrom: self.chrom.clone().unwrap(),
                        begin,
                        end: pos,
                    };
                    let result = Bed4 {
                        core,
                        name: Rc::new(tag.to_string()),
                    };
                    self.begins.remove(&tag);
                    return Some(result);
                }
            } else {
                self.begins.entry(tag).or_insert(pos);
            }
        }
        None
    }
}

pub trait MergeExt
where
    Self: IntoIterator + Sized,
    <Self as IntoIterator>::IntoIter: Sorted,
    <Self as IntoIterator>::Item: WithRegion<LexicalChromRef> + Clone,
{
    fn merge_overlaps(self) -> Merger<<Self as IntoIterator>::IntoIter> {
        let mut iter = self.into_iter().components();
        let peek = iter.next();
        Merger { iter, peek }
    }
    fn tagged_merge<T: Clone + Hash + Eq, F: FnMut(&Self::Item) -> T>(
        self,
        f: F,
    ) -> TaggedMerger<
        TaggedComponent<
            LexicalChromRef,
            ComponentsIter<LexicalChromRef, Self::IntoIter>,
            Self::Item,
            T,
            F,
        >,
        T,
    > {
        TaggedMerger {
            iter: self.into_iter().components().with_tag(f),
            begins: HashMap::new(),
            chrom: None,
        }
    }
}
impl<T: IntoIterator + Sized> MergeExt for T
where
    T::IntoIter: Sorted,
    T::Item: WithRegion<LexicalChromRef> + Clone,
{
}
