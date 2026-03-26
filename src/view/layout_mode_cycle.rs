use crate::types::LayoutMode;

pub fn cycle_layout_mode(current: LayoutMode, first_page_offset: bool) -> LayoutMode {
    match current {
        LayoutMode::Single => LayoutMode::Dual {
            rtl: false,
            first_page_offset,
        },
        LayoutMode::Dual { rtl: false, .. } => LayoutMode::Dual {
            rtl: true,
            first_page_offset,
        },
        LayoutMode::Dual { rtl: true, .. } => LayoutMode::VerticalScroll,
        LayoutMode::VerticalScroll => LayoutMode::Single,
    }
}
