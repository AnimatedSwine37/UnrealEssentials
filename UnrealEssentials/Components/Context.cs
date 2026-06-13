using System.Diagnostics;
using System.Reflection;
using System.Reflection.Metadata.Ecma335;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using Reloaded.Memory.Sigscan.Definitions;
using Reloaded.Mod.Interfaces;
using UnrealEssentials.Types;
using UnrealEssentials.Unreal;
using UTOC.Stream.Emulator.Interfaces;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace UnrealEssentials.Components;
using static Utils;

internal static class ContextBuilder
{
    public static unsafe Context? CreateContext(IModLoader _modLoader, IModConfig _modConfig)
    {
        var modFolder = _modLoader.GetDirectoryForModId(_modConfig.ModId);
        var modPath = new DirectoryInfo(modFolder).Parent!.FullName;
        var propFactory = new SignaturePropertyFactory(Path.Combine(modFolder, "Signatures"));
        if (!TryGetProperties(_modLoader, propFactory, out var props)) return null;
        Log($"Engine version is {props.EngineVersion.ToBranchVersion()}");
        return new(Native.FPakSigningKeys.NewBlank(), modPath, CheckIoStore(props), props, _modLoader);   
    }
    
    private const string RootBlock = "\\";
    private const string TranslateBlock = "\\VarFileInfo\\Translation";

    private static bool GetFileVersionInfo(nint winVerDll, ProcessModule mainModule, out byte[] infoBuffer)
    {
        infoBuffer = [];
        unsafe
        {
            var getFileVersionInfoSizeA = (delegate* unmanaged[Stdcall]<string, uint*, uint>)Imports.GetProcAddress(
                winVerDll, "GetFileVersionInfoSizeA");
            if (getFileVersionInfoSizeA == null) return false;
            var infoSize = getFileVersionInfoSizeA(mainModule.FileName, null);
            infoBuffer = new byte[infoSize];
            var getFileVersionInfoA = (delegate* unmanaged[Stdcall]<string, uint, uint, byte*, bool>)Imports.GetProcAddress(
                winVerDll, "GetFileVersionInfoA");
            if (getFileVersionInfoA == null) return false;
            fixed (byte* pInfoBuffer = infoBuffer)
                if (!getFileVersionInfoA(mainModule.FileName, 0, infoSize, pInfoBuffer))
                    return false;
            return true;
        }
    }

    private delegate bool TryGetSignatureFromPropertyCallback(string nameDesc, 
        SignaturePropertyFactory factory, out Properties? sigs);
    
    private static bool TryGetSignatureFromStringProperty(nint winVerDll, SignaturePropertyFactory factory, 
        byte[] infoBuffer, string property, TryGetSignatureFromPropertyCallback callback, out Properties? sigs)
    {
        sigs = null;
        unsafe
        {
            fixed (byte* pInfoBuffer = infoBuffer)
            {
                // https://learn.microsoft.com/en-us/windows/win32/api/winver/nf-winver-verqueryvaluea
                var verQueryValueA = (delegate* unmanaged[Stdcall]<byte*, string, nint*, uint*, bool>)
                    Imports.GetProcAddress(winVerDll, "VerQueryValueA");
                if (verQueryValueA == null) return false;
                // Get language + codepage
                LanguageCodePage* translate = null;
                uint translateSize = 0;
                if (!verQueryValueA(pInfoBuffer, TranslateBlock, (nint*)(&translate), &translateSize)) return false;
                for (var i = 0; i < translateSize / sizeof(LanguageCodePage); i++)
                {
                    // Check FileDescription entry for StringFileInfo
                    var translateEntry = translate + i;
                    char* fileDescription = null;
                    uint fileDescBytes = 0;
                    // VerQueryValue for strings includes null terminator in length
                    if (!verQueryValueA(pInfoBuffer,
                            $"\\StringFileInfo\\{translateEntry->wLanguage:x04}{translateEntry->wCodePage:x04}\\{property}",
                            (nint*)(&fileDescription), &fileDescBytes))
                        return false;
                    var nameDesc = Marshal.PtrToStringAnsi((nint)fileDescription, (int)fileDescBytes - 1);
                    if (callback(nameDesc, factory, out var sigsMaybe))
                    {
                        sigs = sigsMaybe;
                        return true;
                    }
                }
            }
        }
        return false;
    }

    private static bool TryGetSignatureFromProductNameCallback(string nameDesc, 
        SignaturePropertyFactory factory, out Properties? sigs)
        => factory.GameRegistry.ProductName.TryGetValue(nameDesc, out sigs);

