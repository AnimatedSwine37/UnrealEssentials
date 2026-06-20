using FileEmulationFramework.Lib.Utilities;

namespace UTOC.Stream.Emulator;

public static class Extensions
{
    public static void LogDebug(this Logger self, string Text)
        => self.Debug("[UtocEmulator] " + Text);
    
    public static void LogInfo(this Logger self, string Text)
        => self.Info("[UtocEmulator] " + Text);
    
    public static void LogWarning(this Logger self, string Text)
        => self.Warning("[UtocEmulator] " + Text);
    
    public static void LogError(this Logger self, string Text)
        => self.Error("[UtocEmulator] " + Text);
}