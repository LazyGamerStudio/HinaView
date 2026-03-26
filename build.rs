use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    if cfg!(target_os = "windows") {
        let project_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let project_path = Path::new(&project_dir);

        // 1. 아이콘 및 리소스 컴파일
        let icon_dir = project_path.join("icon_win");
        let rc_path = icon_dir.join("icons_generated.rc");
        let icon_mappings = [
            (1, "hinaview.ico"),
            (101, "webp.ico"),
            (102, "avif.ico"),
            (103, "heif.ico"),
            (104, "heic.ico"),
            (105, "jxl.ico"),
            (106, "jpeg.ico"),
            (107, "png.ico"),
            (108, "gif.ico"),
            (109, "bmp.ico"),
            (110, "tiff.ico"),
            (111, "tga.ico"),
            (112, "dds.ico"),
            (113, "exr.ico"),
            (114, "hdr.ico"),
            (115, "pnm.ico"),
            (116, "ico.ico"),
            (201, "cbz.ico"),
        ];

        let mut rc_content = String::new();
        for (id, icon_file) in icon_mappings.iter() {
            let icon_path = icon_dir
                .join(icon_file)
                .to_string_lossy()
                .replace('\\', "/");
            rc_content.push_str(&format!("{} ICON \"{}\"\n", id, icon_path));
        }
        fs::write(&rc_path, rc_content).ok();
        let _ = embed_resource::compile(&rc_path, embed_resource::NONE);

        // 2. 동적 링크 설정 및 빌드된 DLL 복사
        let profile = env::var("PROFILE").unwrap();
        let target_dir = project_path.join("target").join(&profile);

        // tools/setup_external.ps1에 의해 생성된 경로 참조
        let lib_path = project_path.join("external").join("libs").join("lib");
        let bin_path = project_path.join("external").join("libs").join("bin");

        println!("cargo:rustc-link-search=native={}", lib_path.display());
        println!("cargo:rustc-link-lib=delayimp");

        // DLL들을 target 디렉토리로 복사하여 실행 가능하게 함
        if bin_path.exists() {
            if let Ok(entries) = fs::read_dir(&bin_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("dll") {
                        let file_name = path.file_name().unwrap().to_str().unwrap();
                        let dest = target_dir.join(file_name);
                        fs::copy(&path, &dest).ok();

                        // 지연 로딩 설정
                        println!("cargo:rustc-link-arg=/DELAYLOAD:{}", file_name);
                    }
                }
            }
        }

        let updater_path = project_path
            .join("updater")
            .join("target")
            .join("release")
            .join("updater.exe");
        if updater_path.exists() {
            let dest = target_dir.join("updater.exe");
            fs::copy(&updater_path, &dest).ok();
        }
    }

    println!("cargo:rustc-link-lib=user32");
    println!("cargo:rustc-link-lib=shell32");
    println!("cargo:rustc-link-lib=dylib=heif");
    println!("cargo:rustc-link-lib=dylib=de265");
    println!("cargo:rustc-link-lib=dylib=lcms2");
}
