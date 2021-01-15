use super::properties::{Parsable, Serializable, WithName, WithRegion};
use std::io::{Result, Write};

#[cfg(feature = "d4-hts")]
mod bamsupport {
    use super::*;

    use d4_hts::{Alignment, AlignmentReader, BamFile};

    pub struct BAMRecord<'a> {
        chrom_name: &'a str,
        record: Alignment<'a>,
    }

    impl<'a> BAMRecord<'a> {
        pub fn iter_of(file: &BamFile) -> impl Iterator<Item = BAMRecord<'_>> {
            let chrom_list: Vec<_> = file.chroms().iter().map(|(name, _)| name).collect();
            let iter = file.to_alignment_iter();
            iter.map(|res| res.unwrap()).map(move |record| BAMRecord {
                chrom_name: chrom_list[record.ref_id() as usize],
                record,
            })
        }
    }

    impl WithRegion for BAMRecord<'_> {
        fn left(&self) -> u32 {
            self.record.ref_begin() as u32
        }

        fn right(&self) -> u32 {
            self.record.ref_end() as u32
        }

        fn chrom(&self) -> &str {
            self.chrom_name
        }
    }
}

#[cfg(feature = "d4-hts")]
pub use bamsupport::*;

pub struct Bed3 {
    pub left: u32,
    pub right: u32,
    pub chrom: String,
}

impl Bed3 {
    #[allow(dead_code)]
    pub fn new<T: WithRegion>(region: &T) -> Self {
        Self {
            left: region.left(),
            right: region.right(),
            chrom: region.chrom().to_owned(),
        }
    }
}

impl Parsable for Bed3 {
    fn parse<'a, T: Iterator<Item = &'a str>>(tokens: &mut T) -> Option<Self> {
        Some(Self {
            chrom: tokens.next()?.to_string(),
            left: tokens.next()?.parse().ok()?,
            right: tokens.next()?.parse().ok()?,
        })
    }
}

fn write_number<W: Write>(mut fp: W, mut n: u32) -> Result<()> {
    if n == 0 {
        fp.write(b"0").map(|_| ())
    } else {
        let mut buf = [0; 10];
        let mut offset = 0;
        let mut left = 0;
        while n > 0 {
            buf[offset] = b'0' + (n % 10) as u8;
            n /= 10;
            offset += 1;
        }
        let mut right = offset - 1;
        while left < right {
            buf.swap(left, right);
            left += 1;
            right -= 1;
        }

        fp.write(&buf[..offset]).map(|_| ())
    }
}

impl Serializable for Bed3 {
    fn dump<W: Write>(&self, mut fp: W) -> Result<()> {
        fp.write_all(self.chrom().as_bytes())?;
        fp.write(b"\t")?;
        write_number(&mut fp, self.left())?;
        fp.write(b"\t")?;
        write_number(&mut fp, self.right()).map(|_| ())
    }
}

impl WithRegion for Bed3 {
    fn left(&self) -> u32 {
        self.left
    }

    fn right(&self) -> u32 {
        self.right
    }

    fn chrom(&self) -> &str {
        &self.chrom
    }
}

pub struct Bed4 {
    core: Bed3,
    name: String,
}

impl WithRegion for Bed4 {
    fn left(&self) -> u32 {
        self.core.left()
    }

    fn right(&self) -> u32 {
        self.core.right()
    }

    fn chrom(&self) -> &str {
        self.core.chrom()
    }
}

impl WithName for Bed4 {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Parsable for Bed4 {
    fn parse<'a, T: Iterator<Item = &'a str>>(tokens: &mut T) -> Option<Self> {
        let core = Bed3::parse(tokens)?;
        Some(Self {
            core,
            name: tokens.next()?.to_string(),
        })
    }
}

impl Serializable for Bed4 {
    fn dump<W: Write>(&self, mut fp: W) -> Result<()> {
        self.core.dump(&mut fp)?;
        fp.write(b"\t")?;
        fp.write_all(self.name().as_bytes())
    }
}

impl Bed4 {
    #[allow(dead_code)]
    pub fn with_name<T: WithRegion>(region: &T, name: &str) -> Self {
        Self {
            core: Bed3::new(region),
            name: name.to_owned(),
        }
    }
}
