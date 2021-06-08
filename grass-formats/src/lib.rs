use std::{
    fs::File,
    io::{BufRead, BufReader, Cursor, Error, ErrorKind, Read, Result, Write},
    path::Path,
};

use libflate::gzip::Decoder;

#[derive(Debug, PartialEq)]
pub enum AlignmentFlavor {
    BAM,
    CRAM,
    SAM,
}

#[derive(Debug, PartialEq)]
pub enum FileKind {
    Bed(usize),
    Vcf,
    Fasta,
    Alignment(AlignmentFlavor),
}

#[derive(Debug, PartialEq)]
pub struct FileFormat {
    pub deflated: bool,
    pub kind: FileKind,
}

fn detect_gzip<R: Read>(mut input: R) -> Result<std::result::Result<impl Read, impl Read>> {
    let mut magic_code = [0u8; 2];
    input.read_exact(&mut magic_code)?;
    let rewind = Cursor::new(magic_code).chain(input);
    if magic_code == [0x1f, 0x8b] {
        Ok(Ok(Decoder::new(rewind)?))
    } else {
        Ok(Err(rewind))
    }
}

fn detect_bam_or_cram<R: Read>(mut input: R) -> Result<(Option<AlignmentFlavor>, impl Read)> {
    let mut head = [0u8; 4];
    input.read_exact(&mut head)?;
    let rewind = Cursor::new(head).chain(input);
    match &head {
        b"BAM\x01" => Ok((Some(AlignmentFlavor::BAM), rewind)),
        b"CRAM" => Ok((Some(AlignmentFlavor::CRAM), rewind)),
        _ => Ok((None, rewind)),
    }
}

fn detect_uncompressed_text_file_kind<R: Read>(input: R) -> Result<(FileKind, impl Read)> {
    let mut reader = BufReader::new(input);

    let mut seen: Vec<u8> = vec![];
    let mut line = String::new();
    let mut line_count = 0;
    let mut detect_type = Err(Error::new(ErrorKind::Other, "Unsupported file format"));
    loop {
        line.clear();
        reader.read_line(&mut line)?;
        seen.write(line.as_bytes())?;

        if line_count == 0 {
            if line.starts_with("##fileformat=VCF") {
                detect_type = Ok(FileKind::Vcf);
                break;
            } else if line.starts_with("#") {
                detect_type = Ok(FileKind::Bed(0));
            } else {
                match &line[..1] {
                    ";" | ">" => detect_type = Ok(FileKind::Fasta),
                    "@" => detect_type = Ok(FileKind::Alignment(AlignmentFlavor::SAM)),
                    "#" => detect_type = Ok(FileKind::Bed(0)),
                    _ => {
                        detect_type = Ok(FileKind::Bed(
                            line.trim_end().chars().filter(|&c| c == '\t').count() + 1,
                        ));
                        break;
                    }
                }
            }
        } else {
            match detect_type {
                Ok(FileKind::Bed(0)) => {
                    if !line.starts_with("#") {
                        detect_type = Ok(FileKind::Bed(
                            line.trim_end().chars().filter(|&c| c == '\t').count() + 1,
                        ));
                        break;
                    }
                }
                _ => break,
            }
        }

        line_count += 1;
    }

    Ok((detect_type?, Cursor::new(seen).chain(reader)))
}

impl FileFormat {
    pub fn detect_file<P: AsRef<Path>>(p: P) -> Result<FileFormat> {
        let input = File::open(p.as_ref())?;

        match detect_gzip(input)? {
            Ok(stream) => match detect_bam_or_cram(stream)? {
                (Some(flavor), _) => Ok(FileFormat {
                    kind: FileKind::Alignment(flavor),
                    deflated: false,
                }),
                (_, stream) => Ok(FileFormat {
                    kind: detect_uncompressed_text_file_kind(stream)?.0,
                    deflated: true,
                }),
            },
            Err(stream) => match detect_bam_or_cram(stream)? {
                (Some(flavor), _) => Ok(FileFormat {
                    kind: FileKind::Alignment(flavor),
                    deflated: false,
                }),
                (_, stream) => Ok(FileFormat {
                    kind: detect_uncompressed_text_file_kind(stream)?.0,
                    deflated: false,
                }),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_format_detect() -> std::result::Result<(), Box<dyn std::error::Error>> {
        use AlignmentFlavor::*;
        use FileKind::*;
        assert_eq!(
            FileFormat::detect_file("range.bam")?,
            FileFormat {
                deflated: false,
                kind: Alignment(BAM)
            }
        );
        assert_eq!(
            FileFormat::detect_file("range.cram")?,
            FileFormat {
                deflated: false,
                kind: Alignment(CRAM)
            }
        );
        assert_eq!(
            FileFormat::detect_file("test.bed.gz")?,
            FileFormat {
                deflated: true,
                kind: Bed(3)
            }
        );
        assert_eq!(
            FileFormat::detect_file("test.bed")?,
            FileFormat {
                deflated: false,
                kind: Bed(4)
            }
        );
        assert_eq!(
            FileFormat::detect_file("test.vcf.gz")?,
            FileFormat {
                deflated: true,
                kind: Vcf
            }
        );
        Ok(())
    }
}
