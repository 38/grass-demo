use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Ident, LitStr};

pub(super) fn generate_bed_open_code(
    id: &Ident,
    path: &LitStr,
    size: usize,
    compressed: bool,
) -> TokenStream2 {
    // TODO : At this point we just assume everything is sorted, but this
    // is not actually the case
    let rec_type_id = Ident::new(format!("Bed{}", size).as_str(), path.span());
    if !compressed {
        quote! {
              let #id = grass::high_level_api::get_global_chrom_list().with(|gcl| {
                  use grass::LineRecordStreamExt;
                  use grass::algorithm::AssumeSorted;

                 std::fs::File::open(#path).unwrap().into_record_iter::<grass::records::#rec_type_id, _>(gcl).assume_sorted()
          });
        }
    } else {
        quote! {
            let #id = grass::high_level_api::get_global_chrom_list().with(|gcl| {
                use grass::LineRecordStreamExt;
                use grass::algorithm::AssumeSorted;

                libflate::gzip::Decoder::new(std::fs::File::open(#path).unwrap()).unwrap().into_record_iter::<grass::records::#rec_type_id, _>(gcl).assume_sorted()
            });
        }
    }
}

pub(super) fn generate_xam_open_code(id: &Ident, path: &LitStr) -> TokenStream2 {
    // TODO : At this point we just assume everything is sorted, but this
    // is not actually the case
    let bam_file_id = Ident::new(format!("{}_owned_hts_instance", id).as_str(), id.span());
    quote! {
        let #bam_file_id = grass::records::BamFile::open(#path).unwrap();
        let #id = grass::high_level_api::get_global_chrom_list().with(|gcl| {
            use grass::ChromSet;
            use grass::algorithm::AssumeSorted;
            grass::records::BAMRecord::iter_of::<grass::chromset::LexicalChromSet>(&#bam_file_id, gcl.get_handle()).assume_sorted()
        });
    }
}

pub(super) fn generate_vcf_open_code(id: &Ident, path: &LitStr) -> TokenStream2 {
    // TODO : At this point we just assume everything is sorted, but this
    // is not actually the case
    let bam_file_id = Ident::new(format!("{}_owned_hts_instance", id).as_str(), id.span());
    quote! {
        let #bam_file_id = grass::records::VcfFile::open(#path).unwrap();
        let #id = grass::high_level_api::get_global_chrom_list().with(|gcl| {
            use grass::ChromSet;
            use grass::algorithm::AssumeSorted;
            grass::records::VcfRecord::iter_of::<grass::chromset::LexicalChromSet>(&#bam_file_id, gcl.get_handle()).assume_sorted()
        });
    }
}
