using FileEmulationFramework.Interfaces;
using FileEmulationFramework.Lib.Utilities;
using Reloaded.Hooks.Definitions;
using Reloaded.Hooks.ReloadedII.Interfaces;
using Reloaded.Memory.SigScan.ReloadedII.Interfaces;
using Reloaded.Mod.Interfaces;
using Reloaded.Mod.Interfaces.Internal;
using IReloadedHooks = Reloaded.Hooks.ReloadedII.Interfaces.IReloadedHooks;
using System.Diagnostics;
using System.Xml.Linq;
using UnrealEssentials.Interfaces;
using UTOC.Stream.Emulator.Configuration;
using UTOC.Stream.Emulator.Template;
using System.Runtime.InteropServices;
using Reloaded.Memory.Sigscan.Definitions.Structs;

namespace UTOC.Stream.Emulator
{
    /// <summary>
    /// Your mod logic goes here.
    /// </summary>
    public class Mod : ModBase // <= Do not Remove.
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

        // File Emulation Framework Globals
        private Logger _log;
        private UtocEmulator _emu;

        private long BaseAddress;
        private IHook<OpenContainerDelegate> _openContainerHook;
        private IHook<FFileIoStore_ReadBlocks> _readBlocksHook;

        public Mod(ModContext context)
        {
            _modLoader = context.ModLoader;
            _hooks = context.Hooks;
            _logger = context.Logger;
            _owner = context.Owner;
            _configuration = context.Configuration;
            _modConfig = context.ModConfig;

            _modLoader.GetController<IUtocUtilities>().TryGetTarget(out var tocUtils); // For communication with Unreal Essentials

            _log = new Logger(_logger, _configuration.LogLevel);
            _log.Info("Starting UTOC.Stream.Emulator");
            _emu = new UtocEmulator(
                _log, _configuration.DumpFiles, _modLoader.GetDirectoryForModId(_modConfig.ModId),
                tocUtils.GetUnrealEssentialsPath(), tocUtils.AddPakFolder
            );

            _modLoader.ModLoading += OnModLoading;
            _modLoader.OnModLoaderInitialized += OnLoaderInit;

            var ctrl_weak = _modLoader.GetController<IEmulationFramework>().TryGetTarget(out var framework);
            _modLoader.GetController<IStartupScanner>().TryGetTarget(out var scanFactory);
            _emu.TocVersion = tocUtils.GetTocVersion(); // Set Toc Version
            _emu.PakVersion = tocUtils.GetPakVersion(); // Set Pak Version
            framework!.Register(_emu);
            BaseAddress = Process.GetCurrentProcess().MainModule.BaseAddress;
            ContainerFileSizeOverride(tocUtils, scanFactory);
        }

        private void ContainerFileSizeOverride(IUtocUtilities utils, IStartupScanner scanFactory)
        {
            if (utils.GetFileIoStoreHookSig() != null)
            {
                scanFactory.AddMainModuleScan(utils.GetFileIoStoreHookSig(), result =>
                {
                    if (!result.Found)
                    {
                        _log.Info($"[UtocEmulator] Unable to find Open Container, stuff won't work :(");
                        return;
                    }
                    var openContainerAddress = BaseAddress + result.Offset;
                    _log.Info($"[UtocEmulator] Found OpenContainer at 0x{openContainerAddress:X}");
                    _openContainerHook = _hooks.CreateHook<OpenContainerDelegate>(OpenContainer, openContainerAddress).Activate();
                });
            } else if (utils.GetReadBlockSig() != null)
            {
                scanFactory.AddMainModuleScan(utils.GetReadBlockSig(), result2 =>
                {
                    if (!result2.Found)
                    {
                        _log.Info($"[UtocEmulator] Unable to find Read Blocks, stuff won't work :(");
                        return;
                    }
                    var readBlocksAddress = BaseAddress + result2.Offset;
                    _log.Info($"[UtocEmulator] Found OpenContainer at 0x{readBlocksAddress:X}");
                    unsafe { _readBlocksHook = _hooks.CreateHook<FFileIoStore_ReadBlocks>(ReadBlocks, readBlocksAddress).Activate(); }
                });
            }
        }

        private void OnLoaderInit()
        {
            _modLoader.OnModLoaderInitialized -= OnLoaderInit;
            _modLoader.ModLoading -= OnModLoading;
            _emu.OnLoaderInit();
        }
        private void OnModLoading(IModV1 mod, IModConfigV1 conf) => _emu.OnModLoading(conf.ModId, _modLoader.GetDirectoryForModId(conf.ModId));

        public bool OpenContainer(nuint thisPtr, nuint containerFilePath, nuint containerFileHandle, nuint containerFileSize)
        { // This is a temporary measure due to a bug in FileEmulationFramework
            var returnValue = _openContainerHook.OriginalFunction(thisPtr, containerFilePath, containerFileHandle, containerFileSize);
            unsafe
            {
                if (Marshal.PtrToStringUni((nint)containerFilePath).Contains(Constants.UnrealEssentialsName))
                {
                    *(long*)containerFileSize = _emu.CasStream.Length;
                }
            }
            return returnValue;
        }

        public unsafe void ReadBlocks(nuint thisPtr, FFileIoStoreResolvedRequest* ResolvedRequest)
        { // This is a temporary measure due to a bug in FileEmulationFramework
            var currentContainer = ResolvedRequest->ContainerFile->Partitions;
            var name = Marshal.PtrToStringUni((nint)ResolvedRequest->ContainerFile->FilePath);
            if (name.Contains(Constants.UnrealEssentialsName))
            {
                currentContainer->FileSize = _emu.CasStream.Length;
            }
            _readBlocksHook.OriginalFunction(thisPtr, ResolvedRequest);
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

    public delegate bool OpenContainerDelegate(nuint thisPtr, nuint containerFilePath, nuint containerFileHandle, nuint containerFileSize);
    public unsafe delegate void FFileIoStore_ReadBlocks(nuint thisPtr, FFileIoStoreResolvedRequest* ResolvedRequest);

    [StructLayout(LayoutKind.Explicit, Size = 0x10)]
    public unsafe struct FFileIoStoreResolvedRequest
    {
        [FieldOffset(0x8)] public FFileIoStoreContainerFile* ContainerFile;
        [FieldOffset(0x10)] public FFileIoStoreReadRequestLink* ReadRequestsHead;
    }

    [StructLayout(LayoutKind.Explicit, Size = 0x88)]
    public unsafe struct FFileIoStoreContainerFile
    {
        [FieldOffset(0x30)] public nuint FilePath;
        [FieldOffset(0x88)] public FFileIoStoreContainerFilePartition* Partitions;
    }

    [StructLayout(LayoutKind.Explicit, Size = 0x10)]
    public unsafe struct FFileIoStoreContainerFilePartition
    {
        [FieldOffset(0x8)] public long FileSize;
    }

    [StructLayout(LayoutKind.Explicit, Size = 0x10)]
    public unsafe struct FFileIoStoreReadRequestLink
    {
        [FieldOffset(0x0)] public FFileIoStoreReadRequestLink* Next;
        [FieldOffset(0x8)] public FFileIoStoreReadRequest* ReadRequest;
    }

    [StructLayout(LayoutKind.Explicit, Size = 0x20)]
    public unsafe struct FFileIoStoreReadRequest 
    {
        [FieldOffset(0x8)] public long FileHandle;
        [FieldOffset(0x8)] public long FileOffset;
        [FieldOffset(0x18)] public long FileSize;
    }

}