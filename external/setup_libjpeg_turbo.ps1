# external/setup_libjpeg_turbo.ps1
$ExternalDir = $PSScriptRoot
$LibsDir = Join-Path $ExternalDir "libs"

$RepoUrl = "https://github.com/libjpeg-turbo/libjpeg-turbo"
$RepoName = "libjpeg-turbo"
$RepoDir = Join-Path $ExternalDir $RepoName

Write-Host "--- Setup $RepoName ---" -ForegroundColor Cyan

if (!(Test-Path $RepoDir)) {
    git clone $RepoUrl $RepoDir
} else {
    Push-Location $RepoDir; git fetch origin; git pull; Pop-Location
}

Push-Location $RepoDir
$NasmCheck = Get-Command "nasm" -ErrorAction SilentlyContinue
$SimdOption = if ($NasmCheck) { "-DWITH_SIMD=ON" } else { "-DWITH_SIMD=OFF" }
if (!(Test-Path "build")) { 
    cmake -G Ninja -B build -DCMAKE_BUILD_TYPE=Release -DENABLE_SHARED=ON -DENABLE_STATIC=ON "-DCMAKE_INSTALL_PREFIX=$LibsDir" $SimdOption
}
cmake --build build --config Release; cmake --install build --config Release
Pop-Location

Write-Host "--- $RepoName Setup Complete! ---" -ForegroundColor Green
