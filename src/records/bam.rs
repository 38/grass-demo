use crate::properties::WithRegion;

use crate::{ChromName, ChromSet, ChromSetHandle};
use d4_hts::{Alignment, AlignmentReader};
use std::rc::Rc;

pub use d4_hts::BamFile;

#[derive(Clone)]
pub struct BAMRecord<'a, C: ChromName> {
    chrom_name: C,
    record: Rc<Alignment<'a>>,
}

impl<'a, C: ChromName + 'a> BAMRecord<'a, C> {
    pub fn iter_of<S: ChromSet<RefType = C>>(
        file: &'a BamFile,
        mut handle: S::Handle,
    ) -> impl Iterator<Item = BAMRecord<'a, C>> + 'a {
        let chrom_list: Vec<C> = file
            .chroms()
            .iter()
            .map(|(name, _)| handle.query_or_insert(name))
            .collect();
        let iter = file.to_alignment_iter();
        iter.map(|res| res.unwrap()).map(move |record| BAMRecord {
            chrom_name: chrom_list[record.ref_id() as usize].clone(),
            record: Rc::new(record),
        })
    }
}

impl<'a, C: ChromName> WithRegion<C> for BAMRecord<'a, C> {
    fn begin(&self) -> u32 {
        self.record.ref_begin() as u32
    }

    fn end(&self) -> u32 {
        self.record.ref_end() as u32
    }

    fn chrom(&self) -> &C {
        &self.chrom_name
    }
}
