// src/color_management/lcms2_ffi.rs

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::os::raw::{c_double, c_void};

pub type cmsContext = *mut c_void;
pub type cmsHPROFILE = *mut c_void;
pub type cmsHTRANSFORM = *mut c_void;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct cmsCIEXYZ {
    pub X: c_double,
    pub Y: c_double,
    pub Z: c_double,
}

pub type cmsToneCurve = *mut c_void;

pub const cmsSigRedColorantTag: u32 = 0x7258595A; // 'rXYZ'
pub const cmsSigGreenColorantTag: u32 = 0x6758595A; // 'gXYZ'
pub const cmsSigBlueColorantTag: u32 = 0x6258595A; // 'bXYZ'
pub const cmsSigRedTRCTag: u32 = 0x72545243; // 'rTRC'
pub const cmsSigGreenTRCTag: u32 = 0x67545243; // 'gTRC'
pub const cmsSigBlueTRCTag: u32 = 0x62545243; // 'bTRC'
pub const cmsSigMediaWhitePointTag: u32 = 0x77747074; // 'wtpt'

unsafe extern "C" {
    pub fn cmsOpenProfileFromMem(data: *const c_void, size: u32) -> cmsHPROFILE;
    pub fn cmsCloseProfile(profile: cmsHPROFILE);
    pub fn cmsReadTag(profile: cmsHPROFILE, sig: u32) -> *mut c_void;
    pub fn cmsEstimateGamma(curve: cmsToneCurve, precision: c_double) -> c_double;
}
