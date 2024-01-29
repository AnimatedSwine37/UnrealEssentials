using System.ComponentModel;
using UnrealEssentials.Template.Configuration;

namespace UnrealEssentials.Configuration;
public class Config : Configurable<Config>
{
    [DisplayName("Log File Access")]
    [Description("Logs to the console whenever the game opens a file (currently only ones in PAKs)")]
    [DefaultValue(false)]
    public bool FileAccessLog { get; set; } = false;


    [DisplayName("Debug Mode")]
    [Description("Logs additional information to the console that is useful for debugging.")]
    [DefaultValue(false)]
    public bool DebugEnabled { get; set; } = false;
}

/// <summary>
/// Allows you to override certain aspects of the configuration creation process (e.g. create multiple configurations).
/// Override elements in <see cref="ConfiguratorMixinBase"/> for finer control.
/// </summary>
public class ConfiguratorMixin : ConfiguratorMixinBase
{
    // 
}