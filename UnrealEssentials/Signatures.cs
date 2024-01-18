namespace UnrealEssentials;
internal struct Signatures
{
    internal string GetPakSigningKeys { get; set; }

    internal static Dictionary<string, Signatures> VersionSigs = new()
    {
        {
            "4.25",
            new Signatures
            {
                GetPakSigningKeys = "E8 ?? ?? ?? ?? 48 8B D8 39 78 ??"
            }
        },
        {
            "4.27",
            new Signatures
            {
                GetPakSigningKeys = "E8 ?? ?? ?? ?? 48 8B F8 39 70 ??"
            }
        },
        {
            "Hi-Fi-RUSH.exe",
            new Signatures
            {
                GetPakSigningKeys = "E8 ?? ?? ?? ?? 48 8B F0 44 39 78 ??"
            }
        }
    };
}
