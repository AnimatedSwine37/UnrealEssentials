# Changelog
## Unreal Essentials 2.0.0 & UTOC Emulator 2.0.0
@rirurin ( + testing from @raycopper ) :
- Logs made in the Rust part of UTOC Emulator are now printed using the Reloaded logger so it's saved to the log file.
- Refactored signature storage into a collection of YAML files stored in `Signature`. `Signature/Engine` contains YAML definitions for each supported UE version and `Signature/Game` contains definitions for games where the default engine signatures and parameters are not sufficient
  - To add a new game, specify the `EngineVersion` property with a value matching one of the file names in `Signature/Engine`, then define some identifier for the executable
  - This can be `ExecutableName` using the name of the executable without the file extension (use `<DistVersion>` to match Win64 and WinGDK), or `ExecutableNameStartsWith` or `ProductName` to check based on the file's product description (this is done for Guardians of Azuma)
- `TryGetProperties` in `Context.cs` will check the executable's file version before sigscanning for the branch version. From the testing done, branch versions appear less frequently in UE5 games.
- Added a file log to report loading IO Store assets.
- Fixed signatures for Hi-Fi RUSH on Steam that broke as of Patch 10.
- Added signatures for the following games:
  - [Clair Obscur: Expedition 33](https://store.steampowered.com/app/1903340/Clair_Obscur_Expedition_33/) (5.4, in `Expedition33.yaml`)
  - [Jujutsu Kaisen: Cursed Clash](https://store.steampowered.com/app/1877020/Jujutsu_Kaisen_Cursed_Clash/) (5.1, in `JJKCC.yaml`)
  - [Lego Batman: Legacy of the Dark Knight](https://store.steampowered.com/app/2215200/LEGO_Batman_Legacy_of_the_Dark_Knight/) (5.6, in `LegoBatmanLotDK.yaml`)
  - Marvel Rivals (5.3, in `MarvelRivals.yaml`). This is not listed as supported since ASI Loader hangs while trying to load Reloaded-II
  - [Nobody Wants to Die](https://store.steampowered.com/app/1939970/Nobody_Wants_to_Die/)  (5.3, in `NobodyWantsToDie.yaml`)
  - [ROMEO IS A DEAD MAN](https://store.steampowered.com/app/3050900/ROMEO_IS_A_DEAD_MAN/) (5.6, in `RomeoIsADeadMan.yaml`)
  - [Rune Factory: Guardians of Azuma](https://store.steampowered.com/app/2864560/Rune_Factory_Guardians_of_Azuma/) (5.4, in `GuardiansOfAzuma.yaml`)
  - [Sonic Racing: CrossWorlds](https://store.steampowered.com/app/2486820/Sonic_Racing_CrossWorlds/) (5.4, in `SonicRacingCrossworlds.yaml`)
  - [The Adventures of Elliot: The Millennium Tales](https://store.steampowered.com/app/3483510/The_Adventures_of_Elliot_The_Millennium_Tales/) (5.6, in `ElliotMillennium.yaml`)
- Fixed bug where asset dependency information from the first mod to replace/add an asset would be retained even if it was overwritten by a higher priority mod.
- Added full archive loading and loose file support for Unreal Engine 5.0 to 5.7.
  - For versions UE 5.0 - 5.2, it's required that all loose assets include some asset metadata to ensure that dependencies can be resolved accurately (this is also an issue with UE4 but is optional to maintain backwards compatibility). This can either take the form of metadata for each asset (`.uassetmeta`) or as one table in the root folder (`.utocmeta`)
- Rewritten UTOC Emulator to use [retoc](https://github.com/trumank/retoc) for serialization and to simplify the asset collector and archive builder.
- Added a command-line and GUI tool (`utoc-extractor`) to allow for IO Store archive unpacking with automatically generated asset metadata and for conversion between metadata forms.

## Unreal Essentials 1.3.0 & UTOC Emulator 1.2.0
- @TheBestAstroNOT Added support for adding files or folders with a virtual path (path that is different from that on the OS) through the mod's API and the new UEMounts.yaml file (documentation pending)