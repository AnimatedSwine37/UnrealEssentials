# globals
$unreal_essentials = "UnrealEssentials"
# $unreal_essentials_interface = $unreal_essentials + ".Interfaces"
$emulator_parent = "UtocEmulator"
$emulator_main = "fileemu-utoc-stream-emulator"
# $emulator_tests = "toc-builder-test"
$emulator_name_csharp = "UTOC.Stream.Emulator"

# build Unreal Essentials
Push-Location "./$unreal_essentials"
dotnet build "./$unreal_essentials.csproj" -v q -c Debug 
Pop-Location
# build Unreal Essentials Interfaces
# build UTOC Emulator
# Push-Location "./$emulator_parent"
# cargo +nightly build --target x86_64-pc-windows-msvc # build for both targets in workspace
Push-Location "./$emulator_parent/$emulator_main"
cargo +nightly build --lib --target x86_64-pc-windows-msvc --profile release -Z unstable-options --out-dir "$env:RELOADEDIIMODS\$emulator_name_csharp\"
Pop-Location
Push-Location "./$emulator_parent/$emulator_name_csharp"
dotnet build "./$emulator_name_csharp.csproj" -v q -c Debug 
Pop-Location