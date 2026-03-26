# external/setup_libjxl.ps1
$ExternalDir = $PSScriptRoot
$LibsDir = Join-Path $ExternalDir "libs"

$RepoUrl = "https://github.com/libjxl/libjxl"
$RepoName = "libjxl"
$RepoDir = Join-Path $ExternalDir $RepoName

Write-Host "--- Setup $RepoName ---" -ForegroundColor Cyan

if (!(Test-Path $RepoDir)) {
    git clone --recursive $RepoUrl $RepoDir
} else {
    Push-Location $RepoDir; git fetch origin; git pull; git submodule update --init --recursive; Pop-Location
}

Push-Location $RepoDir
if (!(Test-Path "build")) {
    # JPEGXL_ENABLE_AVX512=ON: Force include AVX-512 for runtime dispatch
    # HWY_SKIP_CAN_COMPILE: Ensure it tries to build all targets the compiler supports
    cmake -G Ninja -B build -DCMAKE_BUILD_TYPE=Release -DBUILD_SHARED_LIBS=ON `
        "-DCMAKE_INSTALL_PREFIX=$LibsDir" -DJPEGXL_ENABLE_TOOLS=OFF `
        -DJPEGXL_ENABLE_EXAMPLES=OFF -DJPEGXL_ENABLE_BENCHMARK=OFF `
        -DJXL_ENABLE_MANPAGES=OFF -DBUILD_TESTING=OFF -DJPEGXL_ENABLE_SJPEG=OFF `
        -DJPEGXL_ENABLE_SKCMS=OFF -DJPEGXL_FORCE_SYSTEM_LCMS2=OFF `
        -DJPEGXL_ENABLE_AVX512=ON
}
cmake --build build --config Release; cmake --install build --config Release

# Renaming for compatibility
$JxlMap = @( "build\lib\jxl.lib", "libjxl.lib"; "build\lib\jxl_cms.lib", "libjxl_cms.lib"; "build\lib\jxl_threads.lib", "libjxl_threads.lib"; "build\third_party\brotli\brotlicommon.lib", "libbrotlicommon.lib"; "build\third_party\brotli\brotlidec.lib", "libbrotlidec.lib"; "build\third_party\brotli\brotlienc.lib", "libbrotlienc.lib"; "build\third_party\highway\hwy.lib", "libhwy.lib" )
for ($i=0; $i -lt $JxlMap.Length; $i+=2) { 
    $src = $JxlMap[$i]; $name = $JxlMap[$i+1]
    if (Test-Path $src) { Copy-Item $src (Join-Path (Split-Path $src) $name) -Force } 
}
Pop-Location

Write-Host "--- $RepoName Setup Complete! ---" -ForegroundColor Green
