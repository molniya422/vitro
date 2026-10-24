$p = Get-Process vitro
$p.Refresh()
$c1 = $p.TotalProcessorTime.TotalSeconds
Start-Sleep -Milliseconds 8000
$p.Refresh()
$c2 = $p.TotalProcessorTime.TotalSeconds
'CPU {0:P2}  RAM {1:N2}MB' -f (($c2 - $c1) / 8), ($p.WorkingSet64 / 1MB)
