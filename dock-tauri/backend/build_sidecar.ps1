# Freezes the Python sidecar into a single exe for release packaging.
#
# Requires PyInstaller, which is NOT installed by default:
#   D:/jiqixuexi/anaconda/python.exe -m pip install pyinstaller
#
# This script produces src-tauri/binaries/monitor-x86_64-pc-windows-msvc.exe.
# Tauri requires externalBin files to carry the target triple in their name; the
# triple below is what `rustc -vV` reports for this host. Keeping stdout is
# mandatory: the sidecar speaks newline JSON on the pipe, so no --windowed.

$ErrorActionPreference = "Stop"
$python = "D:/jiqixuexi/anaconda/python.exe"
$targetTriple = "x86_64-pc-windows-msvc"

& $python -c "import PyInstaller" 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Error "PyInstaller is not installed. Run: $python -m pip install pyinstaller"
}

Push-Location $PSScriptRoot
try {
    & $python -m PyInstaller `
        --onefile `
        --noconfirm `
        --name "monitor-$targetTriple" `
        --distpath ../src-tauri/binaries `
        --workpath ../build/pyinstaller `
        --specpath ../build `
        run_sidecar.py
    if ($LASTEXITCODE -ne 0) {
        throw "PyInstaller failed with exit code $LASTEXITCODE"
    }
} finally {
    Pop-Location
}

Write-Host "Sidecar written to src-tauri/binaries/monitor-$targetTriple.exe"
