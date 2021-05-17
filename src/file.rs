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

macro_rules! impl_line_record_stream {
    ($rec_ty:ident) => {
        impl<C: ChromSet, R: Read> Iterator for LineRecordStream<C, R, $rec_ty<C::RefType>> {
            type Item = $rec_ty<C::RefType>;
            fn next(&mut self) -> Option<Self::Item> {
                self.buffer.clear();
                self.reader.read_line(&mut self.buffer).ok()?;
                let (parsed, _) = $rec_ty::parse(self.buffer.as_ref())?;
                Some(parsed.with_chrom_set(&mut self.chrom_set_handle))
            }
        }
    };
}

impl_line_record_stream!(Bed3);
impl_line_record_stream!(Bed4);
impl_line_record_stream!(Bed5);
