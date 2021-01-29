use super::properties::{Parsable, Serializable, WithName, WithRegion};
use std::io::{Result, Write};

#[cfg(feature = "d4-hts")]
mod bamsupport {
    use super::*;

    use d4_hts::{Alignment, AlignmentReader};
    use std::rc::Rc;

    pub use d4_hts::BamFile;

    #[derive(Clone)]
    pub struct BAMRecord<'a> {
        chrom_name: &'a str,
        record: Rc<Alignment<'a>>,
    }

    impl<'a> BAMRecord<'a> {
        pub fn iter_of(file: &BamFile) -> impl Iterator<Item = BAMRecord<'_>> {
            let chrom_list: Vec<_> = file.chroms().iter().map(|(name, _)| name).collect();
            let iter = file.to_alignment_iter();
            iter.map(|res| res.unwrap()).map(move |record| BAMRecord {
                chrom_name: chrom_list[record.ref_id() as usize],
                record: Rc::new(record),
            })
        }
    }

    impl WithRegion for BAMRecord<'_> {
        fn begin(&self) -> u32 {
            self.record.ref_begin() as u32
        }

        fn end(&self) -> u32 {
            self.record.ref_end() as u32
        }

        fn chrom(&self) -> &str {
            self.chrom_name
        }
    }
}

#[cfg(feature = "d4-hts")]
pub use bamsupport::*;

#[derive(Clone)]
pub struct Bed3<T: AsRef<str>> {
    pub begin: u32,
    pub end: u32,
    pub chrom: T,
}

impl<T: AsRef<str>> Bed3<T> {
    #[allow(dead_code)]
    pub fn to_borrowed(&self) -> Bed3<&str> {
        Bed3::new(self)
    }
}

impl<'a> Bed3<&'a str> {
    #[allow(dead_code)]
    pub fn new<T: WithRegion>(region: &'a T) -> Self {
        Self {
            begin: region.begin(),
            end: region.end(),
            chrom: region.chrom(),
        }
    }
}

impl Parsable for Bed3<String> {
    fn parse<'a, T: Iterator<Item = &'a str>>(tokens: &mut T) -> Option<Self> {
        Some(Self {
            chrom: tokens.next()?.to_string(),
            begin: tokens.next()?.parse().ok()?,
            end: tokens.next()?.parse().ok()?,
        })
    }
}

fn write_number<W: Write>(mut fp: W, mut n: u32) -> Result<()> {
    if n == 0 {
        fp.write(b"0").map(|_| ())
    } else {
        let mut buf = [0; 10];
        let mut offset = 0;
        let mut begin = 0;
        while n > 0 {
            buf[offset] = b'0' + (n % 10) as u8;
            n /= 10;
            offset += 1;
        }
        let mut end = offset - 1;
        while begin < end {
            buf.swap(begin, end);
            begin += 1;
            end -= 1;
        }

        fp.write(&buf[..offset]).map(|_| ())
    }
}

impl<T: AsRef<str>> Serializable for Bed3<T> {
    fn dump<W: Write>(&self, mut fp: W) -> Result<()> {
        fp.write_all(self.chrom().as_bytes())?;
        fp.write(b"\t")?;
        write_number(&mut fp, self.begin())?;
        fp.write(b"\t")?;
        write_number(&mut fp, self.end()).map(|_| ())
    }
}

impl<T: AsRef<str>> WithRegion for Bed3<T> {
    fn begin(&self) -> u32 {
        self.begin
    }

    fn end(&self) -> u32 {
        self.end
    }

    fn chrom(&self) -> &str {
        self.chrom.as_ref()
    }
}

#[derive(Clone)]
pub struct Bed4<T: AsRef<str>> {
    core: Bed3<T>,
    name: T,
}

impl<T: AsRef<str>> WithRegion for Bed4<T> {
    fn begin(&self) -> u32 {
        self.core.begin()
    }

    fn end(&self) -> u32 {
        self.core.end()
    }

    fn chrom(&self) -> &str {
        self.core.chrom()
    }
}

impl<T: AsRef<str>> WithName for Bed4<T> {
    fn name(&self) -> &str {
        self.name.as_ref()
    }
}

impl Parsable for Bed4<String> {
    fn parse<'a, T: Iterator<Item = &'a str>>(tokens: &mut T) -> Option<Self> {
        let core = Bed3::parse(tokens)?;
        Some(Self {
            core,
            name: tokens.next()?.to_string(),
        })
    }
}

impl<T: AsRef<str>> Serializable for Bed4<T> {
    fn dump<W: Write>(&self, mut fp: W) -> Result<()> {
        self.core.dump(&mut fp)?;
        fp.write(b"\t")?;
        fp.write_all(self.name().as_bytes())
    }
}

impl<'a> Bed4<&'a str> {
    #[allow(dead_code)]
    pub fn with_name<T: WithRegion + 'a>(region: &'a T, name: &'a str) -> Self {
        Self {
            core: Bed3::new(region),
            name,
        }
    }
}
