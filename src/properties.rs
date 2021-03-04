use crate::ChromName;
use std::io::{Result, Write};

pub trait Parsable<'a>: Sized {
    fn parse(s: &'a str) -> Option<Self>;
}

pub trait Serializable {
    fn dump<W: Write>(&self, fp: W) -> Result<()>;
}

pub trait WithRegion<Chrom: ChromName> {
    fn begin(&self) -> u32;
    fn end(&self) -> u32;

    fn chrom(&self) -> Chrom;

    #[inline(always)]
    fn overlaps(&self, b: &impl WithRegion<Chrom>) -> bool {
        let a = self;
        if a.chrom() != b.chrom() {
            return false;
        }

        !(a.end() <= b.begin() || b.end() <= a.begin())
    }

    #[inline(always)]
    fn empty(&self) -> bool {
        self.end() <= self.begin()
    }
    #[inline(always)]
    fn length(&self) -> u32 {
        self.end().max(self.begin()) - self.begin()
    }
}

impl<'a, Chrom: ChromName, T: WithRegion<Chrom>> WithRegion<Chrom> for &'a T {
    fn begin(&self) -> u32 {
        T::begin(*self)
    }
    fn end(&self) -> u32 {
        T::end(*self)
    }
    fn chrom(&self) -> Chrom {
        T::chrom(*self)
    }
}

impl<Chrom: ChromName, A: WithRegion<Chrom>, B: WithRegion<Chrom>> WithRegion<Chrom> for (A, B) {
    #[inline(always)]
    fn begin(&self) -> u32 {
        if self.0.overlaps(&self.1) {
            self.0.begin().max(self.1.begin())
        } else {
            0
        }
    }

    #[inline(always)]
    fn end(&self) -> u32 {
        if self.0.overlaps(&self.1) {
            self.0.end().min(self.1.end())
        } else {
            0
        }
    }

    #[inline(always)]
    fn chrom(&self) -> Chrom {
        self.0.chrom()
    }
}

pub trait WithName {
    fn name(&self) -> &str;
}
