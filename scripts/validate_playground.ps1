$ErrorActionPreference = "Stop"

$ProjectRoot = Resolve-Path "$PSScriptRoot/.."
$BinaryPath = "$ProjectRoot/target/debug/woven.exe"
$PlaygroundDir = "$ProjectRoot/playground"
$ReportsDir = "$ProjectRoot/reports"
$ReportFile = "$ReportsDir/playground_report.html"

Write-Host "🐍 WovenSnake Playground Automation" -ForegroundColor Cyan
Write-Host "-----------------------------------" -ForegroundColor DarkGray

if (-not (Test-Path $BinaryPath)) {
    Write-Error "Binary not found at $BinaryPath. Run 'cargo build' first."
}

# Ensure reports dir exists
if (-not (Test-Path $ReportsDir)) {
    New-Item -ItemType Directory -Path $ReportsDir | Out-Null
}

# 1. Cleanup
Write-Host "`n[1/6] Cleaning up previous playground..." -ForegroundColor Yellow
if (Test-Path $PlaygroundDir) {
    Remove-Item -Recurse -Force $PlaygroundDir
}
New-Item -ItemType Directory -Path $PlaygroundDir | Out-Null
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
        $Output = & $Command @ArgList 2>&1
        $Duration = (Get-Date) - $Start
        Write-Host " DONE ($($Duration.TotalSeconds.ToString("N2"))s)" -ForegroundColor Green
        return @{ Success = $true; Output = $Output; Duration = $Duration }
    } catch {
        Write-Host " FAILED" -ForegroundColor Red
        Write-Error $_
        return @{ Success = $false; Output = $_; Duration = (Get-Date) - $Start }
    }
}

function Add-Result {
    param($Action, $Result, $Duration, $OutputContent)
    
    $StatusClass = if($Result.Success -and ($null -eq $Result.Check -or $Result.Check)) { "pass" } else { "fail" }
    $StatusText = if($StatusClass -eq "pass") { "✅ PASS" } else { "❌ FAIL" }
    $DurationText = $Duration.TotalSeconds.ToString("N2") + "s"

    # Add Table Row
    $script:TableRows += "<tr><td>$script:StepCounter</td><td>$Action</td><td><span class='$StatusClass'>$StatusText</span></td><td>$DurationText</td></tr>`n"
    
    # Add Details
    # Clean ANSI codes from output for HTML display
    $CleanOutput = ($OutputContent | Out-String) -replace "\x1b\[[0-9;]*m", ""
    
    $script:DetailsSections += @"
    <details>
        <summary>
            <span>$script:StepCounter. $Action</span>
            <span class='$StatusClass'>$StatusText</span>
        </summary>
        <div class="output-content">
            <pre>$CleanOutput</pre>
        </div>
    </details>
"@
    $script:StepCounter++
}

# 2. Init
$ResInit = Run-Step "Initialize Project" $BinaryPath @("init")
Add-Result "Init Project" $ResInit $ResInit.Duration $ResInit.Output

# 3. Configure Dependencies
Write-Host "  Configuring wovenpkg.json..." -ForegroundColor Gray
$ConfigContent = @{
    name = "playground_automated"
    version = "0.1.0"
    python_version = "3.10"
    dependencies = @{
        requests = "==2.31.0"
        flask = "==3.0.0"
    }
    virtualEnvironment = "venv"
} | ConvertTo-Json
Set-Content -Path "wovenpkg.json" -Value $ConfigContent

# 4. Install
$ResInstall = Run-Step "Install Dependencies" $BinaryPath @("install")
Add-Result "Install Packages" $ResInstall $ResInstall.Duration $ResInstall.Output

# 5. Verify Installation (Run Python)
$PyScript = @"
import requests
import flask
print(f'SUCCESS: Requests {requests.__version__}, Flask {flask.__version__}')
"@
Set-Content "test_app.py" $PyScript
$ResRun = Run-Step "Run Python Script" $BinaryPath @("run", "python", "test_app.py")
$Check = $ResRun.Success -and ($ResRun.Output -match 'SUCCESS')
$ResRun.Check = $Check 
Add-Result "Run Script (wovensnake run)" $ResRun $ResRun.Duration $ResRun.Output

# 6. Remove
$ResRemove = Run-Step "Remove Package (Flask)" $BinaryPath @("remove", "flask")
Add-Result "Remove Package" $ResRemove $ResRemove.Duration $ResRemove.Output

# 7. List
$ResList = Run-Step "List Packages" $BinaryPath @("list")
$ListOutput = $ResList.Output | Out-String
$Cleaned = $ListOutput -notmatch "Flask"
$ResList.Check = $ResList.Success -and $Cleaned
Add-Result "Verify Pruning" $ResList $ResList.Duration $ResList.Output

# Generate HTML
$HtmlContent = @"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>WovenSnake Usability Report</title>
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
    <h1>🐍 WovenSnake Usability Report</h1>
    
    <div class="summary">
        <p><strong>Date:</strong> $(Get-Date)</p>
        <p><strong>Version:</strong> v0.1.0</p>
        <p><strong>Environment:</strong> Windows (PowerShell)</p>
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
Write-Host "`n[6/6] Destroying playground..." -ForegroundColor Yellow
Set-Location $ProjectRoot
if (Test-Path $PlaygroundDir) {
    Remove-Item -Recurse -Force $PlaygroundDir
    Write-Host "Playground destroyed successfully. 🧹" -ForegroundColor Green
}
