use std::env::args;
use std::fs::File;
use std::io::{BufWriter, Result, Write};

use grass::algorithm::SortedIntersect;
use grass::properties::Serializable;
use grass::records::Bed3;
use grass::LexicalChromSet;
use grass::LineRecordStreamExt;
use grass::{algorithm::AssumeSorted, chromset::LexicalChromRef};

fn main() -> Result<()> {
    let args: Vec<_> = args().skip(1).take(4).collect();

    let chroms = LexicalChromSet::new();

    let file1 = File::open(&args[0])?
        .into_record_iter::<Bed3<LexicalChromRef>, _>(&chroms)
        .assume_sorted();
    let file2 = File::open(&args[1])?
        .into_record_iter::<Bed3<LexicalChromRef>, _>(&chroms)
        .assume_sorted();
    let file3 = File::open(&args[2])?
        .into_record_iter::<Bed3<LexicalChromRef>, _>(&chroms)
        .assume_sorted();

    let mut out_file = BufWriter::new(File::create(&args[3])?);

    for triple in file1.sorted_intersect(file2).sorted_intersect(file3) {
        let result = Bed3::new(&triple);
        result.dump(&mut out_file)?;
        out_file.write_all(b"\n")?;
    }

    Ok(())
}
