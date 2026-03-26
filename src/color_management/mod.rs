// src/color_management/mod.rs

mod controller;
mod display_profile;
mod lcms2_ffi;
mod profile;

pub use controller::ColorManagementController;
