using FileEmulationFramework.Interfaces;
using FileEmulationFramework.Lib.Utilities;
using Reloaded.Mod.Interfaces;
using Reloaded.Mod.Interfaces.Internal;
using IReloadedHooks = Reloaded.Hooks.ReloadedII.Interfaces.IReloadedHooks;
using UTOC.Stream.Emulator.Configuration;
using UTOC.Stream.Emulator.Template;
using UTOC.Stream.Emulator.Interfaces;

namespace UTOC.Stream.Emulator
{
    public class Mod : ModBase, IExports
    {
        private readonly IModLoader _modLoader;
        private readonly IReloadedHooks? _hooks;
        private readonly ILogger _logger;
        private readonly IMod _owner;
        private Config _configuration;
        private readonly IModConfig _modConfig;

        // File Emulation Framework Globals
        private Logger _log;
        private UtocEmulator _emu;
        
        private IUtocEmulator _api;

        public Mod(ModContext context) 
        {
#if DEBUG
            //Debugger.Launch();
#endif
            _modLoader = context.ModLoader;
            _hooks = context.Hooks;
            _logger = context.Logger;
            _owner = context.Owner;
            _configuration = context.Configuration;
            _modConfig = context.ModConfig;

            _log = new Logger(_logger, _configuration.LogLevel);
            LogAdapter.RegisterLogger(_log);
            RustApi.SetCallbacks();

            // Expose API
            _api = new Api(Initialise, (folder) => _emu.AddFromFolder(folder), (folder, mount) => _emu.AddFromFolderWithMount(folder, mount));
            _modLoader.AddOrReplaceController(context.Owner, _api);
        }

        public void Initialise(EngineVersion engineVersion, bool hasUtocs, Action<string> addPakFolder, Action<string> removePakFolder)
        {
            _log.Info("Starting UTOC.Stream.Emulator");
            _emu = new UtocEmulator(
                _log, _configuration, _modLoader.GetDirectoryForModId(_modConfig.ModId), addPakFolder);

            _modLoader.ModLoading += OnModLoading;
            _modLoader.OnModLoaderInitialized += OnLoaderInit;

            var ctrl_weak = _modLoader.GetController<IEmulationFramework>().TryGetTarget(out var framework);
            _emu.EngineVersion = engineVersion;
            _emu.HasUtocs = hasUtocs;
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

        public Type[] GetTypes() => [typeof(IUtocEmulator)];
    }
}