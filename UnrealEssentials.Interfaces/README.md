# Unreal Essentials API
Using the Unreal Essentials API you can change what files are loaded from your mod using C# code. The main use case for this is adding configuration to mods.

## Setting Up
Firstly you will need a Reloaded code mod. If you've not set one up before you can follow Reloaded's [documentation](https://reloaded-project.github.io/Reloaded-II/DevelopmentEnvironmentSetup/) to do so.

With a code mod made you'll need to add Unreal Essentials as a dependency by editing the `ModConfig.json` file and adding `UnrealEssentials` to the array of `ModDependencies`. For example, your `ModConfig.json` might look like:

``` json
{
  "ModId": "p3rpc.relationshipFortunesPlus",
  "ModName": "Relationship Fortunes+",
  "ModAuthor": "AnimatedSwine37",
  "ModVersion": "1.0.0",
  "ModDescription": "Adds an option to draw a quick relationship fortune.",
  "ModDll": "p3rpc.relationshipFortunesPlus.dll",
  "ModIcon": "Preview.png",
  "ModR2RManagedDll32": "x86/p3rpc.relationshipFortunesPlus.dll",
  "ModR2RManagedDll64": "x64/p3rpc.relationshipFortunesPlus.dll",
  "ModNativeDll32": "",
  "ModNativeDll64": "",
  "IsLibrary": false,
  "ReleaseMetadataFileName": "p3rpc.relationshipFortunesPlus.ReleaseMetadata.json",
  "PluginData": {},
  "IsUniversalMod": false,
  "ModDependencies": [
    "reloaded.sharedlib.hooks",
    "UnrealEssentials"
  ],
  "OptionalDependencies": [],
  "SupportedAppId": ["p3r.exe"],
  "ProjectUrl": ""
}
```

Then you will need to add the [UnrealEssentials.Interfaces NuGet package](https://www.nuget.org/packages/UnrealEssentials.Interfaces) to your project. 
In Visual Studio you can do this by right clicking the project and selecting **Manage NuGet Packages...** which will open up a new tab. 

![image](https://github.com/AnimatedSwine37/UnrealEssentials/assets/24914353/da9f5c13-0e32-43ac-adc2-e5ab79ff5647)

In this tab search for `UnrealEssentials.Interfaces` and press **Install** on it.

![image](https://github.com/AnimatedSwine37/UnrealEssentials/assets/24914353/6225f44c-3896-4c02-b8b1-1ac8acbc5bb2)

If you're using a different IDE you'll have to work out how to add NuGet packages to your project yourself.

Now you will be able to get access to the API as described in the [Reloaded documentation on dependency injection](https://reloaded-project.github.io/Reloaded-II/DependencyInjection_Consumer/). For example, your code would likely look like:

```cs
var unrealEssentialsController = _modLoader.GetController<IUnrealEssentials>();
if (unrealEssentialsController == null || !unrealEssentialsController.TryGetTarget(out var unrealEssentials))
{
    _logger.WriteLine($"[My Mod] Unable to get controller for Unreal Essentials, stuff won't work :(", System.Drawing.Color.Red);
    return;
}
```

You will want to put this underneath the `// TODO: Implement some mod logic` comment in the constructor in `Mod.cs` so it is run when your mod loads.

## Adding Files From Code
The only thing the API currently can do is add files to be loaded by UnrealEssentials. To do so you use the `AddFromFolder` method of the `IUnrealEssentials` object you got using the above code.

For example, to add files from a folder called `TestFolder` in your mod the code would look like:

```cs
var modDir = _modLoader.GetDirectoryForModId(_modConfig.ModId);
var filesPath = Path.Combine(modDir, "TestFolder");
unrealEssentials.AddFromFolder(filesPath);
```

An important part to note is the need for getting the path to your mod's folder using `_modLoader.GetDirectoryForModId(_modConfig.ModId);`. If you do not do this Unreal Essentials will look for files relative to the game's executable instead of your mod.

This folder will essentially be treated as if it were the `UnrealEssentials` folder in the root of your mod so you would format files the same way. In the case of the `TestFolder` example you'd have files like `TestFolder\Game\Content\...`.
