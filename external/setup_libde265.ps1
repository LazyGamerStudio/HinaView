# external/setup_libde265.ps1
$ExternalDir = $PSScriptRoot
$LibsDir = Join-Path $ExternalDir "libs"

$RepoUrl = "https://github.com/strukturag/libde265"
$RepoName = "libde265"
$RepoDir = Join-Path $ExternalDir $RepoName

Write-Host "--- Setup $RepoName ---" -ForegroundColor Cyan

if (!(Test-Path $RepoDir)) {
    git clone $RepoUrl $RepoDir
} else {
    Push-Location $RepoDir; git fetch origin; git pull; Pop-Location
}

Push-Location $RepoDir
if (!(Test-Path "build")) {
    cmake -G Ninja -B build -DCMAKE_BUILD_TYPE=Release -DBUILD_SHARED_LIBS=ON "-DCMAKE_INSTALL_PREFIX=$LibsDir"
}
cmake --build build --config Release; cmake --install build --config Release
Pop-Location

Write-Host "--- $RepoName Setup Complete! ---" -ForegroundColor Green
