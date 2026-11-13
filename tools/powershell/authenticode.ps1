# Dokime — SPDX-License-Identifier: Apache-2.0
param(
    [ValidateSet('Select', 'Sign', 'Verify')][string]$Operation = 'Sign',
    [string]$Path
)
$ErrorActionPreference = 'Stop'

# Select Explicit Credentials
$azureNames = @('AZURE_TENANT_ID', 'AZURE_CLIENT_ID', 'AZURE_CLIENT_SECRET',
    'DOKIME_SIGNING_ENDPOINT', 'DOKIME_SIGNING_ACCOUNT', 'DOKIME_SIGNING_PROFILE')
$azureValues = @($azureNames | ForEach-Object { [Environment]::GetEnvironmentVariable($_) })
$azureCount = @($azureValues | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }).Count
$pfx = -not [string]::IsNullOrWhiteSpace($env:DOKIME_SIGNING_PFX)
$password = -not [string]::IsNullOrEmpty($env:DOKIME_SIGNING_PFX_PASSWORD)
if (($azureCount -gt 0) -and ($pfx -or $password)) { throw 'Configure Azure or PFX signing, not both.' }
if (($azureCount -gt 0) -and ($azureCount -ne $azureNames.Count)) { throw 'Azure signing configuration is incomplete.' }
if ($pfx -ne $password) { throw 'PFX signing requires both the base64 PFX and its password.' }
$provider = if ($azureCount -eq $azureNames.Count) { 'azure' } elseif ($pfx) { 'pfx' } else { 'none' }
if ($Operation -eq 'Select') { Write-Output $provider; exit 0 }
if ($env:DOKIME_AUTHENTICODE_PROVIDER -and $env:DOKIME_AUTHENTICODE_PROVIDER -ne $provider) {
    throw 'Signing credentials do not match the selected provider.'
}
if ($provider -eq 'none') {
    Write-Host '::notice::Authenticode skipped: no signing secrets configured.'
    exit 0
}
$file = (Get-Item -LiteralPath $Path -ErrorAction Stop).FullName
$timestamp = $env:DOKIME_SIGNING_TIMESTAMP
if ([string]::IsNullOrWhiteSpace($timestamp)) { $timestamp = 'http://timestamp.acs.microsoft.com' }
$uri = [uri]$timestamp
if (-not $uri.IsAbsoluteUri -or $uri.Scheme -notin @('http', 'https')) { throw 'An HTTP(S) RFC 3161 timestamp URL is required.' }

# Retain Already Verified Product Bytes Before Bundling
if ($Operation -eq 'Sign') {
    $existing = Get-AuthenticodeSignature -LiteralPath $file
    if ($existing.Status -eq 'Valid' -and $null -ne $existing.TimeStamperCertificate) { exit 0 }
}

# Sign with the Official Action's Module or SignTool
if ($Operation -eq 'Sign' -and $provider -eq 'azure') {
    Import-Module ArtifactSigning -RequiredVersion 0.1.20 -ErrorAction Stop
    Invoke-ArtifactSigning -Endpoint $env:DOKIME_SIGNING_ENDPOINT `
        -CodeSigningAccountName $env:DOKIME_SIGNING_ACCOUNT -CertificateProfileName $env:DOKIME_SIGNING_PROFILE `
        -Files $file -FileDigest SHA256 -TimestampRfc3161 $timestamp -TimestampDigest SHA256 `
        -ExcludeEnvironmentCredential $false -ExcludeWorkloadIdentityCredential $true `
        -ExcludeManagedIdentityCredential $true -ExcludeSharedTokenCacheCredential $true `
        -ExcludeVisualStudioCredential $true -ExcludeVisualStudioCodeCredential $true `
        -ExcludeAzureCliCredential $true -ExcludeAzurePowerShellCredential $true `
        -ExcludeAzureDeveloperCliCredential $true -ExcludeInteractiveBrowserCredential $true
}
if ($Operation -eq 'Sign' -and $provider -eq 'pfx') {
    $tool = Get-Command signtool.exe -ErrorAction SilentlyContinue
    if ($tool) { $signTool = $tool.Source } else {
        $kits = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits/10/bin'
        $signTool = Get-ChildItem -Path "$kits/*/x64/signtool.exe" | Sort-Object FullName -Descending |
            Select-Object -First 1 -ExpandProperty FullName
    }
    if (-not $signTool) { throw 'SignTool from the Windows SDK is required.' }
    $key = Join-Path ([IO.Path]::GetTempPath()) ([IO.Path]::GetRandomFileName() + '.pfx')
    try {
        [IO.File]::WriteAllBytes($key, [Convert]::FromBase64String($env:DOKIME_SIGNING_PFX))
        & $signTool sign /f $key /p $env:DOKIME_SIGNING_PFX_PASSWORD /fd SHA256 /tr $timestamp /td SHA256 $file
        if ($LASTEXITCODE -ne 0) { throw "SignTool failed with exit code $LASTEXITCODE." }
    } finally {
        if (Test-Path -LiteralPath $key) { Remove-Item -LiteralPath $key -Force }
    }
}

# Verify the Final Bytes and Timestamp
$signature = Get-AuthenticodeSignature -LiteralPath $file
if ($signature.Status -ne 'Valid' -or $null -eq $signature.TimeStamperCertificate) {
    throw "Authenticode verification failed for $file (status: $($signature.Status); RFC 3161 timestamp required)."
}
exit 0

