using Reloaded.Hooks.Definitions;
using Reloaded.Memory.Sigscan;
using Reloaded.Mod.Interfaces;
using System.Diagnostics;
using System.Runtime.InteropServices;
using UnrealEssentials.Configuration;
using UnrealEssentials.Template;
using static UnrealEssentials.Unreal.Native;
using static UnrealEssentials.Utils;
using static UnrealEssentials.Unreal.UnrealMemory;
using static UnrealEssentials.Unreal.UnrealString;
using static UnrealEssentials.Unreal.UnrealArray;
using IReloadedHooks = Reloaded.Hooks.ReloadedII.Interfaces.IReloadedHooks;
using Reloaded.Mod.Interfaces.Internal;

namespace UnrealEssentials;
/// <summary>
/// Your mod logic goes here.
/// </summary>

public unsafe class Mod : ModBase // <= Do not Remove.
{
    /// <summary>
    /// Provides access to the mod loader API.
    /// </summary>
    private readonly IModLoader _modLoader;

    /// <summary>
    /// Provides access to the Reloaded.Hooks API.
    /// </summary>
    /// <remarks>This is null if you remove dependency on Reloaded.SharedLib.Hooks in your mod.</remarks>
    private readonly IReloadedHooks? _hooks;

    /// <summary>
    /// Provides access to the Reloaded logger.
    /// </summary>
    private readonly ILogger _logger;

    /// <summary>
    /// Entry point into the mod, instance that created this class.
    /// </summary>
    private readonly IMod _owner;

    /// <summary>
    /// Provides access to this mod's configuration.
    /// </summary>
    private Config _configuration;

    /// <summary>
    /// The configuration of the currently executing mod.
    /// </summary>
    private readonly IModConfig _modConfig;

    private IHook<GetPakSigningKeysDelegate> _getSigningKeysHook;
    private IHook<GetPakFoldersDelegate> _getPakFoldersHook;
    private FPakSigningKeys* _signingKeys;

    // For testing with Scarlet Nexus 
    // TODO either remove this or add signatures for other unreal versions
    private IHook<IoDispatcherMountDelegate> _mountUtocHook;
    private IHook<PakPlatformFileMountDelegate> _mountPakHook;
    private IHook<FindAllPakFilesDelegate> _findAllPakFilesHook;

    private List<string> _pakFolders = new();

    public Mod(ModContext context)
    {
        //Debugger.Launch();
        _modLoader = context.ModLoader;
        _hooks = context.Hooks;
        _logger = context.Logger;
        _owner = context.Owner;
        _configuration = context.Configuration;
        _modConfig = context.ModConfig;

        Initialise(_logger, _configuration, _modLoader);

        // Setup empty signing keys
        _signingKeys = (FPakSigningKeys*)NativeMemory.Alloc((nuint)sizeof(FPakSigningKeys));
        _signingKeys->Function = 0;
        _signingKeys->Size = 0;

        // Get Signatures
        var sigs = GetSignatures();

        InitialiseGMalloc(sigs.GMalloc, _hooks);

        // Remove utoc signing
        SigScan(sigs.GetPakSigningKeys, "GetSigningKeysPtr", address =>
        {
            var funcAddress = GetGlobalAddress(address + 1);
            LogDebug($"Found GetSigningKeys at 0x{funcAddress:X}");
            _getSigningKeysHook = _hooks.CreateHook<GetPakSigningKeysDelegate>(GetPakSigningKeys, (long)funcAddress).Activate();
        });

        // Load files from our mod
        SigScan(sigs.GetPakFolders, "GetPakFolders", address =>
        {
            _getPakFoldersHook = _hooks.CreateHook<GetPakFoldersDelegate>(GetPakFolders, address).Activate();
        });

        // Log mounting (for testing)
        //SigScan("40 53 41 56 41 57 48 81 EC 70 01 00 00", "FIoDispatcherImpl::Mount", address =>
        //{
        //    _mountUtocHook = _hooks.CreateHook<IoDispatcherMountDelegate>(MountUtoc, address).Activate();
        //});

        //SigScan("40 55 53 56 57 41 54 41 55 41 56 41 57 48 8D AC 24 ?? ?? ?? ?? 48 81 EC 28 02 00 00 48 8B 05 ?? ?? ?? ?? 48 33 C4 48 89 85 ?? ?? ?? ??", "FPakPlatformFile::Mount", address =>
        //{
        //    _mountPakHook = _hooks.CreateHook<PakPlatformFileMountDelegate>(MountPak, address).Activate();
        //});

        //SigScan("48 8B C4 4C 89 40 ?? 53 55 56 57 48 83 EC 58", "FindAllPakFiles", address =>
        //{
        //    _findAllPakFilesHook = _hooks.CreateHook<FindAllPakFilesDelegate>(FindAllPakFiles, address).Activate();
        //});

        // Gather pak files from mods
        _modLoader.ModLoading += ModLoading;
    }

