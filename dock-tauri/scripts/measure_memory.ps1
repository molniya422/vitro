# Measures the two memory gates and idle CPU for dock-tauri.
#
#   host  = the dock-tauri.exe process alone (gate: < 30 MB private bytes)
#   total = the whole process tree, including every msedgewebview2 child
#           (gate: < 200 MB private bytes)
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File scripts/measure_memory.ps1
#   powershell -ExecutionPolicy Bypass -File scripts/measure_memory.ps1 -SettleSeconds 60 -Runs 3

param(
    [int]$SettleSeconds = 60,
    [int]$SampleSeconds = 10,
    [int]$Runs = 3
)

$ErrorActionPreference = "Stop"
$processName = "dock-tauri"

function Get-ProcessTree {
    param([int]$RootId)
    $all = Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId, Name
    $ids = @($RootId)
    $frontier = @($RootId)
    while ($frontier.Count -gt 0) {
        $next = @()
        foreach ($parent in $frontier) {
            foreach ($child in $all | Where-Object { $_.ParentProcessId -eq $parent }) {
                if ($ids -notcontains $child.ProcessId) {
                    $ids += $child.ProcessId
                    $next += $child.ProcessId
                }
            }
        }
        $frontier = $next
    }
    return $ids
}

function Measure-Once {
    $hostProcess = Get-Process -Name $processName -ErrorAction SilentlyContinue |
        Sort-Object StartTime | Select-Object -First 1
    if (-not $hostProcess) {
        throw "dock-tauri is not running. Start it with 'npm run tauri dev' or the built exe."
    }

    $hostMb = [math]::Round($hostProcess.PrivateMemorySize64 / 1MB, 1)
    $treeIds = Get-ProcessTree -RootId $hostProcess.Id

    $childMb = 0.0
    $breakdown = @()
    foreach ($id in $treeIds) {
        if ($id -eq $hostProcess.Id) { continue }
        $child = Get-Process -Id $id -ErrorAction SilentlyContinue
        if (-not $child) { continue }
        $mb = [math]::Round($child.PrivateMemorySize64 / 1MB, 1)
        $childMb += $mb
        $breakdown += ("{0}#{1} {2}MB" -f $child.ProcessName, $id, $mb)
    }

    # Idle CPU for the whole tree over SampleSeconds.
    $ids = $treeIds
    $start = @{}
    foreach ($id in $ids) {
        $p = Get-Process -Id $id -ErrorAction SilentlyContinue
        if ($p) { $start[$id] = $p.TotalProcessorTime.TotalMilliseconds }
    }
    Start-Sleep -Seconds $SampleSeconds
    $delta = 0.0
    foreach ($id in $ids) {
        if (-not $start.ContainsKey($id)) { continue }
        $p = Get-Process -Id $id -ErrorAction SilentlyContinue
        if ($p) { $delta += ($p.TotalProcessorTime.TotalMilliseconds - $start[$id]) }
    }
    $cpuPercent = [math]::Round(
        ($delta / ($SampleSeconds * 1000) / [Environment]::ProcessorCount) * 100, 2)

    [pscustomobject]@{
        HostMB     = $hostMb
        ChildrenMB = [math]::Round($childMb, 1)
        TotalMB    = [math]::Round($hostMb + $childMb, 1)
        IdleCPU    = $cpuPercent
        Processes  = $treeIds.Count
        Breakdown  = ($breakdown -join ", ")
    }
}

$results = @()
for ($i = 1; $i -le $Runs; $i++) {
    Write-Host ("Run {0}/{1} - settling {2}s ..." -f $i, $Runs, $SettleSeconds)
    Start-Sleep -Seconds $SettleSeconds
    $results += Measure-Once
}

function Median-Of($values) {
    $sorted = $values | Sort-Object
    return $sorted[[int][math]::Floor($sorted.Count / 2)]
}

$hostMedian = Median-Of ($results | ForEach-Object { $_.HostMB })
$totalMedian = Median-Of ($results | ForEach-Object { $_.TotalMB })
$cpuMedian = Median-Of ($results | ForEach-Object { $_.IdleCPU })

Write-Host ""
Write-Host "=== dock-tauri performance ==="
$results | ForEach-Object {
    Write-Host ("host {0} MB | children {1} MB | total {2} MB | idle CPU {3}% | {4} processes" -f `
        $_.HostMB, $_.ChildrenMB, $_.TotalMB, $_.IdleCPU, $_.Processes)
    if ($_.Breakdown) { Write-Host ("   " + $_.Breakdown) }
}

Write-Host ""
Write-Host ("Median host:      {0} MB  (gate: < 30 MB)   {1}" -f $hostMedian, $(if ($hostMedian -lt 30) { "PASS" } else { "FAIL" }))
Write-Host ("Median total:     {0} MB  (gate: < 200 MB)  {1}" -f $totalMedian, $(if ($totalMedian -lt 200) { "PASS" } else { "FAIL" }))
Write-Host ("Median idle CPU:  {0}%   (gate: < 1%)      {1}" -f $cpuMedian, $(if ($cpuMedian -lt 1) { "PASS" } else { "FAIL" }))
