# Set Working Directory
Split-Path $MyInvocation.MyCommand.Path | Push-Location
[Environment]::CurrentDirectory = $PWD

$target_triple = "x86_64-pc-windows-msvc"

./Publish.ps1 -ProjectPath "UnrealEssentials/UnrealEssentials.csproj" `
              -PackageName "UnrealEssentials" `
              -PublishOutputDir "Publish/ToUpload/UnrealEssentials" `
			  -ChangelogPath "UnrealEssentials/CHANGELOG.MD" `
              
Remove-Item "Publish/Builds" -Recurse -ErrorAction SilentlyContinue

.\PublishInterfaces.ps1

# Create Rust project
New-Item "Publish/Builds/CurrentVersion" -ItemType Directory

function BuildEmulator {
    param (
        $Name = ""
    )

    $Output = $Name.Replace("-", "_")

    Push-Location "./UtocEmulator/$Name"
    $env:RUSTFLAGS = "-C panic=abort -C lto=fat -C embed-bitcode=yes"
    $current_ver_folder = [Environment]::CurrentDirectory + "/Publish/Builds/CurrentVersion"
    cargo +nightly rustc --lib --release -Z build-std=std,panic_abort --crate-type cdylib --target x86_64-pc-windows-msvc
    Push-Location "../target/x86_64-pc-windows-msvc/release"
    Copy-Item "$Output.dll" -Destination $current_ver_folder
    Copy-Item "$Output.dll.lib" -Destination $current_ver_folder
    Copy-Item "$Output.dll.exp" -Destination $current_ver_folder
    Pop-Location
    Pop-Location
}

function BuildExtractor {
    param (
        $Name = ""
    )

    Push-Location "./UtocEmulator/$Name"
    # Release
    $env:RUSTFLAGS = "-C panic=abort -C lto=fat -C embed-bitcode=yes"
    cargo +nightly rustc --release -Z build-std=std,panic_abort --target $target_triple
    Push-Location "../target/$target_triple/release"

    
    Pop-Location
    
    Pop-Location
}

BuildEmulator "utoc-emulator"
BuildExtractor "utoc-extractor"

./Publish.ps1 -ProjectPath "UtocEmulator/UTOC.Stream.Emulator/UTOC.Stream.Emulator.csproj" `
              -PackageName "UTOC.Stream.Emulator" `
              -PublishOutputDir "Publish/ToUpload/UTOC.Stream.Emulator" `
			  -ChangelogPath "UtocEmulator/CHANGELOG.MD" `
              -CleanBuildDirectory False `

Remove-Item "Publish/Builds" -Recurse -ErrorAction SilentlyContinue

# utoc-extractor
Remove-Item "Publish/ToUpload/utoc-extractor" -Recurse -ErrorAction SilentlyContinue
New-Item "Publish/ToUpload/utoc-extractor" -ItemType Directory -ErrorAction SilentlyContinue
New-Item "Publish/ToUpload/utoc-extractor/egui.ini" -ErrorAction SilentlyContinue

Copy-Item "UtocEmulator/utoc-extractor/data/config.ini" -Destination "Publish/ToUpload/utoc-extractor"
Copy-Item "UtocEmulator/target/$target_triple/release/utoc-extractor.exe" -Destination "Publish/ToUpload/utoc-extractor"

Compress-Archive -Path "Publish/ToUpload/utoc-extractor/*" -Destination "Publish/ToUpload/utoc-extractor/utoc-extractor.zip"
Remove-Item "Publish/ToUpload/utoc-extractor/*" -Include "*.ini", "*.exe"

Pop-Location