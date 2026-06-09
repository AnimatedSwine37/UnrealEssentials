using Reloaded.Hooks.Definitions;
using UnrealEssentials.Configuration;
using UnrealEssentials.Unreal;
namespace UnrealEssentials.Components;
using static Utils;

internal class UtocMethods
{
    // State objects
    private readonly IReloadedHooks _hooks;
    private Config _config;
    private Context _context;
    
    private readonly MultiHook<GetPakSigningKeys> _getSigningKeysHook;
    private readonly MultiHook<FAsyncPackage2_StartLoading0> _startLoading0;
    private readonly MultiHook<FAsyncPackage2_StartLoading1> _startLoading1;
    private readonly MultiHook<FAsyncPackage2_StartLoading2> _startLoading2;
    private readonly MultiHook<FAsyncPackage2_StartLoading3> _startLoading3;
    private readonly MultiHook<FAsyncPackage2_StartLoading4> _startLoading4;

    private const string PackageStartLoading = "FAsyncPackage2::StartLoading";

    public unsafe UtocMethods(IReloadedHooks hooks, Config config, Context context)
    {
        // Get dependency objects
        _hooks = hooks;
        _config = config;
        _context = context;
        // Remove utoc signing
        _getSigningKeysHook = new("GetPakSigningKeys", context.Properties.Signatures.GetPakSigningKeys, GetPakSigningKeysImpl);
        switch (_context.Properties.StartLoadDelegate)
        {
         
            case StartLoadingDelegateType.NoArgs:
                _startLoading0 = new(PackageStartLoading, context.Properties.Signatures.FAsyncPackage2_StartLoading, FAsyncPackage2_StartLoading0Impl);
                break;
            case StartLoadingDelegateType.AddIoBatch:
                _startLoading1 = new(PackageStartLoading, context.Properties.Signatures.FAsyncPackage2_StartLoading, FAsyncPackage2_StartLoading1Impl);
                break;
            case StartLoadingDelegateType.PackageNodeArray:
                _startLoading2 = new(PackageStartLoading, context.Properties.Signatures.FAsyncPackage2_StartLoading, FAsyncPackage2_StartLoading2Impl);
                break;
            case StartLoadingDelegateType.AddThreadState:
                _startLoading3 = new(PackageStartLoading, context.Properties.Signatures.FAsyncPackage2_StartLoading, FAsyncPackage2_StartLoading3Impl);
                break;
            case StartLoadingDelegateType.DescAddInstancingContext:
                _startLoading4 = new(PackageStartLoading, context.Properties.Signatures.FAsyncPackage2_StartLoading, FAsyncPackage2_StartLoading4Impl);
                break;
        }
    }
    
    internal unsafe delegate Native.FPakSigningKeys* GetPakSigningKeys();
    private unsafe Native.FPakSigningKeys* GetPakSigningKeysImpl()
    {
        // Ensure it's still a dummy key
        // Hi-Fi Rush is special and overwrites it with the actual key at some point lol
        _context.SigningKeys->Function = 0;
        _context.SigningKeys->Size = 0;
        return _context.SigningKeys;
    }
    
    internal unsafe delegate void FAsyncPackage2_StartLoading0(Native.FAsyncPackage2* Self);
    private unsafe void FAsyncPackage2_StartLoading0Impl(Native.FAsyncPackage2* Self) 
    {
        var DiskName = Self->DiskPackageName;
        var ChunkId = new Native.FIoChunkId(Self->DiskPackageId, 0, 2);
        if (!DiskName.IsNone() && _config.FileAccessLog)
        {
            Log($"StartLoading: {DiskName}");    
        }
        _startLoading0.Hook!.OriginalFunction(Self);
    }
    
    internal unsafe delegate void FAsyncPackage2_StartLoading1(Native.FAsyncPackage2_UE5_0* Self, nint IoBatch);
    private unsafe void FAsyncPackage2_StartLoading1Impl(Native.FAsyncPackage2_UE5_0* Self, nint IoBatch)
    {
        var DiskName = Self->PackagePathToLoad;
        if (!DiskName.IsNone() && _config.FileAccessLog)
        {
            Log($"StartLoading: {DiskName}");    
        }
        _startLoading1.Hook!.OriginalFunction(Self, IoBatch);
    }
    
    internal unsafe delegate void FAsyncPackage2_StartLoading2(Native.FAsyncPackage2_UE5_1* Self, nint IoBatch);
    private unsafe void FAsyncPackage2_StartLoading2Impl(Native.FAsyncPackage2_UE5_1* Self, nint IoBatch) 
    {
        var DiskName = Self->PackagePathToLoad;
        if (!DiskName.IsNone() && _config.FileAccessLog)
        {
            Log($"StartLoading: {DiskName}");    
        }
        _startLoading2.Hook!.OriginalFunction(Self, IoBatch);
    }
    
    internal unsafe delegate void FAsyncPackage2_StartLoading3(Native.FAsyncPackage2_UE5_3* Self, nint ThreadState, nint IoBatch);
    private unsafe void FAsyncPackage2_StartLoading3Impl(Native.FAsyncPackage2_UE5_3* Self, nint ThreadState, nint IoBatch) 
    {
        var DiskName = Self->PackagePathToLoad;
        if (!DiskName.IsNone() && _config.FileAccessLog)
        {
            Log($"StartLoading: {DiskName}");    
        }
        _startLoading3.Hook!.OriginalFunction(Self, ThreadState, IoBatch);
    }
    
    internal unsafe delegate void FAsyncPackage2_StartLoading4(Native.FAsyncPackage2_UE5_4* Self, nint ThreadState, nint IoBatch);
    private unsafe void FAsyncPackage2_StartLoading4Impl(Native.FAsyncPackage2_UE5_4* Self, nint ThreadState, nint IoBatch) 
    {
        var DiskName = Self->PackagePathToLoad;
        if (!DiskName.IsNone() && _config.FileAccessLog)
        {
            Log($"StartLoading: {DiskName}");    
        }
        _startLoading4.Hook!.OriginalFunction(Self, ThreadState, IoBatch);
    }
}