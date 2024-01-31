# UnrealEssentials
A **WiP** mod for [Reloaded-II](https://reloaded-project.github.io/Reloaded-II/) that makes it easy for other mods to replace files in Unreal Engine games.

## Current Features
The features that are currently functional are: 
- Loading full UTOC and PAK files from mods
- Removing signature checks so any file can be used
- Logging file access (inside of PAKs only for now)
- Support for UE 4.25 to 4.27 games

## WiP Features
Features still under development include:
- Loading loose files from UTOCs using UTOC Emulator (being developed by Rirurin)
- Loading loose files from PAKs (mostly working)
- Support for UE 5 and older UE 4 versions (UE 4 is the priority)

## Usage
First you'll need to create a Reloaded mod and set Unreal Esentials as a dependency of it. For more details on making a mod check out Reloaded's [documentation](https://reloaded-project.github.io/Reloaded-II/CreatingMods/).

### Full UTOC and PAK Files
To make your mod include full UTOC or PAK files simply put them in an `Unreal` folder inside of your mod. You do not need to suffix the file names with `_P` as you normally would if manually placing files in the game's folder, priority will automatically be sorted by Unreal Essentials (although if they do have `_P` in the name it won't hurt).

Below is an example of what a mod might look like. 
![image](https://github.com/AnimatedSwine37/UnrealEssentials/assets/24914353/75e96214-fefc-4138-a718-220dbedcc412)

### Loose PAK Files
To include loose files from PAKs first create an `Unreal` folder inside of your mod. Then create a folder structure mimicking that of the PAK files with your loose files in it. 
To find out what this file structure should look like you can enable file access logging in Unreal Essentials' configuration. Then, load up the game and look for the files being loaded.

For example, if you wanted to replace the font in Scarlet Nexus you would see the following in the logs:
```
[Unreal Essentials] Opening ../../../ScarletNexus/Content/UI/01_Common/00_Font/NEUEFRUTIGERWORLD-REGULAR.ufont
```
From this you can work out that the path to your edited file should be `MyMod/Unreal/ScarletNexus/Content/UI/01_Common/00_Font/NEUEFRUTIGERWORLD-REGULAR.ufont` by ignoring the leading `../../../` in the logged path. An example of how this would look (with a number of other font files also changed) is below.
![image](https://github.com/AnimatedSwine37/UnrealEssentials/assets/24914353/20c3fccd-d2bd-4fc3-8f5e-e1515e740e4c)
