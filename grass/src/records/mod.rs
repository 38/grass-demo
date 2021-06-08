#[cfg(feature = "hts")]
mod bam;

#[cfg(feature = "hts")]
pub use bam::{BAMRecord, BamFile};

#[cfg(feature = "hts")]
mod vcf;

#[cfg(feature = "hts")]
pub use vcf::{VcfFile, VcfRecord};

mod bed3;
pub use bed3::Bed3;

mod bed4;
pub use bed4::Bed4;

mod bed5;
pub use bed5::Bed5;
