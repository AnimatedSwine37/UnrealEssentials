using System.Runtime.InteropServices;
using static UnrealEssentials.Unreal.UnrealArray;
using static UnrealEssentials.Unreal.UnrealString;

namespace UnrealEssentials.Unreal;
internal unsafe class Native
{
    /// <summary>
    /// This isn't neccessarily accurate to Unreal Engine source, 
    /// it's just good enough for removing signatures
    /// </summary>
    internal struct FPakSigningKeys
    {
        internal nuint Function;
        internal int Size;
    }

    internal struct FIoStoreEnvironment
    {
        internal FString Path;
        internal int Order;
    }

    internal delegate FPakSigningKeys* GetPakSigningKeysDelegate();
    internal delegate void GetPakFoldersDelegate(nuint cmdLine, TArray<FString>* outPakFolders);
    internal delegate nuint IoDispatcherMountDelegate(nuint thisPtr, nuint status, FIoStoreEnvironment* environment);
    internal delegate bool PakPlatformFileMountDelegate(nuint thisPtr, char* InPakFilename, int PakOrder, char* InPath, bool bLoadIndex);
    internal delegate void FindAllPakFilesDelegate(nuint LowerLevelFile, TArray<FString>* PakFolders, FString* WildCard, TArray<FString>* OutPakFiles);
    internal delegate int GetPakOrderDelegate(FString* PakFilePath);
    internal delegate nuint PakOpenReadDelegate(nuint thisPtr, nint fileNamePtr, bool bAllowWrite);
}
