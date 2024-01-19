using Reloaded.Hooks.Definitions;
using System.Runtime.InteropServices;
using static UnrealEssentials.Utils;

namespace UnrealEssentials.Unreal;
internal unsafe class UnrealMemory
{
    private static FMalloc** _gMalloc;
    private static IReloadedHooks _hooks;

    // FMalloc Functions
    private static MallocDelegate? _malloc;
    private static TryMallocDelegate? _tryMalloc;
    private static ReallocDelegate? _realloc;
    private static TryReallocDelegate? _tryRealloc;
    private static FreeDelegate? _free;
    private static QuantizeSizeDelegate? _quantizeSize;
    private static GetAllocationSizeDelegate? _getAllocationSize;
    private static TrimDelegate? _trim;

    internal static void InitialiseGMalloc(string sig, IReloadedHooks hooks)
    {
        _hooks = hooks;
        SigScan(sig, "GMallocPtr", address =>
        {
            _gMalloc = (FMalloc**)GetGlobalAddress(address + 3);
            LogDebug($"Found GMalloc at 0x{(nuint)_gMalloc:X}");
        });
    }

    private static void SetupWrappers()
    {
        // We shouldn't do allocation stuff before the game's setup
        // If we actually need to at some point we'll need to make GMalloc ourselves by hooking FMemory::GCreateMalloc
        if(*_gMalloc == null)
        {
            throw new Exception("GMalloc has not been initialised yet, please report this!");
        }

        FMallocVTable* vTable = (*_gMalloc)->VTable;
        _malloc = _hooks.CreateWrapper<MallocDelegate>((long)vTable->Malloc, out _);
        _tryMalloc = _hooks.CreateWrapper<TryMallocDelegate>((long)vTable->TryMalloc, out _);
        _realloc = _hooks.CreateWrapper<ReallocDelegate>((long)vTable->Realloc, out _);
        _tryRealloc = _hooks.CreateWrapper<TryReallocDelegate>((long)vTable->TryRealloc, out _);
        _free = _hooks.CreateWrapper<FreeDelegate>((long)vTable->Free, out _);
        _quantizeSize = _hooks.CreateWrapper<QuantizeSizeDelegate>((long)vTable->QuantizeSize, out _);
        _getAllocationSize = _hooks.CreateWrapper<GetAllocationSizeDelegate>((long)vTable->GetAllocationSize, out _);
        _trim = _hooks.CreateWrapper<TrimDelegate>((long)vTable->Trim, out _);
    }

    // Wrappers for GMalloc functions
    internal static void* Malloc(nuint Count, uint Alignment = DEFAULT_ALIGNMENT)
    {
        if (_malloc == null)
            SetupWrappers();

        return _malloc!(*_gMalloc, Count, Alignment);
    }

    internal static void* TryMalloc(nuint Count, uint Alignment = DEFAULT_ALIGNMENT)
    {
        if (_tryMalloc == null)
            SetupWrappers();

        return _tryMalloc!(*_gMalloc, Count, Alignment);
    }

    internal static void* Realloc(void* Original, nuint Count, uint Alignment = DEFAULT_ALIGNMENT)
    {
        if (_realloc == null)
            SetupWrappers();

        return _realloc!(*_gMalloc, Original, Count, Alignment);
    }

    internal static void* TryRealloc(void* Original, nuint Count, uint Alignment = DEFAULT_ALIGNMENT)
    {
        if (_tryRealloc == null)
            SetupWrappers();

        return _tryRealloc!(*_gMalloc, Original, Count, Alignment);
    }

    internal static void Free(void* Original)
    {
        if (_free == null)
            SetupWrappers();

        _free!(*_gMalloc, Original);
    }

    internal static nuint QuantizeSize(nuint Count, uint Alignment)
    {
        if (_quantizeSize == null)
            SetupWrappers();

        return _quantizeSize!(*_gMalloc, Count, Alignment);
    }

    internal static bool GetAllocationSize(void* Original, nuint* SizeOut)
    {
        if (_getAllocationSize == null)
            SetupWrappers();

        return _getAllocationSize!(*_gMalloc, Original, SizeOut);
    }

    internal static void Trim(bool bTrimThreadCaches)
    {
        if (_trim == null)
            SetupWrappers();

        _trim!(*_gMalloc, bTrimThreadCaches);
    }

    // Structur definitions
    internal struct FMalloc
    {
        internal FMallocVTable* VTable;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct FMallocVTable
    {
        internal nuint __vec_del_dtor;
        internal nuint exec;
        internal nuint Malloc;
        internal nuint TryMalloc;
        internal nuint Realloc;
        internal nuint TryRealloc;
        internal nuint Free;
        internal nuint QuantizeSize;
        internal nuint GetAllocationSize;
        internal nuint Trim;
    }

    // Memory Delegates
    private const int DEFAULT_ALIGNMENT = 0;

    internal delegate void* MallocDelegate(FMalloc* gMalloc, nuint Count, uint Alignment = DEFAULT_ALIGNMENT);
    internal delegate void* TryMallocDelegate(FMalloc* gMalloc, nuint Count, uint Alignment = DEFAULT_ALIGNMENT);
    internal delegate void* ReallocDelegate(FMalloc* gMalloc, void* Original, nuint Count, uint Alignment = DEFAULT_ALIGNMENT);
    internal delegate void* TryReallocDelegate(FMalloc* gMalloc, void* Original, nuint Count, uint Alignment = DEFAULT_ALIGNMENT);
    internal delegate void FreeDelegate(FMalloc* gMalloc, void* Original);
    internal delegate nuint QuantizeSizeDelegate(FMalloc* gMalloc, nuint Count, uint Alignment);
    internal delegate bool GetAllocationSizeDelegate(FMalloc* gMalloc, void* Original, nuint* SizeOut);
    internal delegate void TrimDelegate(FMalloc* gMalloc, bool bTrimThreadCaches);

    // We really don't need any of these, leaving as a comment in case there's a use in the future (I doubt it)
    //private delegate void SetupTLSCachesOnCurrentThread();
    //private delegate void ClearAndDisableTLSCachesOnCurrentThread();
    //private delegate void InitializeStatsMetadata();
    //private delegate void UpdateStats();
    //private delegate void GetAllocatorStats(FGenericMemoryStats& out_Stats);
    //private delegate void DumpAllocatorStats(class FOutputDevice& Ar);
    //private delegate bool IsInternallyThreadSafe() const;
    //private delegate bool ValidateHeap();
    //private delegate const TCHAR* GetDescriptiveName();

}
