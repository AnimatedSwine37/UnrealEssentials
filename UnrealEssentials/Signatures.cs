namespace UnrealEssentials;
internal struct Signatures
{
    internal string GetPakSigningKeys { get; set; }

    internal static Dictionary<string, Signatures> VersionSigs = new()
    {
        {
            "++UE4+Release-4.25", // 4.25
            new Signatures
            {
                GetPakSigningKeys = "E8 ?? ?? ?? ?? 48 8B D8 39 78 ??"
            }
        },
        {
            "++UE4+Release-4.25Plus M3", // Scarlet Nexus
            new Signatures
            {
                GetPakSigningKeys = "E8 ?? ?? ?? ?? 48 8B D8 39 78 ??"
            }
        },
        {
            "++UE4+Release-4.26", // 4.26
            new Signatures
            {
                GetPakSigningKeys = "E8 ?? ?? ?? ?? 48 8B D8 39 78 ?? 0F 84 ?? ?? ?? ??"
            }
        },
        {
            "++UE4+Release-4.27", // 4.27
            new Signatures
            {
                GetPakSigningKeys = "E8 ?? ?? ?? ?? 48 8B F8 39 70 ??"
            }
        },
        {
            "++ue4+hibiki_patch+4.27hbk", // Hi-Fi Rush
            new Signatures
            {
                GetPakSigningKeys = "E8 ?? ?? ?? ?? 48 8B F0 44 39 78 ??"
            }
        }
    };
}
