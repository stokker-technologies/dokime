# Sample Physical Disk Counters Beside A Test Lane To Attribute Hosted I/O Stalls To A Volume.
param(
    [Parameter(Mandatory)][ValidateSet('start', 'sample', 'stop')][string]$Mode,
    [Parameter(Mandatory)][string]$Output
)
$ErrorActionPreference = 'Stop'
$Samples = Join-Path $Output 'counters.csv'
$Sampler = Join-Path $Output 'sampler.pid'
$Counters = @(
    '\PhysicalDisk(*)\Avg. Disk sec/Transfer',
    '\PhysicalDisk(*)\Current Disk Queue Length',
    '\PhysicalDisk(*)\Disk Bytes/sec'
)
# A Transfer Slower Than One Second Is A Stall, Not Ordinary Contention.
$StallSeconds = 1.0

switch ($Mode) {
    'start' {
        New-Item -ItemType Directory -Force $Output | Out-Null
        $defender = try {
            Get-MpComputerStatus | Select-Object RealTimeProtectionEnabled, OnAccessProtectionEnabled, IoavProtectionEnabled, BehaviorMonitorEnabled
        } catch {
            "unavailable: $($_.Exception.Message)"
        }
        $temporary = [System.IO.Path]::GetTempPath()
        [ordered]@{
            Temporary = $temporary
            Workspace = $env:GITHUB_WORKSPACE
            Defender  = $defender
            Volumes   = @(Get-Volume | Where-Object DriveLetter | Select-Object DriveLetter, FileSystemLabel, Size, SizeRemaining)
            Disks     = @(Get-PhysicalDisk | Select-Object DeviceId, FriendlyName, MediaType, BusType, Size)
        } | ConvertTo-Json -Depth 4 | Set-Content (Join-Path $Output 'environment.json')
        Write-Output "Temporary Directory: $temporary"
        Write-Output "Defender: $($defender | ConvertTo-Json -Compress)"
        'Timestamp,Instance,Counter,Value' | Set-Content $Samples
        $process = Start-Process pwsh -PassThru -WindowStyle Hidden -ArgumentList @(
            '-NoProfile', '-NonInteractive', '-File', $PSCommandPath, '-Mode', 'sample', '-Output', $Output
        )
        $process.Id | Set-Content $Sampler
    }
    'sample' {
        Get-Counter -Counter $Counters -SampleInterval 5 -Continuous | ForEach-Object {
            $stamp = $_.Timestamp.ToUniversalTime().ToString('o')
            $_.CounterSamples | Where-Object InstanceName -NE '_total' | ForEach-Object {
                '{0},{1},{2},{3}' -f $stamp, $_.InstanceName, $_.Path.Split('\')[-1], $_.CookedValue
            } | Add-Content $Samples
        }
    }
    'stop' {
        if (Test-Path $Sampler) {
            Stop-Process -Id ([int](Get-Content $Sampler)) -ErrorAction SilentlyContinue
            Remove-Item $Sampler
        }
        if (-not (Test-Path $Samples)) {
            return
        }
        $rows = @(Import-Csv $Samples)
        $summary = @($rows | Group-Object Instance, Counter | ForEach-Object {
            $values = @($_.Group | ForEach-Object { [double]$_.Value })
            $peak = $_.Group | Sort-Object { [double]$_.Value } -Descending | Select-Object -First 1
            [ordered]@{
                Instance = $_.Group[0].Instance
                Counter  = $_.Group[0].Counter
                Samples  = $values.Count
                Mean     = ($values | Measure-Object -Average).Average
                Peak     = ($values | Measure-Object -Maximum).Maximum
                PeakAt   = $peak.Timestamp
            }
        })
        $stalls = @($rows | Where-Object { $_.Counter -eq 'avg. disk sec/transfer' -and [double]$_.Value -ge $StallSeconds })
        [ordered]@{ Summary = $summary; Stalls = $stalls } | ConvertTo-Json -Depth 4 | Set-Content (Join-Path $Output 'summary.json')
        $lines = @('### Disk Counters', '', '| Disk | Counter | Samples | Mean | Peak | Peak At |', '| --- | --- | --- | --- | --- | --- |')
        $lines += $summary | ForEach-Object { '| {0} | {1} | {2} | {3:N4} | {4:N4} | {5} |' -f $_.Instance, $_.Counter, $_.Samples, $_.Mean, $_.Peak, $_.PeakAt }
        $lines += '', "Samples With A Transfer Of At Least $StallSeconds s: $($stalls.Count)"
        $lines | Write-Output
        if ($env:GITHUB_STEP_SUMMARY) {
            $lines | Add-Content $env:GITHUB_STEP_SUMMARY
        }
    }
}
