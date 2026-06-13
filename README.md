# UnrealEssentials
A mod for [Reloaded-II](https://reloaded-project.github.io/Reloaded-II/) that makes it easy for other mods to replace files in Unreal Engine games.

## Features
- Loading full UTOC and PAK files from mods
- Loading loose files from UTOCs and PAKs
- Removing signature checks so any file can be used
- Logging file access
- Support for UE 4.25-4.27 and UE 5 (see [Supported Games](#supported-games) for more details)
- API for adding file replacements from code (see [documentation](/UnrealEssentials.Interfaces/README.md))

## Planned Features
- Support for older UE4 versions
- Automatic conversion of cooked uassets to IO Store uassets (see note in [Loose Files](#loose-files))

## Supported Games
Below is a list of games that are known to work with Unreal Essentials. Just because a game isn't on the list doesn't mean it doesn't work, generally UE 4 games from 4.25-4.27 and UE 5 games will work.

If you know of a game that doesn't work you can create an [issue](https://github.com/AnimatedSwine37/UnrealEssentials/issues) and support might be added for it.

| Game       | UE Version | Support      |
|------------|-|------------|
| [Clair Obscur: Expedition 33](https://store.steampowered.com/app/1903340/Clair_Obscur_Expedition_33/) | 5.4 | TODO
| [DRAGON BALL: Sparking! ZERO](https://store.steampowered.com/app/1790600/DRAGON_BALL_Sparking_ZERO/) | 5.1 | TODO
| [Final Fantasy 7 Rebirth](https://store.steampowered.com/app/2909400/FINAL_FANTASY_VII_REBIRTH/) | 4.26 | TODO, requires custom engine version due to IO Store changes
| [Hi-Fi Rush](https://store.steampowered.com/app/1817230/HiFi_RUSH/)       | 4.27 |  Microsoft Store version is currently broken ([Issue](https://github.com/AnimatedSwine37/UnrealEssentials/issues/13)) |
| [Hogwarts Legacy](https://store.steampowered.com/app/990080/Hogwarts_Legacy/) | 4.27 |
| [HOLE](https://store.steampowered.com/app/2971610/HOLE/) | 5.5 | TODO
| [Invincible VS](https://store.steampowered.com/app/2353060/Invincible_VS/) | 5.5 | TODO
| [inZOI](https://store.steampowered.com/app/2456740/inZOI/) | 5.6 | TODO
| [Jujutsu Kaisen: Cursed Clash](https://store.steampowered.com/app/1877020/Jujutsu_Kaisen_Cursed_Clash/) | 5.1 | TODO
| [Lies of P](https://store.steampowered.com/app/1627720/Lies_of_P/) | 4.27 | TODO
| [Life is Strange: Double Exposure](https://store.steampowered.com/app/1874000/Life_is_Strange_Double_Exposure/) | 5.2 | TODO
| [Lego Batman: Legacy of the Dark Knight](https://store.steampowered.com/app/2215200/LEGO_Batman_Legacy_of_the_Dark_Knight/) | 5.6 | TODO
| [Marvel Rivals](https://store.steampowered.com/app/2767030/Marvel_Rivals/) | 5.3 | TODO
| [Master Detective Archives RAIN CODE](https://store.steampowered.com/app/2903950/Master_Detective_Archives_RAIN_CODE_Plus/) | 4.27 |
| [Nobody Wants to Die](https://store.steampowered.com/app/1939970/Nobody_Wants_to_Die/) | 5.3 | TODO
| [Outside the Blocks](https://store.steampowered.com/app/2350220/Outside_the_Blocks/) | 5.4 | TODO
| [Persona 3 Reload](https://store.steampowered.com/app/2161700/Persona_3_Reload/) | 4.27 | Use [Persona 3 Reload Essentials](https://gamebanana.com/mods/494020) for game specific features
| [ROMEO IS A DEAD MAN](https://store.steampowered.com/app/3050900/ROMEO_IS_A_DEAD_MAN/) | 5.6 | TODO
| [Rune Factory: Guardians of Azuma](https://store.steampowered.com/app/2864560/Rune_Factory_Guardians_of_Azuma/) | 5.4 | TODO
| [Sackboy: A Big Adventure](https://store.steampowered.com/app/1599660/Sackboy_A_Big_Adventure/) | 4.25 |
| [SCARLET NEXUS](https://store.steampowered.com/app/775500/SCARLET_NEXUS/) | 4.25 |
| [Shin Megami Tensei V: Vengeance](https://store.steampowered.com/app/1875830/Shin_Megami_Tensei_V_Vengeance/) | 4.27 |
| [Sonic Racing: CrossWorlds](https://store.steampowered.com/app/2486820/Sonic_Racing_CrossWorlds/) | 5.4 | TODO
| [Spirit City: Lofi Sessions](https://store.steampowered.com/app/2113850/Spirit_City_Lofi_Sessions/) | 5.7 | TODO
| [Subnautica 2](https://store.steampowered.com/app/1962700/Subnautica_2/) | 5.6 | TODO
| [The Callisto Protocol](https://store.steampowered.com/app/1544020/The_Callisto_Protocol/) | 4.27 | Need to use ASI Loader or remove DRM with [Steamless](https://github.com/atom0s/Steamless/) |

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
