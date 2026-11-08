# Builds BeatCraft.exe and packs it as dist\BeatCraft-<version>-windows-x64.zip
# Needs Rust (https://rustup.rs) with the default MSVC toolchain.
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")

$name = "BeatCraft"
$version = (Select-String -Path Cargo.toml -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches[0].Groups[1].Value
$out = "dist\$name-$version-windows-x64"

cargo build --release
if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }

if (Test-Path $out) { Remove-Item -Recurse -Force $out }
New-Item -ItemType Directory -Force -Path $out | Out-Null
Copy-Item "target\release\$name.exe" "$out\$name.exe"
Copy-Item "target\release\beatcraft-cli.exe" "$out\beatcraft-cli.exe"
Copy-Item "packaging\README.txt" "$out\README.txt"
Copy-Item LICENSE-MIT, LICENSE-APACHE, NOTICE $out

$zip = "$out.zip"
if (Test-Path $zip) { Remove-Item -Force $zip }
Compress-Archive -Path "$out\*" -DestinationPath $zip
$hash = (Get-FileHash $zip -Algorithm SHA256).Hash.ToLower()
"$hash  $(Split-Path $zip -Leaf)" | Out-File -Encoding ascii "dist\SHA256SUMS-windows.txt"
Write-Host "Done: $zip"
