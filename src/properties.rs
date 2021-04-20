use crate::ChromName;
use num::Num;
use std::io::{Result, Write};

pub trait Parsable<'a>: Sized {
    fn parse(s: &'a str) -> Option<(Self, usize)>;
}

pub trait Serializable {
    fn dump<W: Write>(&self, fp: W) -> Result<()>;
}

pub trait WithRegion<Chrom: ChromName> {
    fn begin(&self) -> u32;
    fn end(&self) -> u32;

    fn chrom(&self) -> &Chrom;

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
    fn chrom(&self) -> &Chrom {
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
    fn chrom(&self) -> &Chrom {
        self.0.chrom()
    }
}

pub trait WithName {
    fn name(&self) -> &str;
}

pub trait WithScore<T: Num> {
    fn score(&self) -> Option<T> {
        None
    }
}

pub enum Strand {
    Neg,
    Pos,
}

pub trait WithStrand {
    fn strand(&self) -> Option<Strand> {
        None
    }
}

impl<A: Serializable, B: Serializable> Serializable for (A, B) {
    fn dump<W: Write>(&self, mut fp: W) -> Result<()> {
        self.0.dump(&mut fp)?;
        write!(fp, "\t|\t")?;
        self.1.dump(&mut fp)
    }
}

pub enum Nuclide {
    A,
    T,
    C,
    G,
    U,
    N,
}

pub trait WithSequence {
    type RangeType: IntoIterator<Item = Nuclide>;
    fn at(&self, offset: usize) -> Nuclide;
    fn range(&self, from: usize, to: usize) -> Self::RangeType;
}
