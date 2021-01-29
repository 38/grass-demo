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