    private static bool TryGetSignatureFromFileVersion(nint winVerDll, SignaturePropertyFactory factory,
        byte[] infoBuffer, out Properties? sigs)
    {
        sigs = null;
        unsafe
        {
            fixed (byte* pInfoBuffer = infoBuffer)
            {
                // https://learn.microsoft.com/en-us/windows/win32/api/winver/nf-winver-verqueryvaluea
                var verQueryValueA = (delegate* unmanaged[Stdcall]<byte*, string, nint*, uint*, bool>)
                    Imports.GetProcAddress(winVerDll, "VerQueryValueA");
                if (verQueryValueA == null) return false;
                FixedFileInfo* root = null;
                uint rootSize = 0;
                if (!verQueryValueA(pInfoBuffer, RootBlock, (nint*)(&root), &rootSize) 
                    || root->dwSignature != 0xfeef04bd) return false;
                var major = root->dwFileVersionMS >> 0x10;
                var minor = root->dwFileVersionMS & 0xffff;
                var engineVer = $"++UE{major}+Release-{major}.{minor}";
                if (factory.EngineVersions.TryGetValue(engineVer, out var sigsMaybe))
                {
                    sigs = sigsMaybe;
                    return true;
                }
            }
        }
        return false;
    }

    private static string[] BranchNames =
    [
        "2B 00 2B 00 55 00 45 00 34 00 2B 00", // ++UE4+
        "2B 00 2B 00 75 00 65 00 34 00 2B 00", // ++ue4+
        "2B 00 2B 00 55 00 45 00 35 00 2B 00", // ++UE5+
        "2B 00 2B 00 75 00 65 00 35 00 2B 00", // ++ue5+
    ];
    
    private static bool TryGetProperties(IModLoader _modLoader, SignaturePropertyFactory factory, out Properties props)
    {
        var CurrentProcess = Process.GetCurrentProcess();
        var mainModule = CurrentProcess.MainModule;
        var fileName = Path.GetFileNameWithoutExtension(mainModule!.FileName);
        props = new();
        // Try and find based on file name
        if (factory.GameRegistry.ExecutableName.TryGetValue(fileName, out props)) return true;
        // Dynamically load DLL needed for methods to get executable resource metadata from
        var winVerDll = Imports.LoadLibraryA("Api-ms-win-core-version-l1-1-0.dll");
        if (winVerDll != nint.Zero && GetFileVersionInfo(winVerDll, mainModule, out var infoBuffer))
        {
            // Try and find based on the executable's file description
            if (TryGetSignatureFromStringProperty(winVerDll, factory, infoBuffer,
                    "ProductName", TryGetSignatureFromProductNameCallback, out props)) return true;
            // Try and find based on the file version of the executable (this is more common in UE5 games)
            if (TryGetSignatureFromFileVersion(winVerDll, factory, infoBuffer, out props)) return true;
        }
        else
        {
            LogError("Could not locate the DLL \"Api-ms-win-core-version-l1-1-0.dll\" \n" +
                     "We won't be able to determine the engine version using info from the executable properties!\n");   
        }
        // Try and find based on branch name
        _modLoader.GetController<IScannerFactory>().TryGetTarget(out var scannerFactory);
        var scanner = scannerFactory!.CreateScanner(CurrentProcess, mainModule);
        var results = scanner.FindPatterns(BranchNames).Where(x => x.Found).ToList();
        if (results.Count == 0)
        {
            LogError($"Unable to find Unreal Engine version number, Unreal Essentials will not work!\n" +
                     $"If this game does not use Unreal Engine please disable Unreal Essentials.\n" +
                     $"If you are sure this is an Unreal Engine game then please report this at github.com/AnimatedSwine37/UnrealEssentials " +
                     $"so support can be added.");
            return false;
        }
        var branch = Marshal.PtrToStringUni(results[0].Offset + BaseAddress)!;
        if (factory.EngineVersions.TryGetValue(branch, out props)) return true;
        LogError($"Unable to find signatures for Unreal Engine branch {branch}, Unreal Essentials will not work!\n" +
                 "Please report this at github.com/AnimatedSwine37/UnrealEssentials.");
        return false;
    }
    
    private static bool CheckIoStore(Properties props)
    {
        // props.TocVersion
        if (((uint)props.EngineVersion & ushort.MaxValue) < (uint)EngineVersion.UE_4_25)
        {
            Log($"Game does not use UTOCs, EngineVersion is too old ({props.EngineVersion})");
            return false;
        }
        // Look for any utoc files in the game's folder
        if (Directory.GetFiles("../../..", "*.utoc", SearchOption.AllDirectories).Length == 0)
        {
            Log("Game does not include any UTOC files");
            return false;
        }

        return true;
    }
}

