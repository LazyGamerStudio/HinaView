use egui::{FontData, FontDefinitions, FontFamily};
use fontdb::{Database, Family, Query};
use std::sync::{Arc, OnceLock};

const CJK_FONT_CANDIDATES: [&str; 4] = [
    "Noto Sans CJK JP",
    "Noto Sans CJK KR",
    "Noto Sans JP",
    "Noto Sans KR",
];

static FIRST_FONT_CACHE: OnceLock<Option<(String, Vec<u8>)>> = OnceLock::new();

fn query_face_id(db: &Database, family_name: &str) -> Option<fontdb::ID> {
    db.query(&Query {
        families: &[Family::Name(family_name)],
        ..Query::default()
    })
}

fn load_face_data(db: &Database, id: fontdb::ID) -> Option<Vec<u8>> {
    let mut bytes: Option<Vec<u8>> = None;
    db.with_face_data(id, |font_data, _face_index| {
        bytes = Some(font_data.to_vec());
    });
    bytes
}

fn prepend_font(defs: &mut FontDefinitions, key: String, data: Vec<u8>) {
    defs.font_data
        .insert(key.clone(), Arc::new(FontData::from_owned(data)));
    defs.families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, key.clone());
    defs.families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, key);
}

fn resolve_first_available_font() -> Option<(String, Vec<u8>)> {
    let mut db = Database::new();
    db.load_system_fonts();

    for family_name in CJK_FONT_CANDIDATES {
        let Some(id) = query_face_id(&db, family_name) else {
            continue;
        };
        let Some(data) = load_face_data(&db, id) else {
            continue;
        };
        let key = format!("preferred:{}", family_name);
        tracing::debug!("[Font] Selected preferred CJK font: {}", family_name);
        return Some((key, data));
    }

    None
}

pub fn apply_preferred_cjk_fonts(ctx: &egui::Context) {
    let selected = FIRST_FONT_CACHE.get_or_init(resolve_first_available_font);
    let Some((key, data)) = selected.as_ref() else {
        tracing::debug!("[Font] No preferred CJK fonts found; using egui defaults.");
        return;
    };

    let mut defs = FontDefinitions::default();
    prepend_font(&mut defs, key.clone(), data.clone());
    ctx.set_fonts(defs);
}
