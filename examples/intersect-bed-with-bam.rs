#[cfg(feature = "d4-hts")]
fn main() -> std::io::Result<()> {
    use std::env::args;
    use std::fs::File;
    use std::io::{BufRead, BufReader, BufWriter, Result, Write};

    use gql::algorithm::Intersect;
    use gql::properties::{Parsable, Serializable};
    use gql::records::Bed3;

    fn parse_file<T: Parsable>(path: &str) -> Result<impl Iterator<Item = Option<T>>> {
        let file = BufReader::new(File::open(path)?);
        Ok(file.lines().map(|line| {
            if let Ok(line) = line {
                return T::parse(&mut line.split('\t'));
            }
            None
        }))
    }

    let args: Vec<_> = args().skip(1).take(3).collect();

    let bed3_file = parse_file::<Bed3<String>>(&args[0])?.map(|x| x.unwrap());

    let bam_file = gql::records::BamFile::open(&args[1]).unwrap();
    let bam_rec_iter = gql::records::BAMRecord::iter_of(&bam_file);

    let mut out_file = BufWriter::new(File::create(&args[2])?);

    for pair in bed3_file.intersect(bam_rec_iter) {
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
