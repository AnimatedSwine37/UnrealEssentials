# Set Working Directory
Split-Path $MyInvocation.MyCommand.Path | Push-Location
[Environment]::CurrentDirectory = $PWD

Remove-Item "$env:RELOADEDIIMODS/UTOC.Stream.Emulator/*" -Force -Recurse
dotnet publish "./UTOC.Stream.Emulator.csproj" -c Release -o "$env:RELOADEDIIMODS/UTOC.Stream.Emulator" /p:OutputPath="./bin/Release" /p:ReloadedILLink="true"

# Restore Working Directory
Pop-Location