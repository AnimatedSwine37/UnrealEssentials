# globals
$unreal_essentials = "UnrealEssentials"
$unreal_essentials_interface = $unreal_essentials + ".Interfaces"
$emulator_parent = "UtocEmulator"
$emulator_main = "fileemu-utoc-stream-emulator"
$emulator_name_csharp = "UTOC.Stream.Emulator"
$target_triple = "x86_64-pc-windows-msvc"
$output_name = "fileemu_utoc_stream_emulator"

# build Unreal Essentials
Push-Location "./$unreal_essentials"
dotnet build "./$unreal_essentials.csproj" -v q -c Debug 
Pop-Location
# build Unreal Essentials Interfaces
Push-Location "./$unreal_essentials_interface"
dotnet build "./$unreal_essentials_interface.csproj" -v q -c Debug 
Pop-Location
# build UTOC Emulator
Push-Location "./$emulator_parent/$emulator_main"
# cargo +nightly build --lib --target x86_64-pc-windows-msvc --profile release -Z unstable-options --out-dir "$env:RELOADEDIIMODS\$emulator_name_csharp\"
$env:RUSTFLAGS = "-C panic=abort -C lto=fat -C embed-bitcode=yes"
cargo +nightly rustc --lib --release -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --crate-type cdylib --target $target_triple
# copy required files from target
Push-Location "../target/$target_triple/release"
Copy-Item "$output_name.dll" -Destination "$env:RELOADEDIIMODS\$emulator_name_csharp"
Copy-Item "$output_name.dll.lib" -Destination "$env:RELOADEDIIMODS\$emulator_name_csharp"
Copy-Item "$output_name.dll.exp" -Destination "$env:RELOADEDIIMODS\$emulator_name_csharp"
Pop-Location
Pop-Location
Push-Location "./$emulator_parent/$emulator_name_csharp"
dotnet build "./$emulator_name_csharp.csproj" -v q -c Debug 
Pop-Location