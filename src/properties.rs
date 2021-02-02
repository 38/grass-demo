use std::io::{Result, Write};

pub trait Parsable: Sized {
    fn parse<'a, T: Iterator<Item = &'a str>>(token_stream: &mut T) -> Option<Self>;
}

pub trait Serializable {
    fn dump<W: Write>(&self, fp: W) -> Result<()>;
}

pub trait WithRegion {
    fn begin(&self) -> u32;
    fn end(&self) -> u32;

    fn chrom(&self) -> &str;

    #[inline(always)]
    fn overlaps(&self, b: &impl WithRegion) -> bool {
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

impl<'a, T: WithRegion> WithRegion for &'a T {
    fn begin(&self) -> u32 {
        T::begin(*self)
    }
    fn end(&self) -> u32 {
        T::end(*self)
    }
    fn chrom(&self) -> &str {
        T::chrom(*self)
    }
}

impl<T: WithRegion> WithRegion for Option<T> {
    fn begin(&self) -> u32 {
        self.as_ref().map_or(0, |x| x.begin())
    }
    fn end(&self) -> u32 {
        self.as_ref().map_or(0, |x| x.end())
    }
    fn chrom(&self) -> &str {
        self.as_ref().map_or(".", |x| x.chrom())
    }
}

impl<A: WithRegion, B: WithRegion> WithRegion for (A, B) {
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
    fn chrom(&self) -> &str {
        self.0.chrom()
    }
}

pub trait WithName {
    fn name(&self) -> &str;
}
