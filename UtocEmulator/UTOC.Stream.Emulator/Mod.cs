using FileEmulationFramework.Interfaces;
using FileEmulationFramework.Lib.Utilities;
using Reloaded.Mod.Interfaces;
using Reloaded.Mod.Interfaces.Internal;
using IReloadedHooks = Reloaded.Hooks.ReloadedII.Interfaces.IReloadedHooks;
using System.Diagnostics;
using UTOC.Stream.Emulator.Configuration;
using UTOC.Stream.Emulator.Template;
using UTOC.Stream.Emulator.Interfaces;

namespace UTOC.Stream.Emulator
{
    /// <summary>
    /// Your mod logic goes here.
    /// </summary>
    public class Mod : ModBase, IExports // <= Do not Remove.
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
        
        private IUtocEmulator _api;

        public Mod(ModContext context) 
        {
            //Debugger.Launch();
            _modLoader = context.ModLoader;
            _hooks = context.Hooks;
            _logger = context.Logger;
            _owner = context.Owner;
            _configuration = context.Configuration;
            _modConfig = context.ModConfig;

            _log = new Logger(_logger, _configuration.LogLevel);

            // Expose API
            _api = new Api(Initialise, (folder) => _emu.AddFromFolder(folder));
            _modLoader.AddOrReplaceController(context.Owner, _api);
        }

        public void Initialise(TocType? tocType, PakType pakType, string fileIoStoreSig, string readBlockSig, Action<string> addPakFolder, Action<string> removePakFolder)
        {
            _log.Info("Starting UTOC.Stream.Emulator");
            _emu = new UtocEmulator(
                _log, _configuration.DumpFiles, _modLoader.GetDirectoryForModId(_modConfig.ModId), addPakFolder);

            _modLoader.ModLoading += OnModLoading;
            _modLoader.OnModLoaderInitialized += OnLoaderInit;

            var ctrl_weak = _modLoader.GetController<IEmulationFramework>().TryGetTarget(out var framework);
            _emu.TocVersion = tocType; // Set Toc Version
            _emu.PakVersion = pakType; // Set Pak Version
            framework!.Register(_emu);
        }
        

        private void OnLoaderInit()
        {
            _modLoader.OnModLoaderInitialized -= OnLoaderInit;
            _modLoader.ModLoading -= OnModLoading;
            _emu.OnLoaderInit();
        }
        private void OnModLoading(IModV1 mod, IModConfigV1 conf) => _emu.OnModLoading(_modLoader.GetDirectoryForModId(conf.ModId));
        
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

        public Type[] GetTypes() => new[] { typeof(IUtocEmulator) };
    }
}