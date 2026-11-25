# Pinned CI helpers. Run from the repository root on Windows x64.
$ErrorActionPreference = 'Stop'
function Assert-NativeSuccess([string] $Operation) {
    if ($LASTEXITCODE -ne 0) { throw "$Operation failed with exit code $LASTEXITCODE" }
}

rustup toolchain install 1.90.0-x86_64-pc-windows-msvc --profile minimal --component rustfmt,clippy,llvm-tools-preview --target x86_64-pc-windows-msvc
Assert-NativeSuccess 'Rust toolchain installation'
cargo install cargo-llvm-cov --version 0.6.21 --locked
Assert-NativeSuccess 'cargo-llvm-cov installation'
# 0.18.3 cannot parse the current advisory DB's CVSS 4.0 entries.
# 0.21.0 declares Rust 1.89; newer 0.22.x requires 1.95, beyond this baseline.
cargo install cargo-deny --version 0.21.0 --locked
Assert-NativeSuccess 'cargo-deny installation'

# Verify the downloaded archive against the versioned upstream checksum file.
$version = '8.24.3'
$asset = "gitleaks_${version}_windows_x64.zip"
$destination = Join-Path $env:RUNNER_TEMP 'drape-gitleaks'
New-Item -ItemType Directory -Force $destination | Out-Null
$base = "https://github.com/gitleaks/gitleaks/releases/download/v$version"
Invoke-WebRequest "$base/$asset" -OutFile (Join-Path $destination $asset)
Invoke-WebRequest "$base/gitleaks_${version}_checksums.txt" -OutFile (Join-Path $destination 'checksums.txt')
$line = Get-Content (Join-Path $destination 'checksums.txt') |
    Where-Object { $_ -match "\s+$([regex]::Escape($asset))$" }
if (@($line).Count -ne 1) { throw 'Expected exactly one Gitleaks archive checksum' }
$expected = ($line -split '\s+')[0]
$actual = (Get-FileHash (Join-Path $destination $asset) -Algorithm SHA256).Hash
if ($actual -ne $expected) { throw 'Gitleaks archive checksum mismatch' }
Expand-Archive (Join-Path $destination $asset) -DestinationPath $destination -Force
$destination | Out-File $env:GITHUB_PATH -Encoding utf8 -Append
