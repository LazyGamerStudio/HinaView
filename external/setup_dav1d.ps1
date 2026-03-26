# external/setup_dav1d.ps1
$ExternalDir = $PSScriptRoot
$LibsDir = Join-Path $ExternalDir "libs"

$RepoUrl = "https://github.com/videolan/dav1d"
$RepoName = "dav1d"
$RepoDir = Join-Path $ExternalDir $RepoName

Write-Host "--- Setup $RepoName ---" -ForegroundColor Cyan

if (!(Test-Path $RepoDir)) {
    git clone $RepoUrl $RepoDir
} else {
    Push-Location $RepoDir; git fetch origin; git pull; Pop-Location
}

Push-Location $RepoDir
if (!(Test-Path "build\build.ninja")) {
    meson setup build --buildtype=release --default-library=shared "--prefix=$LibsDir"
}
meson compile -C build; meson install -C build
Pop-Location

Write-Host "--- $RepoName Setup Complete! ---" -ForegroundColor Green