internal class Context
{
    internal unsafe Native.FPakSigningKeys* SigningKeys { get;}
    internal string ModsPath { get; }
    internal List<string> PakFolders = [];
    internal Dictionary<string, string> Redirections { get; } = [];
    internal IUtocEmulator? UtocEmulator;
    internal bool HasUtocs { get; }
    internal Properties Properties { get; }
    
    internal unsafe Context(Native.FPakSigningKeys* signingKeys, string modsPath, bool hasUtocs, Properties properties, IModLoader _modLoader)
    {
        SigningKeys = signingKeys;
        ModsPath = modsPath;
        HasUtocs = hasUtocs;
        Properties = properties;
       
        // Initialize UTOC Emulator
        _modLoader.GetController<IUtocEmulator>().TryGetTarget(out UtocEmulator);
        UtocEmulator!.Initialise(Properties.EngineVersion, HasUtocs, AddPakFolder, RemovePakFolder);
    }

    internal void AddFolder(string folder)
    {
        if (!Directory.Exists(folder))
        {
            LogError($"Folder {folder} does not exist, skipping.");
            return;
        }
        PakFolders.Add(folder);
        AddRedirections(folder, null);
        Log($"Loading files from {folder}");

        // Prevent UTOC Emulator from wasting time creating UTOCs if the game doesn't use them
        if (HasUtocs)
            UtocEmulator.AddFromFolder(folder);
    }

    internal void AddFolderWithVirtualMount(string folder, string virtualPath)
    {
        if (!Directory.Exists(folder))
        {
            LogError($"Folder {folder} does not exist, skipping.");
            return;
        }
        PakFolders.Add(folder);
        AddRedirections(folder, virtualPath);
        Log($"Loading files from {folder}, with emulated mountFilePath {virtualPath}");

        // Prevent UTOC Emulator from wasting time creating UTOCs if the game doesn't use them
        if (HasUtocs)
            UtocEmulator.AddFromFolderWithMount(folder, virtualPath);
    }

    internal void AddFileWithVirtualMount(string file, string virtualPath)
    {
        if(!File.Exists(file))
        {
            LogError($"File {file} does not exist, skipping.");
            return;
        }
        PakFolders.Add(file);
        Redirections[virtualPath] = file;
        Log($"Loading file at {file}, with emulated mountFilePath {virtualPath}");

        // Prevent UTOC Emulator from wasting time creating UTOCs if the game doesn't use them
        if (HasUtocs)
            UtocEmulator.AddFromFolderWithMount(file, virtualPath);
    }

    internal void AddRedirections(string modsPath, string? virtualPath)
    {
        foreach (var file in Directory.EnumerateFiles(modsPath, "*", SearchOption.AllDirectories))
        {
            string relativeFilePath = Path.GetRelativePath(modsPath, file);
            string gamePath;

            if (!string.IsNullOrWhiteSpace(virtualPath))
            {
                // Use virtual mount mountFilePath
                gamePath = Path.Combine(@"..\..\..", virtualPath, relativeFilePath);
            }
            else
            {
                gamePath = Path.Combine(@"..\..\..", relativeFilePath);
            }

            string normalizedGamePath = gamePath.Replace('\\', '/');
            Redirections[gamePath] = file;
            Redirections[normalizedGamePath] = file;
        }
    }
    
    private void AddPakFolder(string path)
    {
        PakFolders.Add(path);
        AddRedirections(path, null);
        Log($"Loading PAK files from {path}");
    }

    private void RemovePakFolder(string path)
    {
        if (PakFolders.Remove(path))
        {
            Log($"Removed pak folder {path}");
        }
    }
    
    internal bool TryFindLooseFile(string gameFilePath, out string? looseFile)
    {
        return Redirections.TryGetValue(gameFilePath, out looseFile);
    }

    internal void LoadUEMounts(string modRootPath, string mountFilePath)
    {
        if (File.Exists(mountFilePath))
        {
            Log($"Loading virtual paths from {mountFilePath}.");
            List<VirtualEntry> virtualPaths = new DeserializerBuilder()
            .WithNamingConvention(UnderscoredNamingConvention.Instance).WithEnforceRequiredMembers()
            .Build().Deserialize<List<VirtualEntry>>(File.ReadAllText(mountFilePath));
            foreach (var item in virtualPaths)
            {
                if (File.Exists(item.OSPath))
                {
                    AddFileWithVirtualMount(Path.Combine(modRootPath, item.OSPath), item.VirtualPath);
                }
                else if (Directory.Exists(item.OSPath))
                {
                    AddFolderWithVirtualMount(Path.Combine(modRootPath, item.OSPath), item.VirtualPath);
                }
                else
                {
                    LogError($"OSPath: {item.OSPath} supplied in {mountFilePath} does not exist!");
                }
            }
        }
    }
}