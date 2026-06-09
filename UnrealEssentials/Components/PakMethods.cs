using Reloaded.Hooks.Definitions;
using System.Runtime.InteropServices;
using UnrealEssentials.Configuration;
namespace UnrealEssentials.Components;
using static Utils;
using static Unreal.UnrealArray;
using static Unreal.UnrealString;

internal class PakMethods
{
    // State objects
    private readonly IReloadedHooks _hooks;
    private Config _configuration;
    private Context _context;
   
    // Function hooks
    private readonly MultiHook<GetPakFolders> _getPakFoldersHook;
    private readonly MultiHook<GetPakOrder> _getPakOrderHook;
    private readonly MultiHook<PakOpenRead> _pakOpenReadHook;
    private readonly MultiHook<PakOpenAsyncRead> _pakOpenAsyncReadHook;
    private readonly MultiHook<IsNonPakFilenameAllowed> _isNonPakFilenameAllowedHook;
    private readonly MultiHook<FileExists> _fileExistsHook;
    
    public unsafe PakMethods(IReloadedHooks hooks, Config configuration, Context context)
    {
        // Get dependency objects
        _hooks = hooks;
        _configuration = configuration;
        _context = context;
        // Load files from our mod
        _getPakFoldersHook = new("GetPakFolders", 
            context.Properties.Signatures.GetPakFolders, GetPakFoldersImpl);
        // Fix priority
        _getPakOrderHook = new("GetPakOrder", 
            context.Properties.Signatures.GetPakOrder, GetPakOrderImpl);
        // Allow loose pak loading
        _pakOpenReadHook = new("PakOpenRead", 
            context.Properties.Signatures.PakOpenRead, PakOpenReadImpl);
        _pakOpenAsyncReadHook = new("PakOpenAsyncRead", 
            context.Properties.Signatures.PakOpenAsyncRead, PakOpenAsyncReadImpl);
        _isNonPakFilenameAllowedHook = new("IsNonPakFilenameAllowed", 
            context.Properties.Signatures.IsNonPakFilenameAllowed, IsNonPakFilenameAllowedImpl);
        _fileExistsHook = new("FileExists", 
            context.Properties.Signatures.FileExists, FileExistsImpl);
    }
    
    internal unsafe delegate void GetPakFolders(nuint cmdLine, TArray<FString>* outPakFolders);
    private unsafe void GetPakFoldersImpl(nuint cmdLine, TArray<FString>* outPakFolders)
    {
        _getPakFoldersHook.Hook!.OriginalFunction(cmdLine, outPakFolders);
        // Resize the array
        if (outPakFolders->Capacity <= _context!.PakFolders.Count + outPakFolders->Length)
        {
            outPakFolders->Resize(_context!.PakFolders.Count + outPakFolders->Length);
        }

        // Add files from mods
        foreach (var pakFolder in _context!.PakFolders)
        {
            var str = new FString(pakFolder);
            outPakFolders->Add(str);
        }
    }
    internal unsafe delegate int GetPakOrder(FString* PakFilePath);
    private unsafe int GetPakOrderImpl(FString* PakFilePath)
    {
        // TODO write/copy Contains and StartsWith functions that use the FString* directly
        // instead of making it a string each time (StartsWith is probably much more important)
        var path = PakFilePath->ToString();

        // A vanilla file, use normal order
        if (!path.StartsWith(_context.ModsPath))
            return _getPakOrderHook.Hook!.OriginalFunction(PakFilePath);

        // One of our files, override order
        for (int i = 0; i < _context!.PakFolders.Count; i++)
        {
            if (path.Contains(_context!.PakFolders[i]))
            {
                LogDebug($"Set order of {path} to {(i + 1) * 1000}");
                return (i + 1) * 10000;
            }
        }

        // This shouldn't happen...
        LogError($"Unable to decide order for {path}. This shouldn't happen!");
        return 0;
    }
    internal delegate nuint PakOpenRead(nuint thisPtr, nint fileNamePtr, bool bAllowWrite);
    private nuint PakOpenReadImpl(nuint thisPtr, nint fileNamePtr, bool bAllowWrite)
    {
        var fileName = Marshal.PtrToStringUni(fileNamePtr);
        if (_configuration.FileAccessLog)
        {
            Log($"Opening: {fileName}");
        }

        // No loose file, vanilla behaviour
        if (!_context.TryFindLooseFile(fileName, out var looseFile))
            return _pakOpenReadHook.Hook!.OriginalFunction(thisPtr, fileNamePtr, bAllowWrite);

        // Get the pointer to the loose file that UE wants
        Log($"Redirecting {fileName} to {looseFile}");
        var looseFilePtr = Marshal.StringToHGlobalUni(looseFile);
        var res = _pakOpenReadHook.Hook!.OriginalFunction(thisPtr, looseFilePtr, bAllowWrite);

        // Clean up
        Marshal.FreeHGlobal(looseFilePtr);
        return res;
    }
    
    internal delegate nuint PakOpenAsyncRead(nint thisPtr, nint fileNamePtr);
    private nuint PakOpenAsyncReadImpl(nint thisPtr, nint fileNamePtr)
    {
        var fileName = Marshal.PtrToStringUni(fileNamePtr);
        if (_configuration.FileAccessLog)
        {
            Log($"Opening async: {fileName}");
        }

        // No loose file, vanilla behaviour
        if (!_context.TryFindLooseFile(fileName, out var looseFile))
            return _pakOpenAsyncReadHook.Hook!.OriginalFunction(thisPtr, fileNamePtr);

        // Get the pointer to the loose file that UE wants
        Log($"Redirecting async {fileName} to {looseFile}");
        var looseFilePtr = Marshal.StringToHGlobalUni(looseFile);
        var res = _pakOpenAsyncReadHook.Hook!.OriginalFunction(thisPtr, looseFilePtr);

        // Clean up
        //Marshal.FreeHGlobal(looseFilePtr);
        return res;
    }
    internal unsafe delegate bool IsNonPakFilenameAllowed(nuint thisPtr, FString* Filename);
    private unsafe bool IsNonPakFilenameAllowedImpl(nuint thisPtr, FString* Filename)
    {
        return true;
    }
    internal unsafe delegate bool FileExists(nuint thisPtr, char* Filename);
    private unsafe bool FileExistsImpl(nuint thisPtr, char* Filename)
    {
        var fileName = Marshal.PtrToStringUni((nint)Filename);

        if (_context.TryFindLooseFile(fileName, out _))
            return true;

        return _fileExistsHook.Hook!.OriginalFunction(thisPtr, Filename);
    }
}