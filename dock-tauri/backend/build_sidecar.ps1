# Freezes the Python sidecar into a single exe for release packaging.
#
# Requires PyInstaller, which is NOT installed by default:
#   D:/jiqixuexi/anaconda/python.exe -m pip install pyinstaller
#
# This script produces src-tauri/binaries/monitor.exe. Wiring it into the Tauri
# bundle additionally needs a `bundle.externalBin` entry in tauri.conf.json and a
# target-triple-suffixed filename (monitor-x86_64-pc-windows-msvc.exe); that is
# deliberately left for the packaging task rather than half-done here.

$ErrorActionPreference = "Stop"
$python = "D:/jiqixuexi/anaconda/python.exe"

& $python -c "import PyInstaller" 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Error "PyInstaller is not installed. Run: $python -m pip install pyinstaller"
}

Push-Location $PSScriptRoot
try {
    & $python -m PyInstaller `
        --onefile `
        --name monitor `
        --distpath ../src-tauri/binaries `
        --workpath ../build/pyinstaller `
        --specpath ../build `
        run_sidecar.py
} finally {
    Pop-Location
}

Write-Host "Sidecar written to src-tauri/binaries/monitor.exe"
