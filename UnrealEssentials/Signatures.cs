namespace UnrealEssentials;
internal struct Signatures
{
    internal string GetPakSigningKeys { get; set; }
    internal string GetPakFolders { get; set; }
    internal string GMalloc { get; set; }

    internal static Dictionary<string, Signatures> VersionSigs = new()
    {
        {
            "++UE4+Release-4.25", // 4.25
            new Signatures
            {
                GetPakSigningKeys = "E8 ?? ?? ?? ?? 48 8B D8 39 78 ??",
                GetPakFolders = "48 89 5C 24 ?? 48 89 74 24 ?? 48 89 7C 24 ?? 4C 89 74 24 ?? 55 48 8B EC 48 83 EC 40 48 8D 4D ??",
                GMalloc = "48 89 05 ?? ?? ?? ?? E8 ?? ?? ?? ?? E8 ?? ?? ?? ?? 84 C0 74 ??",
            }
        },
        {
            "++UE4+Release-4.25Plus M3", // Scarlet Nexus
            new Signatures
            {
                GetPakSigningKeys = "E8 ?? ?? ?? ?? 48 8B D8 39 78 ??",
                GetPakFolders = "48 89 5C 24 ?? 48 89 74 24 ?? 48 89 7C 24 ?? 4C 89 74 24 ?? 55 48 8B EC 48 83 EC 40 48 8D 4D ??",
                GMalloc = "48 89 05 ?? ?? ?? ?? E8 ?? ?? ?? ?? E8 ?? ?? ?? ?? 84 C0 74 ??",
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
                GetPakSigningKeys = "E8 ?? ?? ?? ?? 48 8B F8 39 70 ??",
                GetPakFolders = "48 89 5C 24 ?? 48 89 74 24 ?? 48 89 7C 24 ?? 4C 89 74 24 ?? 55 48 8B EC 48 83 EC 40 48 8D 4D ??",
                GMalloc = "48 89 35 ?? ?? ?? ?? EB ?? 48 8B 3D ?? ?? ?? ??"
            }
        },
        {
            "++ue4+hibiki_patch+4.27hbk", // Hi-Fi Rush
            new Signatures
            {
                GetPakSigningKeys = "E8 ?? ?? ?? ?? 48 8B F0 44 39 78 ??",
                GetPakFolders = "48 89 5C 24 ?? 48 89 74 24 ?? 57 48 83 EC 40 48 8D 4C 24 ??",
                GMalloc = "48 8B 0D ?? ?? ?? ?? 48 85 C9 75 ?? E8 ?? ?? ?? ?? 48 8B 0D ?? ?? ?? ?? 48 8B 01 48 8B D3 FF 50 ?? 48 83 C4 20"
            }
        }
    };
}
