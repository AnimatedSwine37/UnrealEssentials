using UTOC.Stream.Emulator.Interfaces;

namespace UnrealEssentials.Components;

public static class TypeExtensions
{
    public static string ToBranchVersion(this EngineVersion self)
    {
        var parts = self.ToString().Split("_");
        return $"++UE{parts[1]}+Release-{parts[1]}.{parts[2]}";
    }
}