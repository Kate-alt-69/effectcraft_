#Requires -Version 7.0
<#
.SYNOPSIS
Build EffectCraft with your existing Rust toolchain; optionally launch it.
.EXAMPLE
pwsh -File .\build.ps1 -Run
.EXAMPLE
pwsh -File .\build.ps1 -Demo
#>
[CmdletBinding()]
param(
    [switch]$Run,
    [switch]$Demo
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false

Push-Location $PSScriptRoot
try {
    foreach ($tool in @('cargo', 'rustc')) {
        if (-not (Get-Command $tool -CommandType Application -ErrorAction SilentlyContinue)) {
            throw "$tool is missing from PATH. Open a terminal with your existing Rust installation available."
        }
    }
    $rustVersion = & rustc --version
    if ($LASTEXITCODE -ne 0) {
        throw 'Cannot read your active Rust version. Check your existing Rust installation.'
    }
    if ($rustVersion -notmatch '^rustc (\d+\.\d+\.\d+)') {
        throw "Unrecognized Rust version: $rustVersion"
    }
    if ([version]$Matches[1] -lt [version]'1.95.0') {
        throw "EffectCraft needs Rust 1.95 or newer; your active version is $rustVersion. Select a compatible toolchain before rebuilding."
    }

    Write-Host "Building the EffectCraft source in $PSScriptRoot"
    Write-Host "Using $rustVersion; this script does not install or update Rust."
    Write-Host 'Cargo may download the pinned shared FilmCraft and craft-ui libraries from ArtCraft.'
    $cargoArgs = @('build', '--release', '--locked', '-p', 'effectcraft')
    if ($Run -or $Demo) {
        $cargoArgs[0] = 'run'
    }
    if ($Demo) {
        $cargoArgs += @('--', '--demo')
    }
    & cargo @cargoArgs
    if ($LASTEXITCODE -ne 0) {
        throw "Cargo exited with code $LASTEXITCODE. See the build output above."
    }
} catch {
    Write-Host $_.Exception.Message -ForegroundColor Red
    exit 1
} finally {
    Pop-Location
}
