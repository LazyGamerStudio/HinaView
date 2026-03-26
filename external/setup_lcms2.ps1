# external/setup_lcms2.ps1
$ExternalDir = $PSScriptRoot
$LibsDir = Join-Path $ExternalDir "libs"

$RepoUrl = "https://github.com/mm2/Little-CMS"
$RepoName = "lcms2"
$RepoDir = Join-Path $ExternalDir $RepoName

Write-Host "--- Setup $RepoName ---" -ForegroundColor Cyan

if (!(Test-Path $RepoDir)) {
    git clone $RepoUrl $RepoDir
} else {
    Push-Location $RepoDir; git fetch origin; git pull; Pop-Location
}

Push-Location $RepoDir
if (!(Test-Path "build")) {
    # We build static library for lcms2 as requested by main project
    cmake -G Ninja -B build -DCMAKE_BUILD_TYPE=Release -DBUILD_SHARED_LIBS=OFF "-DCMAKE_INSTALL_PREFIX=$LibsDir"
}
cmake --build build --config Release; cmake --install build --config Release
Pop-Location

Write-Host "--- $RepoName Setup Complete! ---" -ForegroundColor Green
