using Reloaded.Mod.Interfaces;
using Reloaded.Mod.Interfaces.Internal;
using UnrealEssentials.Components;
using UnrealEssentials.Configuration;
using UnrealEssentials.Interfaces;
using UnrealEssentials.Template;
using UnrealEssentials.Unreal;
using static UnrealEssentials.Utils;
using IReloadedHooks = Reloaded.Hooks.ReloadedII.Interfaces.IReloadedHooks;

namespace UnrealEssentials;

public class Mod : ModBase, IExports
{
    private readonly IModLoader _modLoader;
    private readonly IReloadedHooks? _hooks;
    private readonly ILogger _logger;
    private readonly IMod _owner;
    private Config _configuration;
    private readonly IModConfig _modConfig;
   
    private readonly Context? _context;
    private readonly PakMethods _pakMethods;
    private readonly UtocMethods? _utocMethods;
    
    private IUnrealEssentials _api;
    internal static IUnrealMemory Memory;

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

        Initialise(_logger, _configuration, _modLoader, _hooks);

        _context = ContextBuilder.CreateContext(_modLoader, _modConfig);
        if (_context == null) return;

        Memory = new UnrealMemory(_context!.Properties.Signatures, _hooks, 
            _context!.Properties.AllowExecuteCommands, _context!.Properties.CommandExecutorType);
        _pakMethods = new PakMethods(_hooks!, _configuration, _context!);
        if (_context!.HasUtocs)
        {
            _utocMethods = new UtocMethods(_hooks!, _configuration, _context!);
        }
        UnrealName.FNamePool.Initialize(_hooks!, _context!.Properties.Signatures);

        // Gather pak files from mods
        _modLoader.OnModLoaderInitialized += ModLoaderInit;
        _modLoader.ModLoading += ModLoading;

        // Expose API
        _api = new Api(_context!.AddFolder, _context!.AddFolderWithVirtualMount, _context!.AddFileWithVirtualMount);
        _modLoader.AddOrReplaceController(context.Owner, _api);
    }

    #region Mod Loader Events
    private void ModLoading(IModV1 mod, IModConfigV1 modConfig)
    {
        var modRootPath = _modLoader.GetDirectoryForModId(modConfig.ModId);
        _context!.LoadUEMounts(modRootPath, Path.Combine(modRootPath, "UEMounts.yaml"));
        var modsPath = Path.Combine(modRootPath, "UnrealEssentials");
        if (!Directory.Exists(modsPath))
            return;

        _context!.AddFolder(modsPath);
    }

    private void ModLoaderInit()
    {
        _modLoader.OnModLoaderInitialized -= ModLoaderInit;
        _modLoader.ModLoading -= ModLoading;
    }
    #endregion

    #region Standard Overrides
    public override void ConfigurationUpdated(Config configuration)
    {
        _configuration = configuration;
        _logger.WriteLine($"[{_modConfig.ModId}] Config Updated: Applying");
    }
    #endregion

    public Type[] GetTypes() => [typeof(IUnrealEssentials)];

    #region For Exports, Serialization etc.
#pragma warning disable CS8618 // Non-nullable field must contain a non-null value when exiting constructor. Consider declaring as nullable.
    public Mod() { }
#pragma warning restore CS8618
    #endregion
}