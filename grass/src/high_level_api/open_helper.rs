use super::CHROM_NAMES;
use crate::{
    algorithm::{AssumeSorted, AssumingSortedIter},
    chromset::LexicalChromRef,
    records::{Bed3, Bed4, Bed5},
    LineRecordStreamExt,
};
use std::path::Path;
macro_rules! define_open_helper {
        ($name:ident, $($record_type:tt)*) => {
            pub fn $name<P: AsRef<Path>>(path: P) -> AssumingSortedIter<impl Iterator<Item = $($record_type)*>> {
                CHROM_NAMES.with(|chrom_names| {
                    std::fs::File::open(path).map(|file| file.into_record_iter::<$($record_type)*, _>(chrom_names).assume_sorted())
                }).unwrap()
            }
        };
    }
define_open_helper!(open_sorted_bed3, Bed3);
define_open_helper!(open_sorted_bed4, Bed4);
define_open_helper!(open_sorted_bed5, Bed5<LexicalChromRef, f64>);
