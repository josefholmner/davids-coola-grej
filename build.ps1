param(
    [string]$Cargo = 'cargo',
    [switch]$SkipTests
)
$ErrorActionPreference = 'Stop'
Push-Location $PSScriptRoot
try {
    $cargoCommand = (Get-Command $Cargo -ErrorAction Stop).Source
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
    if (Test-Path -LiteralPath $vswhere) {
        $visualStudio = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
        if ($visualStudio) {
            $compiler = Get-ChildItem -LiteralPath (Join-Path $visualStudio 'VC\Tools\MSVC') -Directory | Sort-Object Name -Descending | Select-Object -First 1
            $sdkBase = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10\Lib'
            $sdk = Get-ChildItem -LiteralPath $sdkBase -Directory | Where-Object Name -Match '^\d+\.' | Sort-Object Name -Descending | Select-Object -First 1
            $env:PATH = (Join-Path $compiler.FullName 'bin\Hostx64\x64') + ';' + $env:PATH
            $env:LIB = @((Join-Path $compiler.FullName 'lib\x64'), (Join-Path $sdk.FullName 'um\x64'), (Join-Path $sdk.FullName 'ucrt\x64'), $env:LIB) -join ';'
        }
    }
    if (!$SkipTests) {
        & $cargoCommand test
        if ($LASTEXITCODE -ne 0) { throw 'Rust tests failed.' }
    }
    & $cargoCommand build --release
    if ($LASTEXITCODE -ne 0) { throw 'Release build failed.' }
    $distDirectory = Join-Path $PSScriptRoot 'dist'
    New-Item -ItemType Directory -Path $distDirectory -Force | Out-Null
    Copy-Item -LiteralPath 'target\release\NumberCatch.exe' -Destination (Join-Path $distDirectory 'NumberCatch.exe') -Force
    Copy-Item -LiteralPath 'packaging\README.txt' -Destination (Join-Path $distDirectory 'README.txt') -Force
    Write-Output "Ready to play, zip, or check in: $distDirectory"
} finally {
    Pop-Location
}
