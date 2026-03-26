# external/setup_crabbyavif.ps1
$ExternalDir = $PSScriptRoot
$LibsDir = Join-Path $ExternalDir "libs"

$RepoUrl = "https://github.com/webmproject/CrabbyAvif"
$RepoName = "CrabbyAvif"
$RepoDir = Join-Path $ExternalDir $RepoName

Write-Host "--- Setup $RepoName ---" -ForegroundColor Cyan

if (!(Test-Path $RepoDir)) {
    git clone $RepoUrl $RepoDir
} else {
    Push-Location $RepoDir; git fetch origin; git pull; Pop-Location
}

# 1. Junctions
$CrabbySys = Join-Path $RepoDir "sys"
$Junctions = @(
    @{ target = Join-Path $CrabbySys "dav1d-sys\dav1d"; source = Join-Path $ExternalDir "dav1d" },
    @{ target = Join-Path $CrabbySys "libjxl-sys\libjxl"; source = Join-Path $ExternalDir "libjxl" }
)
foreach ($j in $Junctions) {
    if (!(Test-Path $j.source)) {
        Write-Host "  [WARN] Source $($j.source) not found! Run setup script for it first." -ForegroundColor Yellow
        continue
    }
    if (Test-Path $j.target) {
        if ((Get-Item $j.target).Attributes -match "ReparsePoint") { cmd /c rmdir "$($j.target)" } else { Remove-Item -Recurse -Force $j.target }
    }
    $parent = Split-Path $j.target
    if (!(Test-Path $parent)) { New-Item -ItemType Directory -Path $parent }
    cmd /c mklink /J "$($j.target)" "$($j.source)"
}

# 2. Patching sys crates
Write-Host "--- Patching sys crates for HinaView support ---" -ForegroundColor Yellow

$DavBuildRs = Join-Path $CrabbySys "dav1d-sys\build.rs"
if (Test-Path $DavBuildRs) {
    (Get-Content $DavBuildRs) -replace 'join\("libdav1d\.a"\)', 'join("dav1d.lib")' | Set-Content $DavBuildRs
}

$JxlBuildRs = Join-Path $CrabbySys "libjxl-sys\build.rs"
if (Test-Path $JxlBuildRs) {
    $content = Get-Content $JxlBuildRs
    
    # 1. Remove lcms2 from objects array
    $content = $content -replace '\["lcms2", "third_party", "\."\],', ''
    
    # 2. Add jxl_threads if missing (already doing this but let's be sure)
    if ($content -notmatch 'jxl_threads') {
        $content = $content -replace '\["jxl_cms", "lib", "\."\],', '["jxl_cms", "lib", "."], ["jxl_threads", "lib", "."],'
    }

    # 3. Fix library prefix for Windows (MSVC uses .lib without lib prefix often, but we renamed them to libxxx.lib)
    # The original code adds "lib" prefix on Windows. Our setup_libjxl.ps1 renames them to libjxl.lib etc.
    # So the prefix is actually correct, but lcms2 being missing is the main issue.

    # 4. Add missing symbols for bindgen
    $newSymbols = @("JxlDecoderSetParallelRunner", "JxlThreadParallelRunner", "JxlThreadParallelRunnerCreate")
    foreach ($sym in $newSymbols) {
        if ($content -notmatch "`"$($sym)`"") {
            $content = $content -replace '"JxlBasicInfo",', "`"$($sym)`",`r`n        `"JxlBasicInfo`","
        }
    }
    $content | Set-Content $JxlBuildRs
}

$JxlWrapper = Join-Path $CrabbySys "libjxl-sys\wrapper.h"
if (Test-Path $JxlWrapper) {
    $c = Get-Content $JxlWrapper
    if ($c -notmatch 'thread_parallel_runner\.h') {
        $c += "`n#include <jxl/thread_parallel_runner.h>"
        $c | Set-Content $JxlWrapper
    }
}

Write-Host "--- $RepoName Setup Complete! ---" -ForegroundColor Green
