namespace UTOC.Stream.Emulator.Interfaces;

public enum TocType
{
    Initial = 1, // 4.25
    DirectoryIndex = 2, // 4.25+, 4.26
    PartitionSize = 3, // 4.27
    PerfectHash = 4, // 5.0 - 5.3
    OnDemandMetaData = 6, // 5.4
    ReplaceIoChunkHashWithIoHash = 8, // 5.5
}

// UE4 pak type
// See https://github.com/trumank/repak?tab=readme-ov-file#compatibility for more details
public enum PakType
{
    NoTimestamps = 1,
    CompressionEncryption = 2,
    IndexEncryption = 3,
    RelativeChunkOffsets = 4,
    EncryptionKeyGuid = 5,
    FNameBasedCompressionA = 6,
    FNameBasedCompressionB = 7,
    FrozenIndex = 8,
    Fn64BugFix = 9
}

/// This must stay in sync with EngineVersion in utoc-emulator over in Rust land!
public enum EngineVersion : uint
{
    UE_4_25 = (4 << 0x8) + 25,
    UE_4_26 = (4 << 0x8) + 26, // 4.25+ (e.g Scarlet Nexus) is treated as 4.26
    UE_4_27 = (4 << 0x8) + 27,
    UE_5_0 = (5 << 0x8) + 0,
    UE_5_1 = (5 << 0x8) + 1,
    UE_5_2 = (5 << 0x8) + 2,
    UE_5_3 = (5 << 0x8) + 3,
    UE_5_4 = (5 << 0x8) + 4,
    UE_5_5 = (5 << 0x8) + 5,
    UE_5_6 = (5 << 0x8) + 6,
    UE_5_7 = (5 << 0x8) + 7,
    // if there are any games that require a special ID, then define them with [value] << 0x10
}