// src/document/spread_builder.rs
use super::logical_spread::LogicalSpread;
use super::page_meta::PageMeta;
use crate::types::LayoutMode;

/// Constructs logical spreads based on the specified layout mode and page metadata.
pub fn build_spreads(pages: &[PageMeta], mode: LayoutMode) -> Vec<LogicalSpread> {
    match mode {
        LayoutMode::Single => build_single_spreads(pages),
        LayoutMode::Dual {
            rtl,
            first_page_offset,
        } => build_dual_spreads(pages, rtl, first_page_offset),
        LayoutMode::VerticalScroll => build_single_spreads(pages), // Vertical scroll treats each page as a single unit
    }
}

fn build_single_spreads(pages: &[PageMeta]) -> Vec<LogicalSpread> {
    pages
        .iter()
        .map(|p| LogicalSpread {
            left: Some(p.index),
            right: None,
        })
        .collect()
}

fn build_dual_spreads(
    pages: &[PageMeta],
    rtl: bool,
    first_page_offset: bool,
) -> Vec<LogicalSpread> {
    let mut spreads = Vec::new();
    let mut i = 0;

    // Handle the first page offset (e.g., cover page).
    // Wide/animated first page must remain a standalone spread, not offset slot.
    if first_page_offset && !pages.is_empty() {
        let first = &pages[0];
        if first.is_wide || first.is_animated {
            spreads.push(create_spread(Some(first.index), None, rtl));
        } else {
            spreads.push(create_spread(None, Some(first.index), rtl));
        }
        i = 1;
    }

    while i < pages.len() {
        let p1 = &pages[i];

        // Wide pages or animated pages are always treated as a single spread
        if p1.is_wide || p1.is_animated {
            spreads.push(create_spread(Some(p1.index), None, rtl));
            i += 1;
            continue;
        }

        if i + 1 < pages.len() {
            let p2 = &pages[i + 1];
            if p2.is_wide || p2.is_animated {
                // If the next page is wide or animated, current page must be a single spread
                spreads.push(create_spread(Some(p1.index), None, rtl));
                i += 1;
            } else {
                // Pair two normal pages
                spreads.push(create_spread(Some(p1.index), Some(p2.index), rtl));
                i += 2;
            }
        } else {
            // Last remaining page
            spreads.push(create_spread(Some(p1.index), None, rtl));
            i += 1;
        }
    }

    spreads
}

fn create_spread(idx1: Option<usize>, idx2: Option<usize>, rtl: bool) -> LogicalSpread {
    if rtl {
        // In RTL mode, the first index (logical start) is placed on the right
        LogicalSpread {
            left: idx2,
            right: idx1,
        }
    } else {
        // In LTR mode, the first index is placed on the left
        LogicalSpread {
            left: idx1,
            right: idx2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(index: usize) -> PageMeta {
        PageMeta {
            index,
            name: format!("page_{}", index),
            format_label: "Unknown".to_string(),
            file_size_bytes: None,

            width: 1000,
            height: 1500,
            metadata_probe_failed: false,
            is_wide: false,
            is_animated: false,
            icc_profile: None,
            exif_camera: None,
            exif_lens: None,
            exif_f_stop: None,
            exif_shutter: None,
            exif_iso: None,
            exif_datetime: None,
        }
    }

    #[test]
    fn dual_ltr_pairs_pages_normally() {
        let pages = vec![page(1), page(2), page(3), page(4)];
        let spreads = build_spreads(
            &pages,
            LayoutMode::Dual {
                rtl: false,
                first_page_offset: false,
            },
        );

        assert_eq!(spreads.len(), 2);
        assert_eq!(spreads[0].left, Some(1));
        assert_eq!(spreads[0].right, Some(2));
        assert_eq!(spreads[1].left, Some(3));
        assert_eq!(spreads[1].right, Some(4));
    }

    #[test]
    fn dual_ltr_with_offset_places_first_page_on_right_slot() {
        let pages = vec![page(1), page(2), page(3), page(4)];
        let spreads = build_spreads(
            &pages,
            LayoutMode::Dual {
                rtl: false,
                first_page_offset: true,
            },
        );

        assert_eq!(spreads[0].left, None);
        assert_eq!(spreads[0].right, Some(1));
        assert_eq!(spreads[1].left, Some(2));
        assert_eq!(spreads[1].right, Some(3));
    }

    #[test]
    fn wide_or_animated_pages_are_forced_to_single_spread() {
        let mut p5 = page(5);
        p5.is_wide = true;
        let mut p8 = page(8);
        p8.is_animated = true;
        let pages = vec![page(1), page(2), page(3), page(4), p5, page(6), page(7), p8];

        let spreads = build_spreads(
            &pages,
            LayoutMode::Dual {
                rtl: false,
                first_page_offset: false,
            },
        );

        assert_eq!(spreads[0].left, Some(1));
        assert_eq!(spreads[0].right, Some(2));
        assert_eq!(spreads[1].left, Some(3));
        assert_eq!(spreads[1].right, Some(4));
        assert_eq!(spreads[2].left, Some(5));
        assert_eq!(spreads[2].right, None);
        assert_eq!(spreads[3].left, Some(6));
        assert_eq!(spreads[3].right, Some(7));
        assert_eq!(spreads[4].left, Some(8));
        assert_eq!(spreads[4].right, None);
    }
}
