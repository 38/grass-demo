#[cfg(feature = "d4-hts")]
fn main() -> std::io::Result<()> {
    use std::env::args;
    use std::fs::File;
    use std::io::{BufWriter, Write};

    use gql::algorithm::{AssumeSorted, SortedIntersect};
    use gql::chromset::{ChromSet, LexicalChromRef, LexicalChromSet};
    use gql::properties::Serializable;
    use gql::records::Bed3;
    use gql::LineRecordStreamExt;
    let args: Vec<_> = args().skip(1).take(3).collect();

    let chromset = LexicalChromSet::new();

    let bed3_file = File::open(&args[0])?
        .into_record_iter::<Bed3<LexicalChromRef>, _>(&chromset)
        .assume_sorted();

    let bam_file = gql::records::BamFile::open(&args[1]).unwrap();
    let bam_rec_iter =
        gql::records::BAMRecord::iter_of::<LexicalChromSet>(&bam_file, chromset.get_handle())
            .assume_sorted();

    let mut out_file = BufWriter::new(File::create(&args[2])?);

    for pair in bed3_file.sorted_intersect(bam_rec_iter) {
        let result = Bed3::new(&pair);
        result.dump(&mut out_file)?;
        out_file.write_all(b"\n")?;
    }

    Ok(())
}

#[cfg(not(feature = "d4-hts"))]
fn main() -> ! {
    panic!("Please enable d4-hts feature to use HTSLIB support");
}
