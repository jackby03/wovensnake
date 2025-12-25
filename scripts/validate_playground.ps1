$ErrorActionPreference = "Stop"

$ProjectRoot = Resolve-Path "$PSScriptRoot/.."
$BinaryPath = "$ProjectRoot/target/debug/woven.exe"
$PlaygroundDir = "$ProjectRoot/playground"
$ReportsDir = "$ProjectRoot/reports"
$ReportFile = "$ReportsDir/playground_report.html"

Write-Host "🐍 WovenSnake Playground Automation (v0.2.0)" -ForegroundColor Cyan
Write-Host "-------------------------------------------" -ForegroundColor DarkGray

if (-not (Test-Path $BinaryPath)) {
    Write-Error "Binary not found at $BinaryPath. Run 'cargo build' first."
}

# Ensure reports dir exists
if (-not (Test-Path $ReportsDir)) {
    New-Item -ItemType Directory -Path $ReportsDir | Out-Null
}

# 1. Cleanup
Write-Host "`n[1/12] Cleaning up previous playground..." -ForegroundColor Yellow
Set-Location $ProjectRoot
$PlaygroundDir = "$ProjectRoot/playground_$(Get-Random)"
New-Item -ItemType Directory -Path $PlaygroundDir -Force | Out-Null
Set-Location $PlaygroundDir

# Data Collectors
$script:TableRows = ""
$script:DetailsSections = ""
$script:StepCounter = 1

# Helper to run step
function Run-Step {
    param($Name, $Command, $ArgList)
    $Start = Get-Date
    try {
        Write-Host "  Running: $Name..." -NoNewline
        
        # Set RUST_BACKTRACE=1 for detailed error logs
        $env:RUST_BACKTRACE = "1"
        $env:RUST_LOG = "debug"
        
        # Capture both stdout and stderr
        $ProcessInfo = New-Object System.Diagnostics.ProcessStartInfo
        $ProcessInfo.FileName = $Command
        $ProcessInfo.Arguments = $ArgList -join " "
        $ProcessInfo.RedirectStandardOutput = $true
        $ProcessInfo.RedirectStandardError = $true
        $ProcessInfo.UseShellExecute = $false
        $ProcessInfo.CreateNoWindow = $true
        $ProcessInfo.WorkingDirectory = $PWD.Path
        
        $Process = [System.Diagnostics.Process]::Start($ProcessInfo)
        $Stdout = $Process.StandardOutput.ReadToEnd()
        $Stderr = $Process.StandardError.ReadToEnd()
        $Process.WaitForExit()
        
        $ExitCode = $Process.ExitCode
        $Output = $Stdout + "`n" + $Stderr
        
        $Duration = (Get-Date) - $Start
        if ($ExitCode -eq 0) {
            Write-Host " DONE ($($Duration.TotalSeconds.ToString("N2"))s)" -ForegroundColor Green
        } else {
            Write-Host " FAILED (Exit: $ExitCode) ($($Duration.TotalSeconds.ToString("N2"))s)" -ForegroundColor Red
            Write-Host "    Error details captured in report." -ForegroundColor Gray
        }
        return @{ Success = ($ExitCode -eq 0); Output = $Output; Duration = $Duration; ExitCode = $ExitCode }
    } catch {
        Write-Host " CRITICAL ERROR" -ForegroundColor Red
        return @{ Success = $false; Output = $_.Exception.Message; Duration = (Get-Date) - $Start; ExitCode = -1 }
    }
}

function Add-Result {
    param($Action, $Result, $Duration, $OutputContent)
    
    # A step only passes when the command succeeded and any optional check didn’t fail.
    $CheckPassed = if ($null -eq $Result.Check) { $true } else { $Result.Check }
    $StatusClass = if ($Result.Success -and $CheckPassed) { "pass" } else { "fail" }
    $StatusText = if($StatusClass -eq "pass") { "PASS" } else { "FAIL" }
    $DurationText = $Duration.TotalSeconds.ToString("N2") + "s"

    # Add Table Row
    $script:TableRows += "<tr><td>$script:StepCounter</td><td>$Action</td><td><span class='$StatusClass'>$StatusText</span></td><td>$DurationText</td></tr>`n"
    
    # Add Details
    # Clean ANSI codes from output for HTML display
    $CleanOutput = ($OutputContent | Out-String) -replace "\x1b\[[0-9;]*m", ""
    
    # If it failed, highlight the output
    $PreClass = if($StatusClass -eq "fail") { "output-error" } else { "" }
    
    $script:DetailsSections += @"
    <details>
        <summary>
            <span>$script:StepCounter. $Action</span>
            <span class='$StatusClass'>$StatusText</span>
        </summary>
        <div class="output-content">
            <pre class="$PreClass">$CleanOutput</pre>
        </div>
    </details>
"@
    $script:StepCounter++
}

