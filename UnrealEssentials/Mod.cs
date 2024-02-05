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
using Reloaded.Memory.Sigscan.Definitions;
using UTOC.Stream.Emulator.Interfaces;

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
    private IHook<GetPakOrderDelegate> _getPakOrderHook;
    private IHook<PakOpenReadDelegate> _pakOpenReadHook;
    private IHook<PakOpenAsyncReadDelegate> _pakOpenAsyncReadHook;
    private IHook<IsNonPakFilenameAllowedDelegate> _isNonPakFilenameAllowedHook;
    private IHook<FileExistsDlegate> _fileExistsHook;

    private FPakSigningKeys* _signingKeys;
    private string _modsPath;
    private List<string> _pakFolders = new();
    private Dictionary<string, string> _redirections = new();

    private IUtocEmulator _utocEmulator;
    private bool _hasUtocs;

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

        // Setup mods path
        var modPath = new DirectoryInfo(_modLoader.GetDirectoryForModId(_modConfig.ModId));
        _modsPath = modPath.Parent!.FullName;

        // Get Signatures
        var sigs = GetSignatures();
        _hasUtocs = DoesGameUseUtocs(sigs);

        _modLoader.GetController<IUtocEmulator>().TryGetTarget(out _utocEmulator);
        _utocEmulator.Initialise(sigs.TocVersion, sigs.PakVersion, sigs.FileIoStoreOpenContainer, sigs.ReadBlocks, AddPakFolder, RemovePakFolder);

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

        // Fix priority
        SigScan(sigs.GetPakOrder, "GetPakOrder", address =>
        {
            _getPakOrderHook = _hooks.CreateHook<GetPakOrderDelegate>(GetPakOrder, address).Activate();
        });

        // Allow loose pak loading
        SigScan(sigs.PakOpenRead, "PakOpenRead", address =>
        {
            _pakOpenReadHook = _hooks.CreateHook<PakOpenReadDelegate>(PakOpenRead, address).Activate();
        });
        SigScan(sigs.PakOpenAsyncRead, "PakOpenAsyncRead", address =>
        {
            _pakOpenAsyncReadHook = _hooks.CreateHook<PakOpenAsyncReadDelegate>(PakOpenAsyncRead, address).Activate();
        });

        SigScan(sigs.IsNonPakFilenameAllowed, "IsNonPakFilenameAllowed", address =>
        {
            _isNonPakFilenameAllowedHook = _hooks.CreateHook<IsNonPakFilenameAllowedDelegate>(IsNonPakFilenameAllowed, address).Activate();
        });

        SigScan(sigs.FileExists, "FileExists", address =>
        {
            _fileExistsHook = _hooks.CreateHook<FileExistsDlegate>(FileExists, address).Activate();
        });

        // Gather pak files from mods
        //_modLoader.OnModLoaderInitialized += ModLoaderInit;
        _modLoader.ModLoading += ModLoading;
    }

    private bool DoesGameUseUtocs(Signatures sigs)
    {
        if (sigs.TocVersion == null)
        {
            Log("Game does not use UTOCs as TocVersion was null");
        }

        // Look for any utoc files in the game's folder
        if(Directory.GetFiles("../../..", "*.utoc", SearchOption.AllDirectories).Length == 0)
        {
            Log("Game does not include any UTOC files");
            return false;
        }

        return true;
    }
  
    private bool FileExists(nuint thisPtr, char* Filename)
    {
        var fileName = Marshal.PtrToStringUni((nint)Filename);

        if (TryFindLooseFile(fileName, out _))
            return true;

        return _fileExistsHook.OriginalFunction(thisPtr, Filename);
    }

    private bool IsNonPakFilenameAllowed(nuint thisPtr, FString* Filename)
    {
        return true;
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
        _modLoader.GetController<IScannerFactory>().TryGetTarget(out var scannerFactory);
        var scanner = scannerFactory.CreateScanner(CurrentProcess, mainModule);
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

    private int GetPakOrder(FString* PakFilePath)
    {
        // TODO write/copy Contains and StartsWith functions that use the FString* directly
        // instead of making it a string each time (StartsWith is probably much more important)
        var path = PakFilePath->ToString();

        // A vanilla file, use normal order
        if(!path.StartsWith(_modsPath))
            return _getPakOrderHook.OriginalFunction(PakFilePath);
        
        // One of our files, override order
        for(int i = 0; i < _pakFolders.Count; i++)
        {
            if (path.Contains(_pakFolders[i]))
            {
                LogDebug($"Set order of {path} to {(i+1)*1000}");
                return (i + 1) * 10000;
            }
        }

        // This shouldn't happen...
        LogError($"Unable to decide order for {path}. This shouldn't happen!");
        return 0;
    }

    private nuint PakOpenRead(nuint thisPtr, nint fileNamePtr, bool bAllowWrite)
    {
        var fileName = Marshal.PtrToStringUni(fileNamePtr);
        if(_configuration.FileAccessLog)
        {
            Log($"Opening: {fileName}");
        }
        
        // No loose file, vanilla behaviour
        if(!TryFindLooseFile(fileName, out var looseFile))
            return _pakOpenReadHook.OriginalFunction(thisPtr, fileNamePtr, bAllowWrite);

        // Get the pointer to the loose file that UE wants
        Log($"Redirecting {fileName} to {looseFile}");
        var looseFilePtr = Marshal.StringToHGlobalUni(looseFile);
        var res = _pakOpenReadHook.OriginalFunction(thisPtr, looseFilePtr, bAllowWrite);
        
        // Clean up
        Marshal.FreeHGlobal(looseFilePtr); 
        return res;
    }

    private nuint PakOpenAsyncRead(nint thisPtr, nint fileNamePtr)
    {
        var fileName = Marshal.PtrToStringUni(fileNamePtr);
        if (_configuration.FileAccessLog)
        {
            Log($"Opening async: {fileName}");
        }

        // No loose file, vanilla behaviour
        if (!TryFindLooseFile(fileName, out var looseFile))
            return _pakOpenAsyncReadHook.OriginalFunction(thisPtr, fileNamePtr);

        // Get the pointer to the loose file that UE wants
        Log($"Redirecting async {fileName} to {looseFile}");
        var looseFilePtr = Marshal.StringToHGlobalUni(looseFile);
        var res = _pakOpenAsyncReadHook.OriginalFunction(thisPtr, looseFilePtr);

        // Clean up
        //Marshal.FreeHGlobal(looseFilePtr);
        return res;
    }

    private bool TryFindLooseFile(string gameFilePath, out string? looseFile)
    {
        return _redirections.TryGetValue(gameFilePath, out looseFile);
    }

    private void ModLoading(IModV1 mod, IModConfigV1 modConfig)
    {
            var modsPath = Path.Combine(_modLoader.GetDirectoryForModId(modConfig.ModId), "UnrealEssentials");
            if (!Directory.Exists(modsPath))
                return;
            _pakFolders.Add(modsPath);
            AddRedirections(modsPath);
            Log($"Loading files from {modsPath}");

            // Prevent UTOC Emulator from wasting time creating UTOCs if the game doesn't use them
            if(_hasUtocs)
                _utocEmulator.AddFromFolder(modConfig.ModId, modsPath);
        }

    private void AddRedirections(string modsPath)
    {
        foreach (var file in Directory.EnumerateFiles(modsPath, "*", SearchOption.AllDirectories))
        {
            var gamePath = Path.Combine(@"..\..\..", Path.GetRelativePath(modsPath, file)); // recreate what the game would try to load
            _redirections[gamePath] = file;
            _redirections[gamePath.Replace('\\', '/')] = file; // UE could try to load it using either separator
        }
    }

    private void AddPakFolder(string path)
    {
        _pakFolders.Add(path);
        AddRedirections(path);
        Log($"Loading PAK files from {path}");
    }

    private void RemovePakFolder(string path)
    {
        if (_pakFolders.Remove(path))
        {
            Log($"Removed pak folder {path}");
        }
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