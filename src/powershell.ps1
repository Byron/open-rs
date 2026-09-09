# Start-Process resolves wildcard patterns in parent directories. Invoke-Item
# accepts literal paths; retain Start-Process for URLs and application names.
$ErrorActionPreference = 'Stop'
if (Test-Path -LiteralPath $env:OPEN_RS_TARGET) {
    Invoke-Item -LiteralPath $env:OPEN_RS_TARGET
} else {
    Start-Process -FilePath $env:OPEN_RS_TARGET
}
