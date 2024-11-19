# 1.1.2
- Fixed a crash when a game that uses IO Store has no mods with emulated utoc files. i.e. if only full utoc files are included using Unreal Essentials.

# 1.1.1
- Fixed a crash when trying to load utoc files from paths with non ascii characters in them. (This was partially fixed in 1.0.3, it should be completely fixed now)

# 1.1.0
- Updated to support the new UnrealEssentials API. Check out the [Unreal Essentials documentation](https://github.com/AnimatedSwine37/UnrealEssentials/tree/master/UnrealEssentials.Interfaces/README.md) for information on using it.

# 1.0.4
- Fixed a crash caused by trying to load any assets with multiple export bundle, including blueprints assets

# 1.0.3
- Fixed a crash when trying to load utoc files from paths with non ascii characters in them. 