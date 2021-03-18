use crate::{
    properties::Parsable,
    records::{Bed3, Bed4, Bed5},
    ChromSet, WithChromSet,
};

use std::{
    io::{BufRead, BufReader, Read},
    marker::PhantomData,
};

pub struct LineRecordStream<C: ChromSet, R: Read, Rec> {
    chrom_set_handle: C::Handle,
    reader: BufReader<R>,
    buffer: String,
    _p: PhantomData<Rec>,
}

impl<C: ChromSet, R: Read, Rec> LineRecordStream<C, R, Rec> {
    pub(crate) fn with_chrom_set(chrom_set: &C, reader: R) -> Self {
        let chrom_set_handle = chrom_set.get_handle();
        let reader = BufReader::new(reader);
        Self {
            chrom_set_handle,
            reader,
            buffer: String::with_capacity(4096),
            _p: PhantomData,
        }
    }
}

pub trait LineRecordStreamExt: Read {
    fn into_record_iter<Record, S: ChromSet>(
        self,
        chrom_set: &S,
    ) -> LineRecordStream<S, Self, Record>
    where
        Self: Sized,
    {
        LineRecordStream::with_chrom_set(chrom_set, self)
    }
}

impl<R: Read> LineRecordStreamExt for R {}

impl<C: ChromSet, R: Read> Iterator for LineRecordStream<C, R, Bed3<C::RefType>> {
    type Item = Bed3<C::RefType>;
    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.clear();
        self.reader.read_line(&mut self.buffer).ok()?;
        let (parsed, _) = Bed3::parse(self.buffer.as_ref())?;
        Some(parsed.with_chrom_set(&mut self.chrom_set_handle))
    }
}

impl<C: ChromSet, R: Read> Iterator for LineRecordStream<C, R, Bed4<C::RefType>> {
    type Item = Bed4<C::RefType>;
    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.clear();
        self.reader.read_line(&mut self.buffer).ok()?;
        let (parsed, _) = Bed4::parse(self.buffer.as_ref())?;
        Some(parsed.with_chrom_set(&mut self.chrom_set_handle))
    }
}

impl<C: ChromSet, R: Read> Iterator for LineRecordStream<C, R, Bed5<C::RefType>> {
    type Item = Bed5<C::RefType>;
    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.clear();
        self.reader.read_line(&mut self.buffer).ok()?;
        let (parsed, _) = Bed5::parse(self.buffer.as_ref())?;
        Some(parsed.with_chrom_set(&mut self.chrom_set_handle))
    }
}