    private Signatures GetSignatures()
    {
        var CurrentProcess = Process.GetCurrentProcess();
        var mainModule = CurrentProcess.MainModule;
        var fileName = Path.GetFileName(mainModule!.FileName);

        // Try and find based on file name
        if (Signatures.VersionSigs.TryGetValue(fileName, out var sigs))
            return sigs;

        // Try and find based on branch name
        var scanner = new Scanner(CurrentProcess, mainModule);
        var res = scanner.FindPattern("2B 00 2B 00 55 00 45 00 34 00 2B 00"); // ++UE4+
        if (!res.Found)
        {
            res = scanner.FindPattern("2B 00 2B 00 75 00 65 00 34 00 2B 00"); // ++ue4+
            if (!res.Found)
            {
                throw new Exception($"Unable to find Unreal Engine version number." +
                    $"\nPlease report this!");
            }
        }

        string branch = Marshal.PtrToStringUni(res.Offset + BaseAddress)!;
        Log($"Unreal Engine branch is {branch}");
        if (!Signatures.VersionSigs.TryGetValue(branch, out sigs))
        {
            throw new Exception($"Unable to find signatures for Unreal Engine branch {branch}." +
                $"\nPlease report this!");
        }

        return sigs;
    }

    private void FindAllPakFiles(nuint LowerLevelFile, TArray<FString>* PakFolders, FString* WildCard, TArray<FString>* OutPakFiles)
    {
        LogDebug($"Searching for pak files in folders:\n{string.Join('\n', *PakFolders)}");
        _findAllPakFilesHook.OriginalFunction(LowerLevelFile, PakFolders, WildCard, OutPakFiles);
        LogDebug($"Found pak files:\n{string.Join('\n', *OutPakFiles)}");
    }

    private bool MountPak(nuint thisPtr, char* InPakFilename, int PakOrder, char* InPath, bool bLoadIndex)
    {
        var pakName = Marshal.PtrToStringUni((nint)InPakFilename);
        LogDebug($"Mounting PAK {pakName} with initial priority {PakOrder}");
        return _mountPakHook.OriginalFunction(thisPtr, InPakFilename, PakOrder, InPath, bLoadIndex);
    }

    private void ModLoading(IModV1 mod, IModConfigV1 modConfig)
    {
        if (modConfig.ModDependencies.Contains(_modConfig.ModId))
        {
            var modsPath = Path.Combine(_modLoader.GetDirectoryForModId(modConfig.ModId), "Unreal");
            _pakFolders.Add(modsPath);
            Log($"Loading files from {modsPath}");
        }
    }

    private nuint MountUtoc(nuint thisPtr, nuint status, FIoStoreEnvironment* environment)
    {
        LogDebug($"Mounting UTOC {environment->Path} with {environment->Order} priority");
        return _mountUtocHook.OriginalFunction(thisPtr, status, environment);
    }

    private FPakSigningKeys* GetPakSigningKeys()
    {
        // Ensure it's still a dummy key
        // Hi-Fi Rush is special and overwrites it with the actual key at some point lol
        _signingKeys->Function = 0;
        _signingKeys->Size = 0;
        return _signingKeys;
    }

    private void GetPakFolders(nuint cmdLine, TArray<FString>* outPakFolders)
    {
        _getPakFoldersHook.OriginalFunction(cmdLine, outPakFolders);

        // Resize the array
        if (outPakFolders->Capacity <= _pakFolders.Count + outPakFolders->Length)
        {
            outPakFolders->Resize(_pakFolders.Count + outPakFolders->Length);
        }

        // Add files from mods
        foreach (var pakFolder in _pakFolders)
        {
            var str = new FString(pakFolder);
            outPakFolders->Add(str);
        }
    }


    #region Standard Overrides
    public override void ConfigurationUpdated(Config configuration)
    {
        // Apply settings from configuration.
        // ... your code here.
        _configuration = configuration;
        _logger.WriteLine($"[{_modConfig.ModId}] Config Updated: Applying");
    }
    #endregion

    #region For Exports, Serialization etc.
#pragma warning disable CS8618 // Non-nullable field must contain a non-null value when exiting constructor. Consider declaring as nullable.
    public Mod() { }
#pragma warning restore CS8618
    #endregion
}