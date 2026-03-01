# jarvis login — refreshes Claude Code OAuth token and updates .env
$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$EnvFile = Join-Path $ScriptDir ".env"

Write-Host "Logging into Claude Code..."
$env:CLAUDECODE = $null
& claude auth login

# Pull new token from ~/.claude/.credentials.json (written by claude auth login)
$credPath = Join-Path (Join-Path $env:USERPROFILE ".claude") ".credentials.json"
if (Test-Path $credPath) {
    $json = Get-Content $credPath -Raw | ConvertFrom-Json
    $OAuthToken = $json.claudeAiOauth.accessToken
}

if (-not $OAuthToken) {
    Write-Error "Could not extract OAuth token from credentials file"
    exit 1
}

# Update .env file
if ((Test-Path $EnvFile) -and (Select-String -Path $EnvFile -Pattern "CLAUDE_CODE_OAUTH_TOKEN" -Quiet)) {
    (Get-Content $EnvFile) -replace "^CLAUDE_CODE_OAUTH_TOKEN=.*", "CLAUDE_CODE_OAUTH_TOKEN=$OAuthToken" |
        Set-Content $EnvFile
} else {
    Add-Content -Path $EnvFile -Value "CLAUDE_CODE_OAUTH_TOKEN=$OAuthToken"
}

Write-Host "Token updated in .env ($($OAuthToken.Substring(0,15))...)"
