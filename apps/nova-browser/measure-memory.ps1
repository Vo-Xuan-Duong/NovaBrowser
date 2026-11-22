# Windows PowerShell 5.1+: sample the complete process tree, not only
# nova-servo-preview.exe. This is essential for a fair comparison with
# WebView2/Chromium subprocesses.
param(
    [string]$ProcessName = "nova-servo-preview",
    [ValidateRange(1, 1000)][int]$Samples = 10,
    [ValidateRange(1, 60)][int]$IntervalSeconds = 2,
    [string]$CsvPath = ""
)
$ErrorActionPreference = "Stop"
$rows = @()
for ($n = 1; $n -le $Samples; $n++) {
    $list = @(Get-CimInstance Win32_Process)
    $roots = @($list | Where-Object { $_.Name -ieq ($ProcessName + ".exe") })
    if ($roots.Count -eq 0) {
        Write-Warning "Cannot find $ProcessName.exe; run the application first."
        break
    }

    # Include every descendant of the matched executable, across all roots.
    $selected = New-Object 'System.Collections.Generic.HashSet[int]'
    foreach ($root in $roots) { [void]$selected.Add([int]$root.ProcessId) }
    $changed = $true
    while ($changed) {
        $changed = $false
        foreach ($process in $list) {
            if ($selected.Contains([int]$process.ParentProcessId)) {
                if ($selected.Add([int]$process.ProcessId)) { $changed = $true }
            }
        }
    }

    $private = [int64]0
    $working = [int64]0
    $count = 0
    foreach ($pidValue in $selected) {
        try {
            $process = Get-Process -Id $pidValue -ErrorAction Stop
            $private += [int64]$process.PrivateMemorySize64
            $working += [int64]$process.WorkingSet64
            $count++
        } catch {
            # A process may exit during the sample.
        }
    }
    $row = [pscustomobject]@{
        At = (Get-Date).ToString("o")
        Process = $ProcessName
        RootProcesses = $roots.Count
        TreeProcesses = $count
        PrivateMB = [math]::Round($private / 1MB, 1)
        WorkingSetMB = [math]::Round($working / 1MB, 1)
    }
    $rows += $row
    $row | Format-Table -AutoSize | Out-String | Write-Host
    if ($n -lt $Samples) { Start-Sleep -Seconds $IntervalSeconds }
}
if ($CsvPath -and $rows.Count -gt 0) {
    $rows | Export-Csv -Path $CsvPath -NoTypeInformation -Encoding UTF8
    Write-Host "Saved: $CsvPath"
}
