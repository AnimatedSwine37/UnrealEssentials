using System.Diagnostics;
using UnrealEssentials.Interfaces;

namespace UnrealEssentials
{
    public class Api : IUtocUtilities
    {
        private Signatures sigs;
        private string essentialsDir;
        private Action<string> RemoveFolderCb;
        private Action<string> AddFolderCb;
        public TocType? GetTocVersion() => sigs.TocVersion;
        public Api(
            Signatures sigs, string essentialsDir, Action<string> addFolderCb, Action<string> removeFolderCb
        ) 
        { 
            this.sigs = sigs;
            this.essentialsDir = essentialsDir;
            RemoveFolderCb = removeFolderCb;
            AddFolderCb = addFolderCb;
        }
        public string GetUnrealEssentialsPath() => essentialsDir;
        public string GetTargetTocDirectory() => Path.Combine(essentialsDir, "UTOC");
        public string GetFileIoStoreHookSig() => sigs.FileIoStoreOpenContainer;
        public string GetReadBlockSig() => sigs.ReadBlocks;
        public void RemovePakFolder(string modsPath) => RemoveFolderCb(modsPath);
        public void AddPakFolder(string modsPath) => AddFolderCb(modsPath);
        public PakType GetPakVersion() => sigs.PakVersion;
    }
}
