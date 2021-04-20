use crate::properties::WithRegion;

use crate::{ChromName, ChromSet, ChromSetHandle};
use std::rc::Rc;
use hts::vcf::{VcfRecord as Vcf, VcfReader};
pub use hts::vcf::VcfFile;

#[derive(Clone)]
pub struct VcfRecord<'a, C: ChromName> {
    chrom_name: C,
    record: Rc<Vcf<'a>>,
}

impl<'a, C: ChromName + 'a> VcfRecord<'a, C> {
    pub fn iter_of<S: ChromSet<RefType = C>>(
        file: &'a VcfFile,
        mut handle: S::Handle,
    ) -> impl Iterator<Item = VcfRecord<'a, C>> {
        let iter = file.vcf_iter();
        iter.map(move |record| VcfRecord {
            chrom_name: handle.query_or_insert(record.chrom_name().unwrap()),
            record: Rc::new(record),
        })
    }
}

impl<'a, C: ChromName> WithRegion<C> for VcfRecord<'a, C> {
    fn begin(&self) -> u32 {
        self.record.begin() as u32
    }

    fn end(&self) -> u32 {
        self.record.end().unwrap_or_else(||  self.record.begin()) as u32
    }

    fn chrom(&self) -> &C {
        &self.chrom_name
    }
}
