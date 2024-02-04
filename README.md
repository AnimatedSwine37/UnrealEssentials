# UnrealEssentials
A mod for [Reloaded-II](https://reloaded-project.github.io/Reloaded-II/) that makes it easy for other mods to replace files in Unreal Engine games.

## Features
- Loading full UTOC and PAK files from mods
- Loading loose files from UTOCs and PAKs
- Removing signature checks so any file can be used
- Logging file access (inside of PAKs only for now)
- Support for UE 4.25 to 4.27 games

## Planned Features
- Support for older UE4 versions and UE5
- Automatic conversion of cooked uassets to IO Store uassets (see note in [Loose Files](#loose-files))

## Usage
First you'll need to create a Reloaded mod and set Unreal Esentials as a dependency of it. For more details on making a mod check out Reloaded's [documentation](https://reloaded-project.github.io/Reloaded-II/CreatingMods/).

Next, open your mod's folder and create an `UnrealEssentials` folder inside of it, this is where you will put your edited files. 

### Full UTOC and PAK Files
To include full UTOC or PAK files simply put them anywhere in the `UnrealEssentials` folder (you can use subfolders if you'd like). 

You do not need to suffix the file names with `_P` as you normally would if manually placing files in the game's folder, priority will automatically be sorted by Unreal Essentials (although if they do have `_P` in the name it won't hurt).

For example, a mod from Scarlet Nexus that uses full files looks like

![image](https://github.com/AnimatedSwine37/UnrealEssentials/assets/24914353/54d8bb20-c2d1-4f91-a653-9ca2bb59c6c7)

### Loose Files
To include loose files put them in the `UnrealEssentials` folder, replicating their folder structure from the original game (this structure will generally start with `GameName/Content`).

Note that if your game uses UTOC files, any **.uasset** files you replace will have to come from a UTOC as the file format is different when they are in PAK files. This means that you will need to export them from Unreal Engine in utocs and then extract them if you want to use them loosely. This will be fixed at a later time.

For example, using [FModel](https://github.com/4sval/FModel) we could find the font files in Persona 3 Reload at `P3R/Content/Xrd777/Font`

![image](https://github.com/AnimatedSwine37/UnrealEssentials/assets/24914353/53544a0d-b41c-4aff-afa5-4aa621f462ba)

To then replace one of these files we'd put our edited one in `UnrealEssentials/P3R/Content/Xrd777/Font` like

![image](https://github.com/AnimatedSwine37/UnrealEssentials/assets/24914353/3c25cb0f-c44d-4304-90fa-e71457eb6b45)
