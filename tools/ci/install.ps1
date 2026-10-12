# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# Prepare Native Windows CI Tools.

# === Imports === #

# === Constants === #

# === Functions === #

function Invoke-Mise {
    & mise @args
    if ($LASTEXITCODE -ne 0) { throw "Mise failed: $args" }
}
# === Entry Point === #
$ErrorActionPreference = 'Stop'
. ./tools/ci/install-mise.ps1
if ($env:GITHUB_PATH) {
    (Join-Path $PWD '.cache/bootstrap/mise/bin') | Out-File -FilePath $env:GITHUB_PATH -Encoding utf8 -Append
}
Invoke-Mise trust mise.toml
Invoke-Mise install --locked python uv
Invoke-Mise run --skip-tools ci:bootstrap
# Initialize MSVC Before Cargo-Based Mise Tools Compile.
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
if (!(Test-Path $vswhere)) { throw 'Visual Studio discovery tool is missing' }
$installation = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if ($LASTEXITCODE -ne 0 -or !$installation) { throw 'Visual Studio C++ toolset is missing' }
Import-Module (Join-Path $installation 'Common7/Tools/Microsoft.VisualStudio.DevShell.dll')
Enter-VsDevShell -VsInstallPath $installation -SkipAutomaticLocation -DevCmdArguments '-arch=x64 -host_arch=x64'
if ($env:GITHUB_ENV) {
    foreach ($name in @('Path', 'INCLUDE', 'LIB', 'LIBPATH', 'VSINSTALLDIR', 'VCToolsInstallDir', 'WindowsSdkDir', 'WindowsSDKVersion')) {
        $value = [Environment]::GetEnvironmentVariable($name)
        if ($value -and $value -notmatch '[\r\n]') {
            "$name=$value" | Out-File -FilePath $env:GITHUB_ENV -Encoding utf8 -Append
        }
    }
}
Invoke-Mise install --locked
& git diff --exit-code -- mise.lock
if ($LASTEXITCODE -ne 0) { throw "Mise changed the committed lockfile" }
Invoke-Mise exec -- rustc --version
Invoke-Mise exec -- cargo --version
Invoke-Mise exec -- python --version
Invoke-Mise exec -- uv --version
Invoke-Mise exec -- node --version
Invoke-Mise exec -- pnpm --version

Invoke-Mise run sync
