# Set Working Directory
Split-Path $MyInvocation.MyCommand.Path | Push-Location
[Environment]::CurrentDirectory = $PWD

./Publish.ps1 -ProjectPath "UnrealEssentials/UnrealEssentials.csproj" `
              -PackageName "UnrealEssentials" `
              -PublishOutputDir "Publish/ToUpload/UnrealEssentials" `
			  -ChangelogPath "UnrealEssentials/CHANGELOG.MD" `
              
Remove-Item "Publish/Builds" -Recurse -ErrorAction SilentlyContinue

.\PublishInterfaces.ps1

# Create Rust project
New-Item "Publish/Builds/CurrentVersion" -ItemType Directory

Push-Location "./UtocEmulator/fileemu-utoc-stream-emulator"
$env:RUSTFLAGS = "-C panic=abort -C lto=fat -C embed-bitcode=yes"
$current_ver_folder = [Environment]::CurrentDirectory + "/Publish/Builds/CurrentVersion"
$rust_lib_out = "fileemu_utoc_stream_emulator.dll"
cargo +nightly rustc --lib --release -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --crate-type cdylib --target x86_64-pc-windows-msvc
Push-Location "../target/x86_64-pc-windows-msvc/release"
Copy-Item  $rust_lib_out -Destination $current_ver_folder
Copy-Item "$rust_lib_out.lib" -Destination $current_ver_folder
Copy-Item "$rust_lib_out.exp" -Destination $current_ver_folder
Pop-Location
Pop-Location

./Publish.ps1 -ProjectPath "UtocEmulator/UTOC.Stream.Emulator/UTOC.Stream.Emulator.csproj" `
              -PackageName "UTOC.Stream.Emulator" `
              -PublishOutputDir "Publish/ToUpload/UTOC.Stream.Emulator" `
			  -ChangelogPath "UtocEmulator/CHANGELOG.MD" `
              -CleanBuildDirectory False `

Remove-Item "Publish/Builds" -Recurse -ErrorAction SilentlyContinue

Pop-Location