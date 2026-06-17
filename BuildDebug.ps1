# globals
$emulator_name_csharp = "UTOC.Stream.Emulator"
$target_triple = "x86_64-pc-windows-msvc"
$output_name = "fileemu_utoc_stream_emulator"

function BuildCsharpProject {
    param (
        $Path = "",
        $Project = $Path
    )

    Push-Location "./$Path"
    dotnet build "./$Project.csproj" -v q -c Debug 
    Pop-Location
}

function BuildEmulator {
    param (
        $Name = ""
    )

    $Output = $Name.Replace("-", "_")

    Push-Location "./UtocEmulator/$Name"
    $env:RUSTFLAGS = "-C panic=abort -C lto=fat -C embed-bitcode=yes"
    cargo +nightly rustc --lib --release -Z build-std=std,panic_abort --crate-type cdylib --target $target_triple
    # copy required files from target
    Push-Location "../target/$target_triple/release"
    Copy-Item "$Output.dll" -Destination "$env:RELOADEDIIMODS\$emulator_name_csharp"
    Copy-Item "$Output.dll.lib" -Destination "$env:RELOADEDIIMODS\$emulator_name_csharp"
    Copy-Item "$Output.dll.exp" -Destination "$env:RELOADEDIIMODS\$emulator_name_csharp"
    Pop-Location
    Pop-Location
}

function BuildExtractor {
    param (
        $Name = ""
    )

    Push-Location "./UtocEmulator/$Name"
    $env:RUSTFLAGS = ""
    cargo +nightly rustc --target $target_triple
    Push-Location "../target/$target_triple/release"
    New-Item "$env:RELOADEDIIMODS\UnrealEssentials\Tools" -ItemType Directory -ErrorAction SilentlyContinue
    Copy-Item "$Name.exe" -Destination "$env:RELOADEDIIMODS\UnrealEssentials\Tools"
    Pop-Location
    Pop-Location
}

BuildCsharpProject "UnrealEssentials"
BuildCsharpProject "UnrealEssentials.Interfaces"
BuildEmulator "utoc-emulator"
BuildExtractor "utoc-extractor"
BuildCsharpProject "UtocEmulator/$emulator_name_csharp" $emulator_name_csharp