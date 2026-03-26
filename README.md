# HinaView: High-Performance GPU-Accelerated Image Viewer

**HinaView**는 Rust와 WGPU를 기반으로 개발된 초고속 이미지 뷰어입니다. 현대적인 하드웨어 성능을 최대로 활용하기 위해 GPU 가속 렌더링 및 필터링, 그리고 고도로 최적화된 하드웨어 가속 디코더들을 통합하여 설계되었습니다.

## 🚀 주요 특징

### 1. 하드웨어 가속 렌더링 & 필터링
*   **WGPU 기반 그래픽 파이프라인**: 전적으로 GPU에서 동작하는 렌더링 엔진을 통해 수천만 픽셀의 이미지도 지연 없이 부드럽게 출력합니다.
*   **실시간 GPU 필터**: 이미지 품질 향상을 위해 다음과 같은 필터를 실시간으로 적용합니다.
    *   **AMD FSR 1.0 (FidelityFX Super Resolution)**: 저해상도 이미지를 고해상도로 부드럽게 업스케일링합니다.
    *   **Gaussian Blur & Unsharp Mask**: 선명도 조정 및 블러 효과.
    *   **Levels & Color Adjustment**: 입력/출력 레벨 조정을 포함한 고급 색상 보정.
    *   **Median Filter**: 노이즈 제거 필터.

### 2. 고성능 이미지 디코딩
*   **SIMD 가속 디코더**: `libjpeg-turbo`, `libjxl`, `dav1d` 등을 통합하여 CPU의 SIMD(SSE, AVX2, AVX-512) 명령어를 적극 활용합니다.
*   **다양한 포맷 지원**:
    *   **차세대 포맷**: AVIF, HEIC, JXL (JPEG XL) 지원.
    *   **표준 포맷**: JPEG, PNG, WebP, GIF, BMP, TIFF 등.
    *   **특수 포맷**: DDS, EXR, HDR, PNM, ICO 및 CBZ(압축 파일) 지원.
*   **멀티스레드 프리페치**: `Rayon`을 활용한 백그라운드 디코딩으로 다음 페이지를 미리 준비합니다.

### 3. 지능형 레이아웃 및 UX
*   **다양한 보기 모드**: 단일 페이지, 양면 보기(좌->우, 우->좌), 웹툰 모드(수직 스크롤) 지원.
*   **컬러 매니지먼트**: `Little CMS 2`를 통합하여 ICC 프로필 기반의 정확한 색상을 재현합니다.
*   **북마크 시스템**: 자동 최근 기록 및 수동 북마크 기능을 제공하며, SQLite 데이터베이스를 통해 안전하게 관리됩니다.
*   **다국어 지원**: 완벽하게 로컬라이징된 인터페이스(현재 한국어, 영어 등 다수 지원).

---

## 🛠 기술 아키텍처 (Technical Deep Dive)

### 📂 디렉토리 구조
*   `src/pipeline`: 이미지를 로드하고 GPU 텍스처로 변환하며 필터를 적용하는 핵심 파이프라인입니다.
*   `src/renderer`: WGPU를 이용한 저수준 렌더링 명령을 관리합니다.
*   `src/system`: 윈도우 레지스트리(우클릭 메뉴), 단일 인스턴스 통신(IPC) 등 OS 통합 기능을 담당합니다.
*   `src/i18n`: JSON 기반의 정적 타이핑 로컬라이제이션 시스템입니다.
*   `external/`: 외부 C/C++ 라이브러리 소스 및 빌드 자동화 스크립트를 포함합니다.

### 🔗 지연 로딩 (Delay Loading) 전략
HinaView는 대형 DLL 의존성으로 인한 시작 지연을 방지하기 위해 `/DELAYLOAD` 링커 옵션을 사용합니다. 프로그램은 시작 시 최소한의 라이브러리만 로드하며, 특정 코덱(예: AVIF)이 실제로 필요할 때 해당 DLL을 로드합니다. 또한 `SetDllDirectory`를 통해 실행 파일 하위의 `lib/` 폴더를 안전하게 탐색합니다.

### 🧵 하드웨어 최적화
`Highway` 라이브러리를 통해 빌드 시점에 가능한 모든 SIMD 타겟을 바이너리에 포함합니다. 실행 시점에 사용자의 CPU를 감지하여 가장 효율적인 명령어 세트를 자동으로 선택(Runtime Dispatch)하므로, 구형 CPU부터 최신 서버급 CPU까지 최적의 성능을 보장합니다.

---

## 🏗 빌드 방법 (Compilation Guide)

HinaView는 최상의 성능을 위해 여러 고성능 외부 라이브러리를 직접 컴파일하여 링크합니다.

### 📋 요구 사항
*   **Rust**: 1.80버전 이상의 nightly 또는 stable (Edition 2024 사용)
*   **C++ Build Tools**: Visual Studio 2022 (MSVC)
*   **Build Tools**: CMake, Ninja, NASM (SIMD 가속용), Meson (dav1d용)
*   **Git**: 소스 클론용

### 🚀 단계별 빌드 과정

#### 1. 외부 라이브러리 준비
`external` 폴더에는 각 라이브러리를 자동으로 클론하고 최적화 빌드하는 스크립트들이 준비되어 있습니다. 순서대로 실행해 주세요.

```powershell
# 1. 의존성 순서대로 빌드 (PowerShell 관리자 권한 권장)
.\external\setup_dav1d.ps1
.\external\setup_libde265.ps1
.\external\setup_libheif.ps1
.\external\setup_libjxl.ps1
.\external\setup_libjpeg_turbo.ps1
.\external\setup_lcms2.ps1
.\external\setup_crabbyavif.ps1
```

이 과정이 완료되면 `external/libs/lib` 및 `bin` 폴더에 모든 라이브러리(`.lib`)와 DLL(`.dll`)이 준비됩니다.

#### 2. 메인 프로젝트 빌드
표준 Cargo 명령을 사용하여 빌드합니다.

```bash
cargo build --release
```

#### 3. 실행 환경 구성
실행 파일(`HinaView.exe`)이 정상 작동하려면 다음과 같은 구조로 파일이 배치되어야 합니다.
*   `HinaView.exe`
*   `assets/`: (언어 파일 JSON 포함)
*   `lib/`: (`external/libs/bin`에 생성된 모든 `.dll` 파일들)

---


## 📄 라이선스

이 프로젝트는 **MIT 라이선스**에 따라 배포됩니다. 자세한 내용은 [LICENSE](LICENSE) 파일을 참조하세요.

### 서드파티 라이선스

HinaView 는 여러 오픈소스 라이브러리를 사용합니다. 통합된 외부 라이브러리들 (`libjxl`, `libheif`, `dav1d` 등) 은 각각의 오픈소스 라이선스 (Apache 2.0, BSD, LGPL 등) 를 따릅니다. 자세한 내용은 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) 를 참조하세요.
