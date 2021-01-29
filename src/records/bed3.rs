use crate::properties::{Parsable, Serializable, WithRegion};
use std::io::{Write, Result};

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

