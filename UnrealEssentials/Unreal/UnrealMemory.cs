using Reloaded.Hooks.Definitions;
using System.Runtime.InteropServices;
using UnrealEssentials.Interfaces;
using static UnrealEssentials.Utils;

namespace UnrealEssentials.Unreal;
public unsafe class UnrealMemory : IUnrealMemory
{
    private FMalloc** _gMalloc;
    private IReloadedHooks _hooks;

    // FMalloc Functions
    private MallocDelegate? _malloc;
    private TryMallocDelegate? _tryMalloc;
    private ReallocDelegate? _realloc;
    private TryReallocDelegate? _tryRealloc;
    private FreeDelegate? _free;
    private QuantizeSizeDelegate? _quantizeSize;
    private GetAllocationSizeDelegate? _getAllocationSize;
    private TrimDelegate? _trim;
    
    private static MultiSignature GMallocSignature;

    private readonly bool _allowExecuteCommands;
    private readonly ObjectCommandExecutorType _commandExecutorType;

    internal UnrealMemory(Signatures sigs, IReloadedHooks hooks, bool allowExecuteCommands, 
        ObjectCommandExecutorType commandExecutorType)
    {
        _hooks = hooks;
        _allowExecuteCommands = allowExecuteCommands;
        _commandExecutorType = commandExecutorType;
        GMallocSignature = new("GMalloc", sigs.GMalloc, address => _gMalloc = (FMalloc**)address);
    }

    private void SetupWrappers()
    {
        // We shouldn't do allocation stuff before the game's setup
        // If we actually need to at some point we'll need to make GMalloc ourselves by hooking FMemory::GCreateMalloc
        if(*_gMalloc == null)
        {
            throw new Exception("GMalloc has not been initialised yet, please report this!");
        }

        FMallocVtable vtable = new((*_gMalloc)->vtable, _allowExecuteCommands, _commandExecutorType);
        _malloc = _hooks.CreateWrapper<MallocDelegate>((long)vtable.Malloc(), out _);
        _tryMalloc = _hooks.CreateWrapper<TryMallocDelegate>((long)vtable.TryMalloc(), out _);
        _realloc = _hooks.CreateWrapper<ReallocDelegate>((long)vtable.Realloc(), out _);
        _tryRealloc = _hooks.CreateWrapper<TryReallocDelegate>((long)vtable.TryRealloc(), out _);
        _free = _hooks.CreateWrapper<FreeDelegate>((long)vtable.Free(), out _);
        _quantizeSize = _hooks.CreateWrapper<QuantizeSizeDelegate>((long)vtable.QuantizeSize(), out _);
        _getAllocationSize = _hooks.CreateWrapper<GetAllocationSizeDelegate>((long)vtable.GetAllocationSize(), out _);
        _trim = _hooks.CreateWrapper<TrimDelegate>((long)vtable.Trim(), out _);
    }

    // Wrappers for GMalloc functions
    public nuint Malloc(nuint count, uint alignment = DEFAULT_ALIGNMENT)
    {
        if (_malloc == null)
            SetupWrappers();

        return _malloc!(*_gMalloc, count, alignment);
    }

    public nuint TryMalloc(nuint count, uint alignment = DEFAULT_ALIGNMENT)
    {
        if (_tryMalloc == null)
            SetupWrappers();

        return _tryMalloc!(*_gMalloc, count, alignment);
    }

    public nuint Realloc(nuint original, nuint count, uint alignment = DEFAULT_ALIGNMENT)
    {
        if (_realloc == null)
            SetupWrappers();

        return _realloc!(*_gMalloc, original, count, alignment);
    }

    public nuint TryRealloc(nuint original, nuint count, uint alignment = DEFAULT_ALIGNMENT)
    {
        if (_tryRealloc == null)
            SetupWrappers();

        return _tryRealloc!(*_gMalloc, original, count, alignment);
    }

    public void Free(nuint original)
    {
        if (_free == null)
            SetupWrappers();

        _free!(*_gMalloc, original);
    }

    internal nuint QuantizeSize(nuint count, uint alignment)
    {
        if (_quantizeSize == null)
            SetupWrappers();

        return _quantizeSize!(*_gMalloc, count, alignment);
    }

    public bool GetAllocationSize(nuint original, out nuint size)
    {
        if (_getAllocationSize == null)
            SetupWrappers();

        size = 0;
        fixed (nuint* sizePtr = &size )
        {   
            return _getAllocationSize!(*_gMalloc, original, sizePtr);
        }
    }

    internal void Trim(bool bTrimThreadCaches)
    {
        if (_trim == null)
            SetupWrappers();

        _trim!(*_gMalloc, bTrimThreadCaches);
    }

    // Structure definitions
    internal struct FMalloc
    {
        internal nint vtable;
    }

    // Memory Delegates
    private const int DEFAULT_ALIGNMENT = 0;

    internal delegate nuint MallocDelegate(FMalloc* gMalloc, nuint Count, uint Alignment = DEFAULT_ALIGNMENT);
    internal delegate nuint TryMallocDelegate(FMalloc* gMalloc, nuint Count, uint Alignment = DEFAULT_ALIGNMENT);
    internal delegate nuint ReallocDelegate(FMalloc* gMalloc, nuint Original, nuint Count, uint Alignment = DEFAULT_ALIGNMENT);
    internal delegate nuint TryReallocDelegate(FMalloc* gMalloc, nuint Original, nuint Count, uint Alignment = DEFAULT_ALIGNMENT);
    internal delegate void FreeDelegate(FMalloc* gMalloc, nuint Original);
    internal delegate nuint QuantizeSizeDelegate(FMalloc* gMalloc, nuint Count, uint Alignment);
    internal delegate bool GetAllocationSizeDelegate(FMalloc* gMalloc, nuint Original, nuint* SizeOut);
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

internal unsafe class FMallocVtable
{
    private readonly nint _vtable;
    private readonly bool _allowExecuteCommands;
    private readonly ObjectCommandExecutorType _commandExecutorType;

    internal FMallocVtable(nint vtable, bool allowExecuteCommands, ObjectCommandExecutorType commandExecutorType)
    {
        _vtable = vtable;
        _allowExecuteCommands = allowExecuteCommands;
        _commandExecutorType = commandExecutorType;
    }

    private nint GetBase()
    {
        var Base = _vtable + 0x10;
        if (!_allowExecuteCommands)
        {
            Base += 0x8;
        }
        switch (_commandExecutorType)
        {
            case ObjectCommandExecutorType.AddDevEditor:
                Base += 0x10;
                break;
            case ObjectCommandExecutorType.AddRuntime:
                Base += 0x18;
                break;
        }
        LogDebug($"{_vtable:x} -> {Base:x} ({_commandExecutorType})");
        return Base;
    }
    
    internal nuint Malloc() => *(nuint*)GetBase();
    internal nuint TryMalloc() => *(nuint*)(GetBase() + 0x8);
    internal nuint Realloc() => *(nuint*)(GetBase() + 0x10);
    internal nuint TryRealloc() => *(nuint*)(GetBase() + 0x18);
    internal nuint Free() => *(nuint*)(GetBase() + 0x20);
    internal nuint QuantizeSize() => *(nuint*)(GetBase() + 0x28);
    internal nuint GetAllocationSize() => *(nuint*)(GetBase() + 0x30);
    internal nuint Trim() => *(nuint*)(GetBase() + 0x38);
}