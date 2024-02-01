using UnrealEssentials.Interfaces;
namespace UnrealEssentials;
public struct Signatures
{
    internal string GetPakSigningKeys { get; set; }
    internal string GetPakFolders { get; set; }
    internal string GMalloc { get; set; }
    internal string GetPakOrder { get; set; }
    internal string PakOpenRead { get; set; }
    internal string PakOpenAsyncRead { get; set; }
    internal string IsNonPakFilenameAllowed { get; set; }
    internal string FindFileInPakFiles { get; set; }
<<<<<<< HEAD
    internal string FileIoStoreOpenContainer { get; set; }
    internal string ReadBlocks { get; set; }
    internal TocType? TocVersion { get; set; }
    internal PakType PakVersion { get; set; }
=======
>>>>>>> master

    internal static Dictionary<string, Signatures> VersionSigs = new()
    {
        {
            "++UE4+Release-4.18", // 4.18
            new Signatures
            {
                PakOpenRead = "48 89 6C 24 ?? 48 89 74 24 ?? 57 48 83 EC 40 41 0F B6 E8 48 C7 44 24 ?? 00 00 00 00"
            }
        },
        {
            "++UE4+Release-4.19", // 4.19
            new Signatures
            {
                PakOpenRead = "48 89 6C 24 ?? 48 89 74 24 ?? 57 48 83 EC 40 41 0F B6 E8"
            }
        },
        {
            "++UE4+Release-4.20", // 4.20
            new Signatures
            {

            }
        },
        {
            "++UE4+Release-4.21", // 4.21
            new Signatures
            {
                PakOpenRead = "48 89 5C 24 ?? 55 56 57 41 54 41 55 41 56 41 57 48 8D 6C 24 ?? 48 81 EC B0 00 00 00 48 8B 05 ?? ?? ?? ?? 48 33 C4 48 89 45 ?? 66 0F 6F 05 ?? ?? ?? ?? 48 8D 59 ??"
            }
        },
        {
            "++UE4+Release-4.22", // 4.22
            new Signatures
            {

            }
        },
        {
            "++UE4+Release-4.23", // 4.23
            new Signatures
            {

            }
        },
        {
            "++UE4+Release-4.24", // 4.24
            new Signatures
            {

            }
        },
        {
            "++UE4+Release-4.25", // 4.25
            new Signatures
            {
                GetPakSigningKeys = "E8 ?? ?? ?? ?? 48 8B D8 39 78 ??",
                GetPakFolders = "48 89 5C 24 ?? 48 89 74 24 ?? 48 89 7C 24 ?? 4C 89 74 24 ?? 55 48 8B EC 48 83 EC 40 48 8D 4D ??",
                GMalloc = "48 89 05 ?? ?? ?? ?? E8 ?? ?? ?? ?? E8 ?? ?? ?? ?? 84 C0 74 ??",
                GetPakOrder = "48 89 5C 24 ?? 57 48 83 EC 40 48 8B D9 48 8D 4C 24 ?? E8 ?? ?? ?? ?? 83 78 08 00",
                PakOpenRead = "48 89 5C 24 ?? 55 56 57 41 54 41 55 41 56 41 57 48 8D 6C 24 ?? 48 81 EC D0 00 00 00 48 8B 05 ?? ?? ?? ?? 48 33 C4 48 89 45 ?? 66 0F 6F 05 ?? ?? ?? ??",
                PakOpenAsyncRead = "40 55 57 41 56 41 57 48 81 EC 98 00 00 00",
                IsNonPakFilenameAllowed = "48 8B C4 55 41 55 48 8D 68 ?? 48 81 EC 98 00 00 00",
                FindFileInPakFiles = "48 8B C4 4C 89 48 ?? 4C 89 40 ?? 55 53 48 8B EC",
<<<<<<< HEAD
                TocVersion = TocType.Initial,
                PakVersion = PakType.FrozenIndex
=======
>>>>>>> master
            }
        },
        {
            "++UE4+Release-4.26", // 4.26
            new Signatures
            {
                GetPakSigningKeys = "E8 ?? ?? ?? ?? 48 8B D8 39 78 ?? 0F 84 ?? ?? ?? ??",
                GetPakFolders = "48 89 5C 24 ?? 48 89 74 24 ?? 48 89 7C 24 ?? 4C 89 74 24 ?? 55 48 8B EC 48 83 EC 40 48 8D 4D ??",
                GMalloc = "48 89 05 ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 8B 0D ?? ?? ?? ?? 48 8B 01 FF 90 ?? ?? ?? ?? 84 C0",
                GetPakOrder = "48 89 5C 24 ?? 57 48 83 EC 40 48 8B D9 48 8D 4C 24 ?? E8 ?? ?? ?? ?? 83 78 ?? 00",
                PakOpenRead = "4C 8B DC 55 41 55 49 8D 6B ?? 48 81 EC B8 00 00 00 48 8B 05 ?? ?? ?? ?? 48 33 C4 48 89 45 ?? 66 0F 6F 05 ?? ?? ?? ??",
                PakOpenAsyncRead = "48 89 6C 24 ?? 56 57 41 56 48 81 EC 90 00 00 00 48 8B 05 ?? ?? ?? ?? 48 33 C4 48 89 84 24 ?? ?? ?? ?? 48 8B EA",
                IsNonPakFilenameAllowed = "48 89 5C 24 ?? 48 89 6C 24 ?? 56 57 41 56 48 83 EC 30 48 8B F1 45 33 C0",
                FindFileInPakFiles = "48 89 5C 24 ?? 48 89 6C 24 ?? 48 89 74 24 ?? 4C 89 44 24 ?? 57 41 54 41 55 41 56 41 57 48 83 EC 30 33 ED",
<<<<<<< HEAD
                FileIoStoreOpenContainer = "48 89 5C 24 ?? 48 89 6C 24 ?? 48 89 74 24 ?? 48 89 7C 24 ?? 41 56 48 83 EC 20 49 8B F1 4D 8B F0",
                TocVersion = TocType.DirectoryIndex,
                PakVersion = PakType.Fn64BugFix
=======
>>>>>>> master
            }
        },
        {
            "++UE4+Release-4.27", // 4.27
            new Signatures
            {
                GetPakSigningKeys = "E8 ?? ?? ?? ?? 48 8B F8 39 70 ??",
                GetPakFolders = "48 89 5C 24 ?? 48 89 74 24 ?? 48 89 7C 24 ?? 4C 89 74 24 ?? 55 48 8B EC 48 83 EC 40 48 8D 4D ??",
                GMalloc = "48 89 35 ?? ?? ?? ?? EB ?? 48 8B 3D ?? ?? ?? ??",
                GetPakOrder = "48 89 5C 24 ?? 57 48 83 EC 40 48 8B D9 48 8D 4C 24 ?? E8 ?? ?? ?? ?? 83 78 ?? 00",
                PakOpenRead = "4C 8B DC 55 53 57 41 54 49 8D 6B ?? 48 81 EC B8 00 00 00 48 8B 05 ?? ?? ?? ?? 48 33 C4 48 89 45 ?? 66 0F 6F 05 ?? ?? ?? ??",
                PakOpenAsyncRead = "48 89 5C 24 ?? 55 56 41 54 41 56 41 57 48 8D 6C 24 ?? 48 81 EC 90 00 00 00 48 8B 05 ?? ?? ?? ??",
                IsNonPakFilenameAllowed = "48 89 5C 24 ?? 48 89 6C 24 ?? 56 57 41 56 48 83 EC 30 48 8B F1 45 33 C0",
                FindFileInPakFiles = "48 8B C4 53 48 83 EC 50 48 89 68 ?? 48 89 70 ??",
<<<<<<< HEAD
                FileIoStoreOpenContainer = "48 89 5C 24 ?? 48 89 6C 24 ?? 48 89 74 24 ?? 48 89 7C 24 ?? 41 56 48 83 EC 20 49 8B F1 4D 8B F0",
                ReadBlocks = "4C 8B DC 49 89 4B ?? 53 57 41 54",
                TocVersion = TocType.PartitionSize,
                PakVersion = PakType.Fn64BugFix
=======
>>>>>>> master
            }
        },
        {
            "ScarletNexus-Win64-Shipping.exe", // Scarlet Nexus (Modified 4.25+)
            new Signatures
            {
                GetPakSigningKeys = "E8 ?? ?? ?? ?? 48 8B D8 39 78 ??",
                GetPakFolders = "48 89 5C 24 ?? 48 89 74 24 ?? 48 89 7C 24 ?? 4C 89 74 24 ?? 55 48 8B EC 48 83 EC 40 48 8D 4D ??",
                GMalloc = "48 89 05 ?? ?? ?? ?? E8 ?? ?? ?? ?? E8 ?? ?? ?? ?? 84 C0 74 ??",
                GetPakOrder = "48 89 5C 24 ?? 57 48 83 EC 40 48 8B D9 48 8D 4C 24 ?? E8 ?? ?? ?? ?? 83 78 08 00",
                PakOpenRead = "48 89 5C 24 ?? 55 56 57 41 54 41 55 41 56 41 57 48 8D 6C 24 ?? 48 81 EC B0 00 00 00 48 8B 05 ?? ?? ?? ?? 48 33 C4 48 89 45 ?? 66 0F 6F 05 ?? ?? ?? ??",
                PakOpenAsyncRead = "40 55 53 56 57 41 54 41 55 48 8D 6C 24 ?? 48 81 EC A8 00 00 00",
                IsNonPakFilenameAllowed = "48 89 5C 24 ?? 48 89 6C 24 ?? 56 57 41 56 48 83 EC 30 48 8B F1 45 33 C0",
                FindFileInPakFiles = "48 8B C4 4C 89 48 ?? 4C 89 40 ?? 55 53 48 8B EC",
<<<<<<< HEAD
                TocVersion = TocType.DirectoryIndex,
                PakVersion = PakType.FrozenIndex
=======
>>>>>>> master
            }
        },
        {
            "Hi-Fi-RUSH.exe", // Hi-Fi Rush (Modified 4.27)
            new Signatures
            {
                GetPakSigningKeys = "E8 ?? ?? ?? ?? 48 8B F0 44 39 78 ??",
                GetPakFolders = "48 89 5C 24 ?? 48 89 74 24 ?? 57 48 83 EC 40 48 8D 4C 24 ??",
                GMalloc = "48 8B 0D ?? ?? ?? ?? 48 85 C9 75 ?? E8 ?? ?? ?? ?? 48 8B 0D ?? ?? ?? ?? 48 8B 01 48 8B D3 FF 50 ?? 48 83 C4 20",
                GetPakOrder = "48 89 5C 24 ?? 48 89 6C 24 ?? 48 89 74 24 ?? 57 48 83 EC 40 48 89 CF 48 8D 4C 24 ??",
                PakOpenRead = "4C 8B DC 55 53 57 41 54 49 8D 6B ?? 48 81 EC B8 00 00 00 48 8B 05 ?? ?? ?? ?? 48 33 C4 48 89 45 ?? 66 0F 6F 05 ?? ?? ?? ??",
                PakOpenAsyncRead = "48 89 5C 24 ?? 55 56 41 54 41 56 41 57 48 8D 6C 24 ?? 48 81 EC 90 00 00 00 48 8B 05 ?? ?? ?? ??",
                IsNonPakFilenameAllowed = "48 89 5C 24 ?? 48 89 6C 24 ?? 56 57 41 56 48 83 EC 30 48 8B F1 45 33 C0",
                FindFileInPakFiles = "48 8B C4 53 48 83 EC 50 48 89 68 ?? 48 89 70 ??",
<<<<<<< HEAD
                ReadBlocks = "4C 8B DC 49 89 53 ?? 49 89 4B ?? 53 55",
                TocVersion = TocType.PartitionSize,
                PakVersion = PakType.Fn64BugFix
=======
>>>>>>> master
            }
        },
        {
            "Sackboy-Win64-Shipping.exe", // Sackboy: A Big Adventure (Modified 4.25)
            new Signatures
            {
                GetPakSigningKeys = "E8 ?? ?? ?? ?? 48 8B D8 39 78 ??",
                GetPakFolders = "48 89 5C 24 ?? 48 89 74 24 ?? 48 89 7C 24 ?? 4C 89 74 24 ?? 55 48 8B EC 48 83 EC 40 48 8D 4D ??",
                GMalloc = "48 89 05 ?? ?? ?? ?? E8 ?? ?? ?? ?? E8 ?? ?? ?? ?? 84 C0 74 ??",
                GetPakOrder = "48 89 5C 24 ?? 57 48 83 EC 40 48 8B D9 48 8D 4C 24 ?? E8 ?? ?? ?? ?? 83 78 08 00",
<<<<<<< HEAD
                PakOpenRead ="48 89 5C 24 ?? 55 56 57 41 54 41 55 41 56 41 57 48 8D 6C 24 ?? 48 81 EC B0 00 00 00 48 8B 05 ?? ?? ?? ?? 48 33 C4 48 89 45 ?? 66 0F 6F 05 ?? ?? ?? ?? 48 8D 59 ??",
                PakOpenAsyncRead = "40 55 53 56 57 41 54 41 55 48 8D 6C 24 ?? 48 81 EC A8 00 00 00",
                IsNonPakFilenameAllowed = "48 89 5C 24 ?? 48 89 6C 24 ?? 56 57 41 56 48 83 EC 30 48 8B F1 45 33 C0",
                FindFileInPakFiles = "48 8B C4 4C 89 48 ?? 4C 89 40 ?? 55 53 48 8B EC",
                TocVersion = TocType.Initial,
                PakVersion = PakType.FrozenIndex
=======
                PakOpenRead = "48 89 5C 24 ?? 55 56 57 41 54 41 55 41 56 41 57 48 8D 6C 24 ?? 48 81 EC B0 00 00 00 48 8B 05 ?? ?? ?? ?? 48 33 C4 48 89 45 ?? 66 0F 6F 05 ?? ?? ?? ?? 48 8D 59 ??",
                PakOpenAsyncRead = "40 55 53 56 57 41 54 41 55 48 8D 6C 24 ?? 48 81 EC A8 00 00 00",
                IsNonPakFilenameAllowed = "48 89 5C 24 ?? 48 89 6C 24 ?? 56 57 41 56 48 83 EC 30 48 8B F1 45 33 C0",
                FindFileInPakFiles = "48 8B C4 4C 89 48 ?? 4C 89 40 ?? 55 53 48 8B EC",
>>>>>>> master
            }
        },
    };
}
