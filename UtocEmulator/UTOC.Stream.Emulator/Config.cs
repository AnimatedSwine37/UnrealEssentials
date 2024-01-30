using FileEmulationFramework.Lib.Utilities;
using Reloaded.Mod.Interfaces.Structs;
using System.ComponentModel;
using UTOC.Stream.Emulator.Template.Configuration;

namespace UTOC.Stream.Emulator.Configuration
{
    public class Config : Configurable<Config>
    {
        [DisplayName("Log Level")]
        [Description("Declares which elements should be logged to the console.\nMessages less important than this level will not be logged.")]
        [DefaultValue(LogSeverity.Warning)]
        public LogSeverity LogLevel { get; set; } = LogSeverity.Information;

        [DisplayName("Dump Emulated IO Store Files")]
        [Description("Creates a dump of emulated IO Store files (.utoc + .ucas) as they are written.")]
        [DefaultValue(LogSeverity.Information)]
        public bool DumpFiles { get; set; } = false;
    }

    /// <summary>
    /// Allows you to override certain aspects of the configuration creation process (e.g. create multiple configurations).
    /// Override elements in <see cref="ConfiguratorMixinBase"/> for finer control.
    /// </summary>
    public class ConfiguratorMixin : ConfiguratorMixinBase
    {
        // 
    }
}
