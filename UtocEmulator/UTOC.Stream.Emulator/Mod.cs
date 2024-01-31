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

        private long OpenContainerAddress;
        private IHook<OpenContainerDelegate> _openContainerHook;

        public Mod(ModContext context)
        {
            _modLoader = context.ModLoader;
            _hooks = context.Hooks;
            _logger = context.Logger;
            _owner = context.Owner;
            _configuration = context.Configuration;
            _modConfig = context.ModConfig;

            _modLoader.GetController<IUtocUtilities>().TryGetTarget(out var tocUtils);

            _log = new Logger(_logger, _configuration.LogLevel);
            _log.Info("Starting UTOC.Stream.Emulator");
            _emu = new UtocEmulator(
                _log, _configuration.DumpFiles, tocUtils.GetUnrealEssentialsPath(), 
                tocUtils.GetTargetTocDirectory(), tocUtils.RemoveFolderOnFailure
            );

            _modLoader.ModLoading += OnModLoading;
            _modLoader.OnModLoaderInitialized += OnLoaderInit;

            var ctrl_weak = _modLoader.GetController<IEmulationFramework>().TryGetTarget(out var framework);
            _modLoader.GetController<IStartupScanner>().TryGetTarget(out var scanFactory);
            _emu.TocVersion = tocUtils.GetTocVersion();
            framework!.Register(_emu);
            OpenContainerAddress = Process.GetCurrentProcess().MainModule.BaseAddress;
            if (tocUtils.GetFileIoStoreHookSig() != null)
            {
                scanFactory.AddMainModuleScan(tocUtils.GetFileIoStoreHookSig(), result =>
                {
                    if (!result.Found)
                    {
                        _log.Info($"[UtocEmulator] Unable to find OpenContainer, stuff won't work :(");
                        return;
                    }
                    OpenContainerAddress += result.Offset;
                    _log.Info($"[UtocEmulator] Found OpenContainer at 0x{OpenContainerAddress:X}");
                    _openContainerHook = _hooks.CreateHook<OpenContainerDelegate>(OpenContainer, OpenContainerAddress).Activate();
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
        {
            var returnValue = _openContainerHook.OriginalFunction(thisPtr, containerFilePath, containerFileHandle, containerFileSize);
            unsafe
            {
                if (Marshal.PtrToStringUni((nint)containerFilePath).Contains("UnrealEssentials"))
                {
                    *(long*)containerFileSize = _emu.CasStream.Length;
                }
            }
            return returnValue;
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
}