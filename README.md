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
Below is a list of games that are known to work with Unreal Essentials. Just because a game isn't on the list doesn't mean it doesn't work, generally UE 4 games from 4.25-4.27 and UE 5 games from 5.0-5.7 will work.

If you know of a game that doesn't work you can create an [issue](https://github.com/AnimatedSwine37/UnrealEssentials/issues) and support might be added for it.

| Game       | UE Version | Support      |
|------------|-|------------|
| [Clair Obscur: Expedition 33](https://store.steampowered.com/app/1903340/Clair_Obscur_Expedition_33/) | 5.4 |
| [DRAGON BALL: Sparking! ZERO](https://store.steampowered.com/app/1790600/DRAGON_BALL_Sparking_ZERO/) | 5.1 |
| [Hi-Fi Rush](https://store.steampowered.com/app/1817230/HiFi_RUSH/)       | 4.27 | Requires [hibiki-bootstrap](https://github.com/akmubi/hibiki-bootstrap) |
| [Hogwarts Legacy](https://store.steampowered.com/app/990080/Hogwarts_Legacy/) | 4.27 |
| [HOLE](https://store.steampowered.com/app/2971610/HOLE/) | 5.5 |
| [Invincible VS](https://store.steampowered.com/app/2353060/Invincible_VS/) | 5.5 |
| [inZOI](https://store.steampowered.com/app/2456740/inZOI/) | 5.6 |
| [Jujutsu Kaisen: Cursed Clash](https://store.steampowered.com/app/1877020/Jujutsu_Kaisen_Cursed_Clash/) | 5.1 |
| [Life is Strange: Double Exposure](https://store.steampowered.com/app/1874000/Life_is_Strange_Double_Exposure/) | 5.2 |
| [Lego Batman: Legacy of the Dark Knight](https://store.steampowered.com/app/2215200/LEGO_Batman_Legacy_of_the_Dark_Knight/) | 5.6 |
| [Master Detective Archives RAIN CODE](https://store.steampowered.com/app/2903950/Master_Detective_Archives_RAIN_CODE_Plus/) | 4.27 | Need to use ASI Loader or remove DRM with [Steamless](https://github.com/atom0s/Steamless/)
| [Nobody Wants to Die](https://store.steampowered.com/app/1939970/Nobody_Wants_to_Die/) | 5.3 |
| [Outside the Blocks](https://store.steampowered.com/app/2350220/Outside_the_Blocks/) | 5.4 |
| [Persona 3 Reload](https://store.steampowered.com/app/2161700/Persona_3_Reload/) | 4.27 | Use [Persona 3 Reload Essentials](https://gamebanana.com/mods/494020) for game specific features
| [ROMEO IS A DEAD MAN](https://store.steampowered.com/app/3050900/ROMEO_IS_A_DEAD_MAN/) | 5.6 | Need to use ASI Loader or remove DRM with [Steamless](https://github.com/atom0s/Steamless/)
| [Rune Factory: Guardians of Azuma](https://store.steampowered.com/app/2864560/Rune_Factory_Guardians_of_Azuma/) | 5.4 |
| [Sackboy: A Big Adventure](https://store.steampowered.com/app/1599660/Sackboy_A_Big_Adventure/) | 4.25 |
| [SCARLET NEXUS](https://store.steampowered.com/app/775500/SCARLET_NEXUS/) | 4.25 |
| [Shin Megami Tensei V: Vengeance](https://store.steampowered.com/app/1875830/Shin_Megami_Tensei_V_Vengeance/) | 4.27 |
| [Sonic Racing: CrossWorlds](https://store.steampowered.com/app/2486820/Sonic_Racing_CrossWorlds/) | 5.4 |
| [Spirit City: Lofi Sessions](https://store.steampowered.com/app/2113850/Spirit_City_Lofi_Sessions/) | 5.7 |
| [Subnautica 2](https://store.steampowered.com/app/1962700/Subnautica_2/) | 5.6 |
| [The Adventures of Elliot: The Millennium Tales](https://store.steampowered.com/app/3483510/The_Adventures_of_Elliot_The_Millennium_Tales/) | 5.6 |
| [The Callisto Protocol](https://store.steampowered.com/app/1544020/The_Callisto_Protocol/) | 4.27 | Need to use ASI Loader or remove DRM with [Steamless](https://github.com/atom0s/Steamless/) |

## Usage
First you'll need to create a Reloaded mod and set Unreal Esentials as a dependency of it. For more details on making a mod check out Reloaded's [documentation](https://reloaded-project.github.io/Reloaded-II/CreatingMods/).

Next, open your mod's folder and create an `UnrealEssentials` folder inside of it, this is where you will put your edited files. 

### Adding Full Packages
To include a full package (`.utoc` + `.ucas` or `.pak`), place them anywhere in the `UnrealEssentials` folder. You can use subfolders if you'd like.

You do not need to suffix the file names with `_P` as you normally would if manually placing files in the game's folder, priority will automatically be sorted by Unreal Essentials (although if they do have `_P` in the name it won't hurt).

For example, a mod from Scarlet Nexus that uses full files looks like

![image](https://github.com/AnimatedSwine37/UnrealEssentials/assets/24914353/54d8bb20-c2d1-4f91-a653-9ca2bb59c6c7)

### Adding Loose Assets

To include loose files put them in the `UnrealEssentials` folder, replicating their folder structure from the original game (this structure will generally start with `GameName/Content`).

Note that if your game uses UTOC files, any `.uasset` files you replace will have to come from a UTOC as the file format is different when they are in PAK files. This means that you will need to export them from Unreal Engine into an IO Store container ()`.utoc` + `.ucas` and then extract them if you want to use them loosely. This will be fixed at a later time.

For example, using [FModel](https://github.com/4sval/FModel) we could find the font files in Persona 3 Reload at `P3R/Content/Xrd777/Font`

![image](https://github.com/AnimatedSwine37/UnrealEssentials/assets/24914353/53544a0d-b41c-4aff-afa5-4aa621f462ba)

To then replace one of these files we'd put our edited one in `UnrealEssentials/P3R/Content/Xrd777/Font` like

![image](https://github.com/AnimatedSwine37/UnrealEssentials/assets/24914353/3c25cb0f-c44d-4304-90fa-e71457eb6b45)

### Notes for Loose Zen Assets

*Zen Assets refer to assets originating from an IO Store container, which when unpacked will output a `.uasset` without any associated `.uexp`.*

There are some things to consider when using loose zen assets to ensure that your mod works as expected and to reduce loading times for the emulator.

For UE versions below 5.3, there is insufficient dependency information in the asset to accurately construct the imports and exports needed for the asset to function properly. While Unreal Essentials 1.x includes methods for deriving dependency info in UE4, the process is not perfect and there are occasionally files that don't resolve the right information, resulting in a game crash or other unintended behavior.

To resolve this, Unreal Essentials can use asset metadata that can be either defined per-asset in a `.uassetmeta` or for an entire mod in a `.utocmeta`. These files are generated [using the UTOC Extractor](#using-the-utoc-extractor).

For **UE 4.25 - 4.27**, asset metadata is optional to maintain backwards compatibility with 1.x. However, we recommend that mod authors use the UTOC extractor to generate asset metadata to avoid the issues detailed above.

For **UE 5.0 - 5.2**, asset metadata is required and UTOC emulator will fail with the message *"Asset metadata is required for UE5 versions before 5.3!"* if it is missing.

For **UE 5.3** and above, asset metadata is optional.

### Using the UTOC Extractor

#### CLI

...

#### GUI

...

## Credits
- **[trumank](https://github.com/trumank)** and **[Archengius](https://github.com/Archengius)** - Developers of [retoc](https://github.com/trumank/retoc/), the serialization library used by UTOC Emulator