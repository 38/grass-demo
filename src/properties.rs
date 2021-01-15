use std::cmp::Ordering;
use std::io::{Result, Write};
use std::ops::Deref;

pub trait Parsable: Sized {
    fn parse<'a, T: Iterator<Item = &'a str>>(token_stream: &mut T) -> Option<Self>;
}

pub trait Serializable {
    fn dump<W: Write>(&self, fp: W) -> Result<()>;
}

pub trait WithRegion {
    fn left(&self) -> u32;
    fn right(&self) -> u32;
    fn chrom(&self) -> &str;
    fn overlaps(&self, b: &impl WithRegion) -> bool {
        let a = self;
        if a.chrom() != b.chrom() {
            return false;
        }

        !(a.right() <= b.left() || b.right() <= a.left())
    }
    #[allow(dead_code)]
    fn empty(&self) -> bool {
        self.right() <= self.left()
    }
    #[allow(dead_code)]
    fn length(&self) -> u32 {
        self.right().max(self.left()) - self.left()
    }
}

pub struct RegionOrder<T: WithRegion>(pub T);

impl<T: WithRegion> PartialEq for RegionOrder<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.chrom() == other.0.chrom()
            && self.0.left() == other.0.left()
            && self.0.right() == other.0.right()
    }
}

impl<T: WithRegion> Eq for RegionOrder<T> {}

impl<T: WithRegion> Ord for RegionOrder<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.0.chrom() != other.0.chrom() {
            return self.0.chrom().cmp(other.0.chrom());
        }
        if self.0.right() != other.0.right() {
            return self.0.right().cmp(&other.0.right());
        }
        self.0.left().cmp(&other.0.left())
    }
}

impl<T: WithRegion> PartialOrd for RegionOrder<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: WithRegion> Deref for RegionOrder<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait WithName {
    fn name(&self) -> &str;
}
