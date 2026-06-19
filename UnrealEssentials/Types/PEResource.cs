namespace UnrealEssentials.Types;

#pragma warning disable CS0649

internal struct LanguageCodePage
{
    internal short wLanguage;
    internal short wCodePage;
}

internal struct FixedFileInfo
{
    internal uint dwSignature;
    internal uint dwStrucVersion;
    internal uint dwFileVersionMS;
    internal uint dwFileVersionLS;
    internal uint dwProductVersionMS;
    internal uint dwProductVersionLS;
    internal uint dwFileFlagsMask;
    internal uint dwFileFlags;
    internal uint dwFileOS;
    internal uint dwFileType;
    internal uint dwFileSubtype;
    internal uint dwFileDateMS;
    internal uint dwFileDateLS;
}

#pragma warning restore CS0649