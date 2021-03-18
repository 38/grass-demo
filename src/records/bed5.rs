use super::{Bed3, Bed4};
use num::{Num, ToPrimitive};
use std::io::{Result, Write};

use crate::{
    chromset::LexicalChromRef,
    properties::{Parsable, Serializable, WithName, WithRegion, WithScore, WithStrand},
    ChromName, ChromSetHandle, WithChromSet,
};

#[derive(Clone)]
pub struct Bed5<T: ChromName = LexicalChromRef, N: Num = f64> {
    pub core: Bed4<T>,
    pub score: Option<N>,
}

impl<T: ChromName, H: ChromSetHandle, N: Num> WithChromSet<H> for Bed5<T, N> {
    type Result = Bed5<H::RefType, N>;
    fn with_chrom_set(self, handle: &mut H) -> Self::Result {
        let core = self.core.with_chrom_list(handle);
        let score = self.score;
        Bed5 { core, score }
    }
}

impl<'a, N: Num> Parsable<'a> for Bed5<&'a str, N> {
    fn parse(s: &'a str) -> Option<(Self, usize)> {
        let (core, rem) = Bed4::parse(s)?;
        Some((
            Self {
                core,
                score: N::from_str_radix(&s[rem..], 10).ok(),
            },
            s.len(),
        ))
    }
}

impl<T: ChromName, N: Num> WithRegion<T> for Bed5<T, N> {
    fn begin(&self) -> u32 {
        self.core.begin()
    }

    fn end(&self) -> u32 {
        self.core.end()
    }

    fn chrom(&self) -> &T {
        self.core.chrom()
    }
}

impl<T: ChromName, N: Num> WithName for Bed5<T, N> {
    fn name(&self) -> &str {
        self.core.name()
    }
}

// TODO:gerenalize this implementation
impl<T: ChromName, N: ToPrimitive + Num> Serializable for Bed5<T, N> {
    fn dump<W: Write>(&self, mut fp: W) -> Result<()> {
        self.core.dump(&mut fp)?;
        fp.write(b"\t")?;
        if let Some(ref score) = self.score {
            crate::ioutils::write_number(fp, score.to_i32().unwrap_or(-1))
        } else {
            fp.write_all(b".")
        }
    }
}

impl<T: ChromName, N: Num> Bed5<T, N> {
    pub fn with_chrom_list<H: ChromSetHandle>(self, chrom_list: &mut H) -> Bed5<H::RefType, N> {
        Bed5 {
            core: self.core.with_chrom_list(chrom_list),
            score: self.score,
        }
    }

    pub fn new(chrom: T, begin: u32, end: u32, name: &str, score: N) -> Self {
        Self {
            core: Bed4 {
                name: std::rc::Rc::new(name.to_string()),
                core: Bed3 { begin, end, chrom },
            },
            score: Some(score),
        }
    }
}

impl<T: ChromName, N: Num> WithScore<i32> for Bed5<T, N> {}
impl<T: ChromName, N: Num> WithStrand for Bed5<T, N> {}
