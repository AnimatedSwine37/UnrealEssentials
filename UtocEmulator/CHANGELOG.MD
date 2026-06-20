# UTOC Emulator Changelog
## 2.0.0
@rirurin ( + testing from @raycopper ) :
- Logs made in the Rust part of UTOC Emulator are now printed using the Reloaded logger so it's saved to the log file.
- Added full archive loading and loose file support for Unreal Engine 5.0 to 5.7.
  - For versions UE 5.0 - 5.2, it's required that all loose assets include some asset metadata to ensure that dependencies can be resolved accurately (this is also an issue with UE4 but is optional to maintain backwards compatibility). This can either take the form of metadata for each asset (`.uassetmeta`) or as one table in the root folder (`.utocmeta`)
- Rewritten UTOC Emulator to use [retoc](https://github.com/trumank/retoc) for serialization and to simplify the asset collector and archive builder.
- Added a command-line and GUI tool (`utoc-extractor`) to allow for IO Store archive unpacking with automatically generated asset metadata and for conversion between metadata forms.

## 1.1.3
- Fixed IO Store files not being emulated if the file was created/opened multiple times
- Removed the file size hack as the related bug has now been fixed in File Emulation Framework (see [#15](https://github.com/Sewer56/FileEmulationFramework/issues/15))

## 1.1.2
- Fixed a crash when a game that uses IO Store has no mods with emulated utoc files. i.e. if only full utoc files are included using Unreal Essentials.

## 1.1.1
- Fixed a crash when trying to load utoc files from paths with non ascii characters in them. (This was partially fixed in 1.0.3, it should be completely fixed now)

## 1.1.0
- Updated to support the new UnrealEssentials API. Check out the [Unreal Essentials documentation](https://github.com/AnimatedSwine37/UnrealEssentials/tree/master/UnrealEssentials.Interfaces/README.md) for information on using it.

## 1.0.4
- Fixed a crash caused by trying to load any assets with multiple export bundle, including blueprints assets

## 1.0.3
- Fixed a crash when trying to load utoc files from paths with non ascii characters in them. 