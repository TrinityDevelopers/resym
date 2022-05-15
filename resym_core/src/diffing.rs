use similar::{ChangeTag, TextDiff};

use std::fmt::Write;

use crate::{pdb_file::PdbFile, PKG_NAME, PKG_VERSION};

pub fn diff_type_by_name(
    pdb_file_from: &PdbFile,
    pdb_file_to: &PdbFile,
    type_name: &str,
    print_header: bool,
    reconstruct_dependencies: bool,
    print_access_specifiers: bool,
    show_line_numbers: bool,
) -> String {
    let diff_start = std::time::Instant::now();
    let reconstructed_type_from = pdb_file_from
        .reconstruct_type_by_name(type_name, reconstruct_dependencies, print_access_specifiers)
        .unwrap_or_default();
    let reconstructed_type_to = pdb_file_to
        .reconstruct_type_by_name(type_name, reconstruct_dependencies, print_access_specifiers)
        .unwrap_or_default();
    if reconstructed_type_from.is_empty() && reconstructed_type_to.is_empty() {
        // Make it obvious an error occured
        return "Error: type not found".to_string();
    }

    // Diff reconstructed reprensentations
    let mut output = String::default();
    if print_header {
        // FIXME: Handle error properly
        let _r = write!(
            &mut output,
            "{}",
            generate_diff_header(pdb_file_from, pdb_file_to)
        );
    }

    let reconstructed_type_diff =
        TextDiff::from_lines(&reconstructed_type_from, &reconstructed_type_to);

    let line_count = reconstructed_type_diff.iter_all_changes().count();
    let line_number_max_width = int_log10(line_count);
    let empty_padding = " ".repeat(line_number_max_width);
    for change in reconstructed_type_diff.iter_all_changes() {
        let line_numbers = match change.tag() {
            ChangeTag::Delete => format!(
                "{:>width$} {} |",
                change.old_index().unwrap_or_default() + 1,
                empty_padding,
                width = line_number_max_width
            ),
            ChangeTag::Insert => format!(
                "{} {:>width$} |",
                empty_padding,
                change.new_index().unwrap_or_default() + 1,
                width = line_number_max_width
            ),
            ChangeTag::Equal => format!(
                "{:>width$} {:>width$} |",
                change.old_index().unwrap_or_default() + 1,
                change.new_index().unwrap_or_default() + 1,
                width = line_number_max_width
            ),
        };
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        // FIXME: Handle error properly
        let _r = write!(
            &mut output,
            "{}{}{}",
            if show_line_numbers {
                line_numbers
            } else {
                String::default()
            },
            sign,
            change
        );
    }
    log::debug!("Type diffing took {} ms", diff_start.elapsed().as_millis());

    output
}

fn generate_diff_header(pdb_file_from: &PdbFile, pdb_file_to: &PdbFile) -> String {
    format!(
        concat!(
            "//\n",
            "// Showing differences between two PDB files:\n",
            "//\n",
            "// Reference PDB file: {}\n",
            "// Image architecture: {}\n",
            "//\n",
            "// New PDB file: {}\n",
            "// Image architecture: {}\n",
            "//\n",
            "// Information extracted with {} v{}\n",
            "//\n\n"
        ),
        pdb_file_from.file_path.display(),
        pdb_file_from.machine_type,
        pdb_file_to.file_path.display(),
        pdb_file_to.machine_type,
        PKG_NAME,
        PKG_VERSION,
    )
}

// FIXME: Replace with `checked_log10` once it's stabilized.
fn int_log10<T>(mut i: T) -> usize
where
    T: std::ops::DivAssign + std::cmp::PartialOrd + From<u8> + Copy,
{
    let zero = T::from(0);
    if i == zero {
        return 1;
    }

    let mut len = 0;
    let ten = T::from(10);

    while i > zero {
        i /= ten;
        len += 1;
    }

    len
}