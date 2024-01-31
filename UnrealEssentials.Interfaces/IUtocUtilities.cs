namespace UnrealEssentials.Interfaces
{
    public interface IUtocUtilities
    {
        public TocType? GetTocVersion();
        public string GetUnrealEssentialsPath();
        public string GetTargetTocDirectory();
        // The UCAS will always report having a file size of 0, but has a valid handle. I don't know why this happens, it just does
        public string GetFileIoStoreHookSig();
        public void RemovePakFolder(string modsPath);
        public void AddPakFolder(string modsPath);
        public PakType GetPakVersion();
    }
}
