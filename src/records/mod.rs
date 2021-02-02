#[cfg(feature = "d4-hts")]
mod bam;

#[cfg(feature = "d4-hts")]
pub use bam::{BAMRecord, BamFile};

mod bed3;
pub use bed3::Bed3;

mod bed4;
pub use bed4::Bed4;
