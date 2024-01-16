# Set Working Directory
Split-Path $MyInvocation.MyCommand.Path | Push-Location
[Environment]::CurrentDirectory = $PWD

Remove-Item "$env:RELOADEDIIMODS/UnrealEssentials/*" -Force -Recurse
dotnet publish "./UnrealEssentials.csproj" -c Release -o "$env:RELOADEDIIMODS/UnrealEssentials" /p:OutputPath="./bin/Release" /p:ReloadedILLink="true"

# Restore Working Directory
Pop-Location