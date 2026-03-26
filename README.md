# HinaView: 고성능 GPU 가속 이미지 뷰어 (Deep Technical Overview)

[한국어](#한국어-korean) | [English](#english)

---

<br>

<a id="한국어-korean"></a>
## 한국어 (Korean)

**HinaView**는 현대적인 하드웨어 성능을 한계까지 끌어쓰도록 설계된 Rust 기반의 초고속 이미지 뷰어입니다. 단순한 이미지 출력을 넘어, 수만 장의 대규모 아카이브와 고용량 애니메이션 WebP를 저사양 스토리지(HDD)에서도 끊김 없이 탐색할 수 있도록 고도의 엔지니어링이 적용되었습니다.

### 🛠 핵심 아키텍처 및 상세 분석 (Technical Deep Dive)

HinaView의 탁월한 성능은 다음과 같은 독창적인 기술적 스택 위에서 구현되었습니다.

#### 1. 지능형 이미지 샘플링 파이프라인 (`src/sampling`)
*   **실시간 하프톤 감지 (Laplacian Variance Detection)**: 이미지의 라플라시안 분산을 분석하여 인쇄물 특유의 하프톤 패턴(망점)을 실시간으로 감지합니다. 
*   **Moiré 현상 방지 전략**: 하프톤이 감지되면 리사이징 시 발생하는 계단 현상과 모아레를 막기 위해 **Gaussian Pre-blur**를 선제적으로 적용하고, 리샘플링 알고리즘을 **Lanczos3에서 Mitchell**로 자동 전환하여 부드럽고 깨끗한 화질을 유지합니다.
*   **다단계 가속 다운스케일링**: 8000px 이상의 거대 이미지를 10% 미만으로 축소할 때, **Progressive Box Downscale** 단계를 거쳐 메모리 대역폭을 절약하고 가독성을 높입니다.

#### 2. 하이브리드 병렬 디코딩 파이프라인 (`src/pipeline`)
*   **SIMD 런타임 디스패치 (Runtime Dispatch)**: `Highway` 라이브러리를 통해 사용자의 CPU가 SSE, AVX2, 혹은 AVX-512를 지원하는지 런타임에 감지하여 최적의 명령어를 실행합니다.
*   **네이티브 코덱 통합**: 
    *   `libjpeg-turbo`: SIMD 가속을 통한 업계 최속의 JPEG 디코딩.
    *   `libjxl`: 차세대 포맷 JPEG XL 지원.
    *   `dav1d`: 고성능 AV1/AVIF 오픈소스 디코더.
    *   `libwebp-sys`: WebP 및 Animated WebP 지원.
*   **지능형 스케줄러**: `Rayon` 워커 풀과 `DecodingSemaphore`를 결합하여 시스템 부하에 따라 디코딩 스레드를 동적으로 관리하며, 사용자가 보고 있는 페이지(Urgent)와 다음 페이지(Prefetch)의 우선순위를 엄격히 관리합니다.

#### 3. WGPU 기반 GPGPU 내비게이션 엔진 (`src/view`)
*   **Pipe-Safe VRAM 관리**: 고속 탐색 시 VRAM 폭주를 막기 위해 유효 범위 밖의 텍스처를 제거하되, 현재 페이지(`current`), 전환 중인 페이지(`pending`), 가속 중인 목표 지점(`target`)을 화이트리스트로 보호하여 HDD 환경에서도 끊김 없는 탐색을 보장합니다.
*   **비동기 애니메이션 스트리밍**: 애니메이션 WebP를 위해 독립적인 `FrameStream` 시스템을 구축하여, 메인 내비게이션 로직과 별개로 매 프레임을 GPU로 펌핑합니다.

---

### 🏗 상세 빌드 방법 (Compilation Guide)

#### 1. 필수 요구사항 (Prerequisites)
*   **Rust 1.80+**: stable 채널 (2024 Edition).
*   **CMake 3.20+** 와 **Ninja**: 고성능 네이티브 빌드 도구.
*   **Visual Studio 2022 (MSVC)**: C++ 컴파일러 및 Windows 10/11 SDK.
*   **Git**: 외부 의존성 관리용.

#### 2. 외부 코덱 사전 빌드
`external` 디렉토리 내의 스크립트들을 순서대로 실행하여 최적화된 바이너리를 생성합니다.

```powershell
# 관리자 권한 PowerShell에서 실행 권장
.\external\setup_dav1d.ps1         # AV1/AVIF
.\external\setup_libde265.ps1      # HEIF Base
.\external\setup_libheif.ps1       # HEIC/HEIF
.\external\setup_libjxl.ps1        # JPEG XL
.\external\setup_libjpeg_turbo.ps1 # Fast JPEG
.\external\setup_lcms2.ps1         # ICC Color Management
.\external\setup_crabbyavif.ps1    # Rust AVIF Bridge
```

#### 3. 프로젝트 빌드 및 배치
```bash
# 릴리스 빌드 (LTO 및 높은 최적화 레벨 적용)
cargo build --release
```

**배포 구조:**
1.  `HinaView.exe`를 실행 폴더에 배치.
2.  `external/libs/bin`에 생성된 모든 `.dll` 파일들을 `lib/` 폴더로 복사.
3.  `assets/` 폴더(언어 팩 등)를 실행 파일 경로와 동일하게 배치.

---

<br>

<a id="english"></a>
## English

**HinaView** is a high-performance image viewer built with Rust and WGPU, designed to maximize modern hardware capabilities. Beyond basic viewing, it employs advanced engineering to ensure seamless browsing of massive archives and large animated WebP files, even on slower storage media like HDDs.

### 🛠 Technical Architecture & Deep Dive

#### 1. Intelligent Image Sampling Pipeline (`src/sampling`)
*   **Real-time Halftone Detection**: Analyzes Laplacian variance in real-time to detect halftone patterns common in scanned print media.
*   **Smart Moiré Suppression**: When halftone is detected, the pipeline automatically applies a **Gaussian Pre-blur** and switches from **Lanczos3 to Mitchell** resampling. This effectively suppresses moiré artifacts while keeping the image clear.
*   **Progressive Box Downscaling**: For massive images (>8000px), it performs stage-based downscaling to conserve memory bandwidth and enhance readability.

#### 2. Hybrid Parallel Decoding Pipeline (`src/pipeline`)
*   **Runtime SIMD Dispatch**: Leverages the `Highway` library to probe CPU topologies (SSE, AVX2, AVX-512) at runtime, executing the optimal instruction path for the host machine.
*   **High-Speed Native Codecs**:
    *   `libjpeg-turbo`: Industry-leading SIMD-accelerated JPEG decoding.
    *   `libjxl`: Support for the next-generation JPEG XL format.
    *   `dav1d`: Highly optimized open-source AV1/AVIF decoder.
    *   `libwebp-sys`: Robust support for WebP and high-frame-rate Animated WebP.
*   **Prioritized Task Scheduler**: Combines `Rayon` worker pools with a `DecodingSemaphore` to dynamically manage thread counts and balance urgent view requests against prefetch tasks.

#### 3. WGPU-Accelerated Navigation Engine (`src/view`)
*   **Pipe-Safe VRAM Management**: Automatically prunes unused textures while **whitelisting current, pending, and target pages**. This ensures navigation remains responsive by preventing the UI from stalling due to premature cache eviction on slow HDDs.
*   **Async Animation Streaming**: Uses a dedicated `FrameStream` system for animated images, decoupling frame updates from the main navigation logic to maintain high playback performance.

---

### 🏗 Detailed Compilation Guide

#### 1. Prerequisites
*   **Rust 1.80+**: Minimal stable version (2024 Edition).
*   **CMake 3.20+** & **Ninja**: High-performance native build tools.
*   **Visual Studio 2022 (MSVC)**: C++ compiler and Windows SDK.
*   **Git**: For cloning submodules and sources.

#### 2. Build Native Dependencies
Execute the scripts in the `external/` directory to build optimized binaries tailored to your hardware.

```powershell
# Recommended to run from an elevated PowerShell prompt
.\external\setup_dav1d.ps1         # AV1/AVIF Decoder
.\external\setup_libde265.ps1      # HEIF Engine
.\external\setup_libheif.ps1       # HEIC/HEIF Decoder
.\external\setup_libjxl.ps1        # JPEG XL (AVX-512 explicitly enabled)
.\external\setup_libjpeg_turbo.ps1 # Fast JPEG Decoder
.\external\setup_lcms2.ps1         # Little CMS 2 Color Management
.\external\setup_crabbyavif.ps1    # CrabbyAVIF Rust Bridge
```

#### 3. Compile and Deploy
```bash
# Full Release Build with LTO and Opt-Level 3
cargo build --release
```

**Deployment Structure:**
1.  Place `HinaView.exe` in the root deployment folder.
2.  Copy all `.dll` files from `external/libs/bin` to a `lib/` subfolder.
3.  Ensure the `assets/` folder is present in the executable's directory.

---

> 💡 **AI Assistant Contribution**: Parts of this codebase were co-authored with Gemini 3 Flash and GPT-4o-Codex models.

## 📄 License
MIT License. See `LICENSE` and `THIRD_PARTY_NOTICES.md` for details.
