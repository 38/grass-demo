use crate::properties::{WithRegion, WithSequence};

use crate::{ChromName, ChromSet, ChromSetHandle};
use hts::alignment::{Alignment, AlignmentFile, AlignmentReader};
use std::rc::Rc;

pub type BamFile = AlignmentFile;

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
        let mut chrom_list: Vec<C> = vec![];

        for idx in 0.. {
            if let Ok(chrom_name) = file.get_chrom_name_by_id(idx) {
                chrom_list.push(handle.query_or_insert(chrom_name));
            } else {
                break;
            }
        }

        let iter = file.alignment_iter();
        iter.map(move |record| BAMRecord {
            chrom_name: chrom_list[record.chrom_id()].clone(),
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

