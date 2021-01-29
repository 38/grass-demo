use std::io::{Result, Write};

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

    #[inline(always)]
    fn overlaps(&self, b: &impl WithRegion) -> bool {
        let a = self;
        if a.chrom() != b.chrom() {
            return false;
        }

        !(a.right() <= b.left() || b.right() <= a.left())
    }

    #[inline(always)]
    fn empty(&self) -> bool {
        self.right() <= self.left()
    }
    #[inline(always)]
    fn length(&self) -> u32 {
        self.right().max(self.left()) - self.left()
    }
}

impl<'a, 'b, A: WithRegion, B: WithRegion> WithRegion for (A, B) {
    #[inline(always)]
    fn left(&self) -> u32 {
        if self.0.overlaps(&self.1) {
            self.0.left().max(self.1.left())
        } else {
            0
        }
    }

    #[inline(always)]
    fn right(&self) -> u32 {
        if self.0.overlaps(&self.1) {
            self.0.right().min(self.1.right())
        } else {
            0
        }
    }

    #[inline(always)]
    fn chrom(&self) -> &str {
        self.0.chrom()
    }
}
pub trait WithName {
    fn name(&self) -> &str;
}
