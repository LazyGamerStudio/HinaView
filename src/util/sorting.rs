use std::cmp::Ordering;
use std::path::Path;

/// Natural sort comparison (Case-Insensitive) using natord library.
/// This ensures consistent ordering across the application, especially for
/// determining page indices and resume positions.
pub fn natural_cmp_ci(a: &Path, b: &Path) -> Ordering {
    let a_name = a.file_name().and_then(|n| n.to_str()).unwrap_or_default();
    let b_name = b.file_name().and_then(|n| n.to_str()).unwrap_or_default();

    natord::compare_ignore_case(a_name, b_name)
}

#[cfg(test)]
mod tests {
    use super::natural_cmp_ci;
    use std::cmp::Ordering;
    use std::path::Path;

    #[test]
    fn natural_sort_orders_numbers() {
        let a = Path::new("archive_2.cbz");
        let b = Path::new("archive_10.cbz");
        assert_eq!(natural_cmp_ci(a, b), Ordering::Less);
    }

    #[test]
    fn natural_sort_is_case_insensitive() {
        // natord::compare_ignore_case might return Equal or specific ordering
        // depending on original case, but for our purpose we check if it handles
        // the numeric/natural part correctly.
        let c = Path::new("a2.cbz");
        let d = Path::new("A10.cbz");
        assert_eq!(natural_cmp_ci(c, d), Ordering::Less);
    }
}
