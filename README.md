# HinaView: High-Performance GPU-Accelerated Image Viewer

[한국어](#한국어-korean) | [English](#english)

---

<br>

<a id="한국어-korean"></a>
## 한국어 (Korean)

**HinaView**는 Rust와 WGPU를 기반으로 개발된 초고속 이미지 뷰어입니다. 현대적인 하드웨어 성능을 최대로 활용하기 위해 GPU 가속 렌더링 및 필터링, 그리고 고도로 최적화된 하드웨어 가속 디코더들을 통합하여 설계되었습니다.

### 🚀 주요 특징

#### 1. 하드웨어 가속 렌더링 & 지능형 이미지 처리
*   **WGPU 기반 그래픽 파이프라인**: 전적으로 GPU에서 동작하는 렌더링 엔진을 통해 수천만 픽셀의 이미지도 지연 없이 부드럽게 출력합니다.
*   **자동 하프톤 처리 (Auto Halftone Processing)**: 만화 스캔본 등에서 자주 발생하는 모아레(Moiré) 현상과 하프톤 패턴을 알고리즘으로 자동 감지합니다. 패턴이 감지되면 리사이징 전 가우시안 프리블러(Pre-blur)를 적용하고, 축소 필터를 일반적인 Lanczos3 대신 Mitchell 알고리즘으로 자동 전환하여 모아레 없는 깨끗한 화질을 유지합니다.
*   **실시간 필터** (⚠️ **실험적 기능: RGBA 24비트 처리 한계로 인해 현재 실사용을 권장하지 않습니다.**): 이미지 품질 향상을 위해 다음과 같은 필터를 지원합니다.
    *   **AMD FidelityFX™ FSR 1.0 (FidelityFX Super Resolution)**: 저해상도 이미지를 고해상도로 부드럽게 업스케일링합니다.
    *   **Gaussian Blur & Unsharp Mask**: 선명도 조정 및 블러 효과.
    *   **Levels & Color Adjustment**: 입력/출력 레벨 조정을 포함한 고급 색상 보정.
    *   **Median Filter**: 노이즈 제거 필터.

#### 2. 고성능 이미지 디코딩
*   **SIMD 가속 디코더**: `libjpeg-turbo`, `libjxl`, `dav1d` 등을 통합하여 CPU의 SIMD(SSE, AVX2, AVX-512) 명령어를 적극 활용합니다.
*   **다양한 포맷 지원**:
    *   **차세대 포맷**: AVIF, HEIC, JXL (JPEG XL) 지원.
    *   **표준 포맷**: JPEG, PNG, WebP, GIF, BMP, TIFF 등.
    *   **특수 포맷**: DDS, EXR, HDR, PNM, ICO 및 CBZ(압축 파일) 지원.
*   **멀티스레드 프리페치**: `Rayon`을 활용한 백그라운드 디코딩으로 다음 페이지를 미리 준비합니다.

#### 3. 지능형 레이아웃 및 UX
*   **다양한 보기 모드**: 단일 페이지, 양면 보기(좌->우, 우->좌), 웹툰 모드(수직 스크롤) 지원.
*   **컬러 매니지먼트**: `Little CMS 2`를 통합하여 ICC 프로필 기반의 정확한 색상을 재현합니다.
*   **북마크 시스템**: 자동 최근 기록 및 수동 북마크 기능을 제공하며, SQLite 데이터베이스를 통해 안전하게 관리됩니다.
*   **다국어 지원**: 완벽하게 로컬라이징된 인터페이스(현재 한국어, 영어 등 다수 지원).

---

### 🛠 기술 아키텍처 (Technical Deep Dive)

*   `src/pipeline`: 이미지를 로드하고 GPU 텍스처로 변환하며 필터를 적용하는 핵심 파이프라인입니다.
*   `src/renderer`: WGPU를 이용한 저수준 렌더링 명령을 관리합니다.
*   `src/system`: 윈도우 레지스트리, 단일 인스턴스 통신(IPC) 등 OS 통합 기능을 담당합니다.
*   `src/i18n`: JSON 기반의 정적 타이핑 로컬라이제이션 시스템입니다.
*   **지연 로딩 (Delay Loading) 전략**: 의존성으로 인한 시작 지연을 방지하기 위해 `/DELAYLOAD` 링커 옵션을 사용하며, DLL 모듈은 필요할 때만 로드됩니다.
*   **런타임 디스패치 (Runtime Dispatch)**: `Highway` 라이브러리를 통해 실행 시점에 사용자의 CPU를 감지하여 최적의 SIMD 명령어 세트를 적용합니다.

---

### 🏗 빌드 방법 (Compilation Guide)

#### 1. 외부 라이브러리 준비
`external` 폴더에는 최적화된 고성능 코덱과 라이브러리를 자동으로 다운로드하고 빌드하는 스크립트들이 준비되어 있습니다. 반드시 아래 순서대로 실행해 주세요.

```powershell
# 옵션: PowerShell 관리자 권한 권장
.\external\setup_dav1d.ps1
.\external\setup_libde265.ps1
.\external\setup_libheif.ps1
.\external\setup_libjxl.ps1
.\external\setup_libjpeg_turbo.ps1
.\external\setup_lcms2.ps1
.\external\setup_crabbyavif.ps1
```

이 과정이 완료되면 `external/libs/bin` 폴더 내에 모든 필수 DLL 파일들이 생성됩니다.

#### 2. 메인 프로젝트 빌드
Cargo 명령어를 사용하여 앱 본체를 빌드합니다.

```bash
cargo build --release
```

#### 3. 실행 환경 구성
생성된 `HinaView.exe`가 정상 작동하려면 다음 구조로 파일들이 배치되어야 합니다:
*   `HinaView.exe` (메인 실행 파일)
*   `assets/` (언어 팩 등 정적 리소스)
*   `lib/` (`external/libs/bin`에 생성된 모든 `.dll` 파일들을 이곳에 복사)

---

> 💡 **AI 어시스턴트 활용**: 이 프로젝트의 코드는 약 70% Gemini 3 Flash, 20% GPT-5.4-Codex, 10% Gemini 3.1 Pro 로 작성되었습니다 (추정치).

<br><br>

<a id="english"></a>
## English

**HinaView** is an ultra-fast image viewer developed based on Rust and WGPU. It is designed to maximize modern hardware performance by integrating GPU-accelerated rendering and highly optimized hardware-accelerated decoders.

### 🚀 Key Features

#### 1. Hardware Accelerated Rendering & Smart Image Processing
*   **WGPU-based Graphics Pipeline**: A fully GPU-accelerated rendering engine that smoothly displays multi-megapixel images without latency.
*   **Auto Halftone Processing**: Automatically detects moiré and halftone patterns commonly found in manga/comic scans using algorithmic variance checks. When a pattern is detected, it applies a Gaussian pre-blur before downscaling and smartly switches the resampling filter from the default *Lanczos3* to *Mitchell*, effectively preserving image clarity while suppressing moiré artifacts.
*   **Real-time Filters** (⚠️ **Experimental Feature: Currently NOT recommended for use due to limitations in 24-bit RGBA color processing.**)
    *   **AMD FidelityFX™ FSR 1.0 (FidelityFX Super Resolution)**: Smoothly upscales low-resolution images.
    *   **Gaussian Blur & Unsharp Mask**: Adjusts sharpness and applies blur effects.
    *   **Levels & Color Adjustment**: Advanced color correction.
    *   **Median Filter**: Noise reduction.

#### 2. High-Performance Image Decoding
*   **SIMD Accelerated Decoders**: Seamlessly integrates `libjpeg-turbo`, `libjxl`, `dav1d`, and more, heavily leveraging CPU SIMD instructions (SSE, AVX2, AVX-512).
*   **Extensive Format Support**: AVIF, HEIC, JXL, JPEG, PNG, WebP, GIF, BMP, TIFF, DDS, EXR, HDR, PNM, ICO, and CBZ (Archives).
*   **Multi-threaded Prefetching**: Uses `Rayon` for background decoding to preemptively load next pages.

#### 3. Intelligent Layout & UX
*   **Versatile Viewing Modes**: Single page, double page (LTR/RTL), and Webtoon mode (Vertical scrolling).
*   **Color Management**: Integrates `Little CMS 2` for highly accurate ICC profile-based color reproduction.
*   **Advanced Bookmark System**: Supports automated history logs and manual bookmarks backed by SQLite.
*   **Multilingual Support**: Fully localized interface including English, Korean, and many others.

---

### 🛠 Technical Deep Dive

*   **Delay Loading Strategy**: Employs the MSVC `/DELAYLOAD` linker flag to prevent startup latency caused by heavy DLLs. Modules are dynamically loaded only when specifically required.
*   **Runtime SIMD Dispatch**: Integrates the `Highway` library to evaluate your CPU topology at runtime, executing the optimum path of SIMD instructions without needing customized micro-architecture builds.

---

### 🏗 Compilation Guide

#### 1. Prepare External Libraries
The `external` folder contains scripts to automatically clone and build highly optimized codecs and dependencies. Please execute them in the following order:

```powershell
# Note: PowerShell with Administrator privileges is recommended
.\external\setup_dav1d.ps1
.\external\setup_libde265.ps1
.\external\setup_libheif.ps1
.\external\setup_libjxl.ps1
.\external\setup_libjpeg_turbo.ps1
.\external\setup_lcms2.ps1
.\external\setup_crabbyavif.ps1
```

Once completed, all necessary DLLs will be generated in `external/libs/bin`.

#### 2. Build the Main Project
Build the application using standard Cargo commands.

```bash
cargo build --release
```

#### 3. Execution Environment Setup
For `HinaView.exe` to function correctly, deploy the files in the following structure:
*   `HinaView.exe` (Main executable)
*   `assets/` (Static resources including language packs)
*   `lib/` (Copy all `.dll` files generated in `external/libs/bin` to this folder)

---

> 💡 **AI Assistant Contribution**: This project's codebase was written with approximately 70% Gemini 3 Flash, 20% GPT-5.4-Codex, and 10% Gemini 3.1 Pro (estimated).

---

## 📄 License
This project is distributed under the **MIT License**. Refer to the `LICENSE` file for more details. For third-party open-source components, please refer to `THIRD_PARTY_NOTICES.md`.
