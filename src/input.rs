// src/input.rs
pub use crate::app::AppCommand;
use winit::keyboard::{Key, NamedKey};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RuntimeCommand {
    ToggleFullscreen,
    ToggleUiWindows,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputCommand {
    App(AppCommand),
    Runtime(RuntimeCommand),
}

impl InputCommand {
    pub fn to_localized_key(&self) -> &'static str {
        match self {
            InputCommand::App(AppCommand::NavigatePrevious) => "navigate_previous",
            InputCommand::App(AppCommand::NavigateNext) => "navigate_next",
            InputCommand::App(AppCommand::NavigateFirst) => "navigate_first",
            InputCommand::App(AppCommand::NavigateLast) => "navigate_last",
            InputCommand::App(AppCommand::NavigatePreviousArchive) => "navigate_previous_archive",
            InputCommand::App(AppCommand::NavigateNextArchive) => "navigate_next_archive",
            InputCommand::App(AppCommand::SetFitScreen) => "set_fit_screen",
            InputCommand::App(AppCommand::SetFitWidth) => "set_fit_width",
            InputCommand::App(AppCommand::SetFitHeight) => "set_fit_height",
            InputCommand::App(AppCommand::CycleLayoutMode) => "cycle_layout_mode",
            InputCommand::App(AppCommand::ToggleFirstPageOffset) => "toggle_first_page_offset",
            InputCommand::App(AppCommand::ZoomInStep) => "zoom_in_step",
            InputCommand::App(AppCommand::ZoomOutStep) => "zoom_out_step",
            InputCommand::App(AppCommand::RotateCCW) => "rotate_ccw",
            InputCommand::App(AppCommand::RotateCW) => "rotate_cw",
            InputCommand::App(AppCommand::SaveManualBookmark) => "save_manual_bookmark",
            InputCommand::App(AppCommand::ResetView) => "reset_view",
            InputCommand::App(AppCommand::OpenFile) => "open_file",
            InputCommand::App(AppCommand::AdjustOffset(0.0, 1.0)) => "adjust_offset_up",
            InputCommand::App(AppCommand::AdjustOffset(0.0, -1.0)) => "adjust_offset_down",
            InputCommand::App(AppCommand::AdjustOffset(-1.0, 0.0)) => "adjust_offset_left",
            InputCommand::App(AppCommand::AdjustOffset(1.0, 0.0)) => "adjust_offset_right",
            InputCommand::Runtime(RuntimeCommand::ToggleFullscreen) => "toggle_fullscreen",
            InputCommand::Runtime(RuntimeCommand::ToggleUiWindows) => "toggle_ui_windows",
            _ => "unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputBinding {
    Key(Key),
    MouseWheelUp,
    MouseWheelDown,
    MouseDoubleClick(winit::event::MouseButton),
}

impl InputBinding {
    pub fn to_string(&self) -> String {
        match self {
            InputBinding::Key(Key::Named(n)) => match n {
                NamedKey::ArrowLeft => "←".to_string(),
                NamedKey::ArrowRight => "→".to_string(),
                NamedKey::ArrowUp => "↑".to_string(),
                NamedKey::ArrowDown => "↓".to_string(),
                NamedKey::PageUp => "PgUp".to_string(),
                NamedKey::PageDown => "PgDn".to_string(),
                NamedKey::Space => "Space".to_string(),
                NamedKey::Home => "Home".to_string(),
                NamedKey::End => "End".to_string(),
                _ => format!("{:?}", n),
            },
            InputBinding::Key(Key::Character(s)) => s.to_uppercase(),
            InputBinding::MouseWheelUp => "WheelUp".to_string(),
            InputBinding::MouseWheelDown => "WheelDown".to_string(),
            InputBinding::MouseDoubleClick(winit::event::MouseButton::Left) => {
                "Double Click".to_string()
            }
            _ => format!("{:?}", self),
        }
    }
}

pub struct ShortcutMapping {
    pub command: InputCommand,
    pub bindings: Vec<InputBinding>,
    pub ctrl: bool,
}

lazy_static::lazy_static! {
    pub static ref SHORTCUT_MAPPINGS: Vec<ShortcutMapping> = vec![
        ShortcutMapping {
            command: InputCommand::App(AppCommand::NavigatePrevious),
            bindings: vec![
                InputBinding::Key(Key::Named(NamedKey::ArrowLeft)),
                InputBinding::Key(Key::Named(NamedKey::ArrowUp)),
                InputBinding::Key(Key::Named(NamedKey::PageUp)),
                InputBinding::MouseWheelUp,
            ],
            ctrl: false,
        },
        ShortcutMapping {
            command: InputCommand::App(AppCommand::NavigatePrevious),
            bindings: vec![InputBinding::Key(Key::Named(NamedKey::Space))],
            ctrl: true,
        },
        ShortcutMapping {
            command: InputCommand::App(AppCommand::NavigateNext),
            bindings: vec![
                InputBinding::Key(Key::Named(NamedKey::ArrowRight)),
                InputBinding::Key(Key::Named(NamedKey::ArrowDown)),
                InputBinding::Key(Key::Named(NamedKey::PageDown)),
                InputBinding::MouseWheelDown,
                InputBinding::Key(Key::Named(NamedKey::Space)),
            ],
            ctrl: false,
        },
        ShortcutMapping {
            command: InputCommand::App(AppCommand::NavigateFirst),
            bindings: vec![InputBinding::Key(Key::Named(NamedKey::Home))],
            ctrl: false,
        },
        ShortcutMapping {
            command: InputCommand::App(AppCommand::NavigateLast),
            bindings: vec![InputBinding::Key(Key::Named(NamedKey::End))],
            ctrl: false,
        },
        ShortcutMapping {
            command: InputCommand::App(AppCommand::NavigatePreviousArchive),
            bindings: vec![InputBinding::Key(Key::Character("[".into()))],
            ctrl: false,
        },
        ShortcutMapping {
            command: InputCommand::App(AppCommand::NavigateNextArchive),
            bindings: vec![InputBinding::Key(Key::Character("]".into()))],
            ctrl: false,
        },
        ShortcutMapping {
            command: InputCommand::Runtime(RuntimeCommand::ToggleFullscreen),
            bindings: vec![Key::Character("f".into()).into()],
            ctrl: false,
        },
        ShortcutMapping {
            command: InputCommand::Runtime(RuntimeCommand::ToggleUiWindows),
            bindings: vec![Key::Character("u".into()).into()],
            ctrl: false,
        },
        ShortcutMapping {
            command: InputCommand::App(AppCommand::SetFitScreen),
            bindings: vec![Key::Character("`".into()).into(), Key::Character("1".into()).into()],
            ctrl: false,
        },
        ShortcutMapping {
            command: InputCommand::App(AppCommand::SetFitWidth),
            bindings: vec![Key::Character("3".into()).into()],
            ctrl: false,
        },
        ShortcutMapping {
            command: InputCommand::App(AppCommand::SetFitHeight),
            bindings: vec![Key::Character("h".into()).into()],
            ctrl: false,
        },
        ShortcutMapping {
            command: InputCommand::App(AppCommand::CycleLayoutMode),
            bindings: vec![Key::Character("2".into()).into()],
            ctrl: false,
        },
        ShortcutMapping {
            command: InputCommand::App(AppCommand::ToggleFirstPageOffset),
            bindings: vec![Key::Character("o".into()).into()],
            ctrl: false,
        },
        ShortcutMapping {
            command: InputCommand::App(AppCommand::SaveManualBookmark),
            bindings: vec![Key::Character("b".into()).into()],
            ctrl: false,
        },
        ShortcutMapping {
            command: InputCommand::App(AppCommand::ZoomInStep),
            bindings: vec![Key::Character("+".into()).into(), Key::Character("=".into()).into()],
            ctrl: false,
        },
        ShortcutMapping {
            command: InputCommand::App(AppCommand::ZoomInStep),
            bindings: vec![InputBinding::MouseWheelUp],
            ctrl: true,
        },
        ShortcutMapping {
            command: InputCommand::App(AppCommand::ZoomOutStep),
            bindings: vec![Key::Character("-".into()).into()],
            ctrl: false,
        },
        ShortcutMapping {
            command: InputCommand::App(AppCommand::ZoomOutStep),
            bindings: vec![InputBinding::MouseWheelDown],
            ctrl: true,
        },
        ShortcutMapping {
            command: InputCommand::App(AppCommand::RotateCCW),
            bindings: vec![Key::Character("q".into()).into()],
            ctrl: false,
        },
        ShortcutMapping {
            command: InputCommand::App(AppCommand::RotateCW),
            bindings: vec![Key::Character("e".into()).into()],
            ctrl: false,
        },
        ShortcutMapping {
            command: InputCommand::App(AppCommand::ResetView),
            bindings: vec![Key::Character("r".into()).into(), InputBinding::MouseDoubleClick(winit::event::MouseButton::Left)],
            ctrl: false,
        },
        ShortcutMapping {
            command: InputCommand::App(AppCommand::AdjustOffset(0.0, 1.0)),
            bindings: vec![Key::Character("w".into()).into()],
            ctrl: false,
        },
        ShortcutMapping {
            command: InputCommand::App(AppCommand::AdjustOffset(0.0, -1.0)),
            bindings: vec![Key::Character("s".into()).into()],
            ctrl: false,
        },
        ShortcutMapping {
            command: InputCommand::App(AppCommand::AdjustOffset(-1.0, 0.0)),
            bindings: vec![Key::Character("a".into()).into()],
            ctrl: false,
        },
        ShortcutMapping {
            command: InputCommand::App(AppCommand::AdjustOffset(1.0, 0.0)),
            bindings: vec![Key::Character("d".into()).into()],
            ctrl: false,
        },
        ShortcutMapping {
            command: InputCommand::App(AppCommand::OpenFile),
            bindings: vec![Key::Character("o".into()).into()],
            ctrl: true,
        },
    ];
}

impl From<Key> for InputBinding {
    fn from(key: Key) -> Self {
        InputBinding::Key(key)
    }
}

pub fn is_left_right_arrow_key(key: &Key) -> bool {
    matches!(
        key,
        Key::Named(NamedKey::ArrowLeft) | Key::Named(NamedKey::ArrowRight)
    )
}

fn key_matches(input: &Key, binding: &InputBinding) -> bool {
    match binding {
        InputBinding::Key(b_key) => match (input, b_key) {
            (Key::Character(s1), Key::Character(s2)) => s1.eq_ignore_ascii_case(s2),
            (Key::Named(n1), Key::Named(n2)) => n1 == n2,
            _ => false,
        },
        _ => false,
    }
}

pub fn map_keyboard_input_with_modifiers(key: &Key, ctrl_pressed: bool) -> Option<InputCommand> {
    for mapping in SHORTCUT_MAPPINGS.iter() {
        if mapping.ctrl == ctrl_pressed {
            if mapping.bindings.iter().any(|b| key_matches(key, b)) {
                return Some(mapping.command);
            }
        }
    }
    None
}

pub fn get_shortcut_string(command: InputCommand) -> String {
    let mut parts = Vec::new();
    for mapping in SHORTCUT_MAPPINGS.iter() {
        if mapping.command == command {
            let mut binding_strs = Vec::new();
            for binding in &mapping.bindings {
                let s = binding.to_string();
                if mapping.ctrl {
                    binding_strs.push(format!("Ctrl+{}", s));
                } else {
                    binding_strs.push(s);
                }
            }
            parts.push(binding_strs.join(" | "));
        }
    }
    parts.join(" | ")
}
