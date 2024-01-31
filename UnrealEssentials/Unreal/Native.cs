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

    internal struct FFileIoStoreBuffer
    {
        internal FFileIoStoreBuffer* Next;
        internal byte* Memory;
    }

    internal struct FFileIoStoreReadRequest
    {
        internal FFileIoStoreReadRequest* Next;
        internal nuint FileHandle;
        internal nuint Offset;
        internal nuint Size;
        internal nuint Key; // File Index + Block Index
        internal FFileIoStoreBuffer* Buffer;
    }

    internal struct FFileIoStoreReadRequestLink
    {
        internal FFileIoStoreReadRequestLink* Next;
        internal FFileIoStoreReadRequest* Request;
    }

    internal struct FFileIoStoreResolvedRequest
    {
        internal nuint DispatcherRequest;
        internal nuint ContainerFile;
        internal FFileIoStoreReadRequestLink* ReadRequestsHead;
        internal FFileIoStoreReadRequestLink* ReadRequestsTail;
        internal long ResolvedOffset;
        internal long ResolvedSize;
        internal uint ContainerFileIndex;
    }

    internal delegate FPakSigningKeys* GetPakSigningKeysDelegate();
    internal delegate void GetPakFoldersDelegate(nuint cmdLine, TArray<FString>* outPakFolders);
    internal delegate nuint IoDispatcherMountDelegate(nuint thisPtr, nuint status, FIoStoreEnvironment* environment);
    internal delegate bool PakPlatformFileMountDelegate(nuint thisPtr, char* InPakFilename, int PakOrder, char* InPath, bool bLoadIndex);
    internal delegate void FindAllPakFilesDelegate(nuint LowerLevelFile, TArray<FString>* PakFolders, FString* WildCard, TArray<FString>* OutPakFiles);
    internal delegate int GetPakOrderDelegate(FString* PakFilePath);
    internal delegate nuint PakOpenReadDelegate(nuint thisPtr, nint fileNamePtr, bool bAllowWrite);
    internal delegate void FFileIoStore_ReadBlocks(nuint thisPtr, FFileIoStoreResolvedRequest* requestPtr);
}
