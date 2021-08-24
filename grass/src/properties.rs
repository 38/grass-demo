use crate::{records::Bed3, ChromName};
use num::Num;
use std::io::{Result, Write};

pub trait Parsable<'a>: Sized {
    fn parse(s: &'a str) -> Option<(Self, usize)>;
}

pub trait Serializable {
    fn dump<W: Write>(&self, fp: W) -> Result<()>;
}

pub trait WithRegionCore<Chrom: ChromName> {
    fn begin(&self) -> u32;
    fn end(&self) -> u32;

    fn chrom(&self) -> &Chrom;

    #[inline(always)]
    fn empty(&self) -> bool {
        self.end() <= self.begin()
    }
    #[inline(always)]
    fn length(&self) -> u32 {
        self.end().max(self.begin()) - self.begin()
    }

    #[inline(always)]
    fn to_bed3(&self) -> Bed3<Chrom> {
        Bed3 {
            chrom: self.chrom().clone(),
            begin: self.begin(),
            end: self.end(),
        }
    }
}

pub trait WithRegion<Chrom: ChromName>: WithRegionCore<Chrom> {
    #[inline(always)]
    fn overlaps(&self, b: &impl WithRegion<Chrom>) -> bool {
        let a = self;
        if a.chrom() != b.chrom() {
            return false;
        }

        !(a.end() <= b.begin() || b.end() <= a.begin())
    }
}

impl<C: ChromName, T: WithRegionCore<C>> WithRegion<C> for T {}

impl<'a, Chrom: ChromName, T: WithRegion<Chrom>> WithRegionCore<Chrom> for &'a T {
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
impl<Chrom: ChromName, A: WithRegion<Chrom>, B: WithRegion<Chrom>> WithRegionCore<Chrom>
    for (A, B)
{
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

pub trait Intersection<Chrom: ChromName>: WithRegionCore<Chrom> {
    fn original(&self, idx: usize) -> &dyn WithRegionCore<Chrom>;
    fn size(&self) -> usize;
}

macro_rules! impl_intersection_trait {
    ($($t_name: ident),* => $($idx: tt),*) => {
        impl <Chrom: ChromName, $($t_name: WithRegion<Chrom>),*> Intersection<Chrom> for ($($t_name),*) {
            fn original(&self, idx: usize) -> &dyn WithRegionCore<Chrom> {
                match idx {
                    $($idx => &self.$idx,)*
                    _ => panic!("Index out of range")
                }
            }
            fn size(&self) -> usize {
                $(let _ret = $idx;)*
                _ret + 1
            }
        }
    }
}

impl_intersection_trait!(A, B => 0, 1);

macro_rules! impl_with_region_for_tuple {
    (($($t_var:ident),*), ($($head:tt),*), $tail:tt) => {
       impl <Chrom: ChromName, $($t_var: WithRegion<Chrom>),*> WithRegionCore<Chrom> for ($($t_var),*) {
           #[inline(always)]
           fn begin(&self) -> u32 {
               if ($(&self . $head,)*).overlaps(&self.$tail) {
                   ($(&self . $head,)*).begin().max(self.$tail.begin())
               } else {
                   0
               }
           }
           #[inline(always)]
           fn end(&self) -> u32 {
               if ($(&self . $head,)*).overlaps(&self.$tail) {
                   ($(&self . $head,)*).end().min(self.$tail.end())
               } else {
                   0
               }
           }
           #[inline(always)]
           fn chrom(&self) -> &Chrom {
               self.0.chrom()
           }
       }
       impl_intersection_trait!($($t_var),* => $($head,)* $tail);
    };
}

impl_with_region_for_tuple!((A, B, C), (0, 1), 2);
impl_with_region_for_tuple!((A, B, C, D), (0, 1, 2), 3);
impl_with_region_for_tuple!((A, B, C, D, E), (0, 1, 2, 3), 4);
impl_with_region_for_tuple!((A, B, C, D, E, F), (0, 1, 2, 3, 4), 5);
impl_with_region_for_tuple!((A, B, C, D, E, F, G), (0, 1, 2, 3, 4, 5), 6);
impl_with_region_for_tuple!((A, B, C, D, E, F, G, H), (0, 1, 2, 3, 4, 5, 6), 7);

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