# 2. Init (Auto-detection)
$ResInit = Run-Step "Initialize Project (Auto-detect Python)" $BinaryPath @("init")

# Check if file exists before reading
if (-not (Test-Path "wovenpkg.json")) {
    Write-Host "  CRITICAL: wovenpkg.json was not created!" -ForegroundColor Red
    Write-Host "  Output: $($ResInit.Output)" -ForegroundColor Gray
    exit 1
}

$Config = Get-Content "wovenpkg.json" | ConvertFrom-Json
$SystemPython = (python --version) -replace "Python ", ""
$SystemMajorMinor = $SystemPython.Split(".")[0] + "." + $SystemPython.Split(".")[1]
$ResInit.Check = $Config.python_version -eq $SystemMajorMinor
Add-Result "Init (Auto-detect Python)" $ResInit $ResInit.Duration $ResInit.Output

# 3. Configure Dependencies
Write-Host "  Configuring wovenpkg.json..." -ForegroundColor Gray
$Config.dependencies = @{
    requests = "==2.31.0"
}
$Config | ConvertTo-Json | Set-Content -Path "wovenpkg.json"

# 4. Install
$ResInstall = Run-Step "Install Dependencies" $BinaryPath @("install")
Add-Result "Install Packages" $ResInstall $ResInstall.Duration $ResInstall.Output

# 5. Verify Lockfile Python Version
$Lockfile = Get-Content "wovenpkg.lock" | ConvertFrom-Json
$ResLock = @{ Success = $true; Check = ($null -ne $Lockfile.python_version -and $Lockfile.python_version -eq $Config.python_version); Output = "Lockfile Python Version: $($Lockfile.python_version)"; Duration = [TimeSpan]::Zero }
Add-Result "Verify Lockfile Python" $ResLock $ResLock.Duration $ResLock.Output

# 6. Verify Venv Version Warning
Write-Host "  Simulating Python version mismatch..." -ForegroundColor Gray
$Config.python_version = "3.11" # Force a different version
$Config | ConvertTo-Json | Set-Content -Path "wovenpkg.json"
$ResWarning = Run-Step "Install with Version Mismatch" $BinaryPath @("install")
$ResWarning.Check = $ResWarning.Output -match "Existing virtual environment uses Python"
Add-Result "Venv Version Warning" $ResWarning $ResWarning.Duration $ResWarning.Output

# 7. List (Managed Versions)
$ResList = Run-Step "List Managed Pythons" $BinaryPath @("list")
Add-Result "List Managed Pythons" $ResList $ResList.Duration $ResList.Output

# 8. Run Script
$PyScript = "import requests; print(f'SUCCESS: Requests {requests.__version__}')"
Set-Content "test_app.py" $PyScript
$ResRun = Run-Step "Run Script" $BinaryPath @("run", "python", "test_app.py")
$ResRun.Check = $ResRun.Output -match "SUCCESS"
Add-Result "Run Script" $ResRun $ResRun.Duration $ResRun.Output

# 9. Clean (Standard)
$ResClean = Run-Step "Clean Project" $BinaryPath @("clean")
Add-Result "Clean Project" $ResClean $ResClean.Duration $ResClean.Output

# 10. Clean (Python)
$ResCleanPy = Run-Step "Clean Managed Pythons" $BinaryPath @("clean", "--python")
Add-Result "Clean Managed Pythons" $ResCleanPy $ResCleanPy.Duration $ResCleanPy.Output

