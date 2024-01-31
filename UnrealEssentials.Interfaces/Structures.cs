namespace UnrealEssentials.Interfaces
{
    public enum TocType
    {
        Initial = 1, // 4.25
        DirectoryIndex = 2, // 4.25+, 4.26
        PartitionSize = 3, // 4.27
        PerfectHash = 4, // 5.0+
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
}
