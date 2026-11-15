<#
.SYNOPSIS
  Builds the Microsoft Store package (MSIX) of Nebulance.

.DESCRIPTION
  1. cargo build --release
  2. Copies the game, the readmes, the font licenses and the Store images into target\msix\stage
  3. Fills the manifest with packaging\msix\identity.json and the version from Cargo.toml
  4. Generates resources.pri (makepri) and packs dist\Nebulance_<version>_x64.msix (makeappx)

  Needs the Windows SDK (makeappx.exe, makepri.exe, signtool.exe):
      winget install Microsoft.WindowsSDK.10.0.26100

.PARAMETER TestSign
  Also signs the package with a self-signed certificate, so it can be installed on this PC
  for testing. The Store signs the package itself: do NOT use -TestSign for the file you upload.

.PARAMETER SkipBuild
  Uses the existing target\release\nebulance.exe.

.EXAMPLE
  powershell -ExecutionPolicy Bypass -File packaging\build-msix.ps1
  powershell -ExecutionPolicy Bypass -File packaging\build-msix.ps1 -TestSign
#>
param(
    [switch]$TestSign,
    [switch]$SkipBuild
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

# --- version: Cargo.toml 1.2.3 -> MSIX 1.2.3.0 (the Store requires the last number to be 0)
$cargo = Get-Content Cargo.toml -Raw
if ($cargo -notmatch '(?m)^version\s*=\s*"(\d+)\.(\d+)\.(\d+)"') { throw 'version not found in Cargo.toml' }
$version = "$($Matches[1]).$($Matches[2]).$($Matches[3]).0"

# --- identity from Partner Center
$id = Get-Content packaging\msix\identity.json -Raw | ConvertFrom-Json
if ($id.IdentityName -like 'PREENCHA*' -or $id.Publisher -like '*PREENCHA*') {
    Write-Warning 'packaging\msix\identity.json still has placeholder values. The package works for local tests, but the Store will reject it. Fill it with the values from Partner Center > Product identity.'
}

# --- Windows SDK tools
$kits = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10\bin'
$bin = Get-ChildItem $kits -Directory -ErrorAction SilentlyContinue |
    Where-Object Name -match '^10\.' |
    Sort-Object { [version]$_.Name } -Descending |
    ForEach-Object { Join-Path $_.FullName 'x64' } |
    Where-Object { Test-Path (Join-Path $_ 'makeappx.exe') } |
    Select-Object -First 1
if (-not $bin) { throw 'Windows SDK not found. Install it with: winget install Microsoft.WindowsSDK.10.0.26100' }

# --- build
if (-not $SkipBuild) {
    cargo build --release
    if ($LASTEXITCODE) { throw 'cargo build failed' }
}
$exe = 'target\release\nebulance.exe'
if (-not (Test-Path $exe)) { throw "$exe not found" }

# --- stage
$work = Join-Path $root 'target\msix'
$stage = Join-Path $work 'stage'
if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
New-Item -ItemType Directory -Force (Join-Path $stage 'Assets'), (Join-Path $stage 'licenses') | Out-Null
Copy-Item $exe $stage
Copy-Item README.txt, LEIAME.txt $stage
Copy-Item assets\fonts\OFL-*.txt (Join-Path $stage 'licenses')
Copy-Item packaging\msix\Assets\*.png (Join-Path $stage 'Assets')

$manifest = (Get-Content packaging\msix\AppxManifest.xml -Raw).
    Replace('$(IdentityName)', $id.IdentityName).
    Replace('$(Publisher)', [System.Security.SecurityElement]::Escape($id.Publisher)).
    Replace('$(PublisherDisplayName)', [System.Security.SecurityElement]::Escape($id.PublisherDisplayName)).
    Replace('$(Version)', $version)
[System.IO.File]::WriteAllText((Join-Path $stage 'AppxManifest.xml'), $manifest, (New-Object System.Text.UTF8Encoding $false))

# --- resources.pri (lets Windows pick the right image size for each scale and theme)
$priconfig = Join-Path $work 'priconfig.xml'
& (Join-Path $bin 'makepri.exe') createconfig /cf $priconfig /dq en-US_pt-BR /pv 10.0.0 /o
if ($LASTEXITCODE) { throw 'makepri createconfig failed' }
& (Join-Path $bin 'makepri.exe') new /pr $stage /cf $priconfig /mn (Join-Path $stage 'AppxManifest.xml') /of (Join-Path $stage 'resources.pri') /o
if ($LASTEXITCODE) { throw 'makepri new failed' }

# --- pack
New-Item -ItemType Directory -Force dist | Out-Null
$out = Join-Path $root "dist\Nebulance_${version}_x64.msix"
& (Join-Path $bin 'makeappx.exe') pack /d $stage /p $out /o
if ($LASTEXITCODE) { throw 'makeappx pack failed' }
Write-Host "Package: $out" -ForegroundColor Green

# --- optional test signature
if ($TestSign) {
    $cert = Get-ChildItem Cert:\CurrentUser\My |
        Where-Object { $_.Subject -eq $id.Publisher -and $_.NotAfter -gt (Get-Date) } |
        Select-Object -First 1
    if (-not $cert) {
        $cert = New-SelfSignedCertificate -Type Custom -Subject $id.Publisher `
            -KeyUsage DigitalSignature -FriendlyName 'Nebulance test signing' `
            -CertStoreLocation Cert:\CurrentUser\My `
            -TextExtension @('2.5.29.37={text}1.3.6.1.5.5.7.3.3', '2.5.29.19={text}')
    }
    & (Join-Path $bin 'signtool.exe') sign /fd SHA256 /sha1 $cert.Thumbprint $out
    if ($LASTEXITCODE) { throw 'signtool failed' }
    $cer = Join-Path $root 'dist\nebulance-test.cer'
    Export-Certificate -Cert $cert -FilePath $cer | Out-Null
    Write-Host ''
    Write-Host 'Signed for local testing. To install it on this PC:' -ForegroundColor Yellow
    Write-Host "  1. Once, in an ADMIN PowerShell: Import-Certificate -FilePath `"$cer`" -CertStoreLocation Cert:\LocalMachine\TrustedPeople"
    Write-Host "  2. Add-AppxPackage `"$out`""
    Write-Host 'Upload the UNSIGNED package to the Store (run again without -TestSign).'
}