# Generate HTML
$HtmlContent = @"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>WovenSnake v0.2.0 Usability Report</title>
    <style>
        body { font-family: 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; max-width: 900px; margin: 0 auto; padding: 40px 20px; background-color: #f8f9fa; color: #212529; line-height: 1.6; }
        h1 { color: #2c3e50; border-bottom: 3px solid #3498db; padding-bottom: 15px; margin-bottom: 30px; letter-spacing: -0.5px; }
        h2 { color: #34495e; margin-top: 40px; }
        .summary { background: white; padding: 25px; border-radius: 12px; box-shadow: 0 4px 6px rgba(0,0,0,0.05); margin-bottom: 30px; border-left: 5px solid #3498db; }
        table { width: 100%; border-collapse: separate; border-spacing: 0; margin: 25px 0; background: white; border-radius: 12px; overflow: hidden; box-shadow: 0 4px 6px rgba(0,0,0,0.05); }
        th, td { padding: 15px 20px; text-align: left; border-bottom: 1px solid #eee; }
        th { background-color: #34495e; color: white; font-weight: 600; text-transform: uppercase; font-size: 0.85em; letter-spacing: 1px; }
        tr:last-child td { border-bottom: none; }
        tr:hover td { background-color: #fdfdfd; }
        .pass { color: #27ae60; font-weight: 700; background-color: rgba(39, 174, 96, 0.1); padding: 5px 10px; border-radius: 20px; display: inline-block; font-size: 0.9em; }
        .fail { color: #e74c3c; font-weight: 700; background-color: rgba(231, 76, 60, 0.1); padding: 5px 10px; border-radius: 20px; display: inline-block; font-size: 0.9em; }
        pre { background: #2d3436; color: #dfe6e9; padding: 20px; border-radius: 8px; overflow-x: auto; font-family: 'Consolas', 'Monaco', monospace; font-size: 0.9em; white-space: pre-wrap; word-wrap: break-word; }
        .output-error { border: 2px solid #e74c3c; background: #2c1a1a; }
        details { background: white; margin-bottom: 15px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.05); border: 1px solid #eee; }
        summary { padding: 15px 20px; cursor: pointer; font-weight: 600; outline: none; list-style: none; display: flex; align-items: center; justify-content: space-between; transition: background-color 0.2s; border-radius: 8px; }
        summary:hover { background-color: #f8f9fa; }
        summary::-webkit-details-marker { display: none; }
        summary::after { content: '+'; font-size: 1.2em; color: #bdc3c7; }
        details[open] summary { border-radius: 8px 8px 0 0; border-bottom: 1px solid #eee; }
        details[open] summary::after { content: '-'; }
        .output-content { padding: 0 20px 20px; animation: fadeIn 0.3s ease-in-out; }
        @keyframes fadeIn { from { opacity: 0; transform: translateY(-10px); } to { opacity: 1; transform: translateY(0); } }
        .footer { text-align: center; margin-top: 50px; color: #95a5a6; font-size: 0.9em; }
    </style>
</head>
<body>
    <h1>WovenSnake v0.2.0 Usability Report</h1>
    
    <div class="summary">
        <p><strong>Date:</strong> $(Get-Date)</p>
        <p><strong>Version:</strong> v0.2.0</p>
        <p><strong>Environment:</strong> Windows (PowerShell)</p>
        <p><strong>Features Tested:</strong> Python Auto-detection, Lockfile Versioning, Venv Validation, Managed Python Management.</p>
    </div>

    <h2>Test Execution Log</h2>
    <table>
        <thead>
            <tr>
                <th>Step</th>
                <th>Action</th>
                <th>Status</th>
                <th>Duration</th>
            </tr>
        </thead>
        <tbody>
            $script:TableRows
        </tbody>
    </table>

    <h2>Detailed Outputs</h2>
    $script:DetailsSections

    <div class="footer">
        Generated automatically by WovenSnake Validation Script
    </div>
</body>
</html>
"@

Set-Content -Path $ReportFile -Value $HtmlContent
Write-Host "`nReport generated at $ReportFile" -ForegroundColor Cyan

# Final Cleanup
Write-Host "`n[12/12] Destroying playground..." -ForegroundColor Yellow
Set-Location $ProjectRoot
if (Test-Path $PlaygroundDir) {
    Remove-Item -Recurse -Force $PlaygroundDir -ErrorAction SilentlyContinue
    Write-Host "Playground destroyed successfully." -ForegroundColor Green
}
