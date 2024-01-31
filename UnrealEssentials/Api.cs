using System.Diagnostics;
using UnrealEssentials.Interfaces;

namespace UnrealEssentials
{
    public class Api : IUtocUtilities
    {
        private Signatures sigs;
        private string essentialsDir;
        private Action<string> RemoveFolderCb;
        public TocType? GetTocVersion() => sigs.TocVersion;
        public Api(Signatures sigs, string essentialsDir, Action<string> removeFolderCb) 
        { 
            this.sigs = sigs;
            this.essentialsDir = essentialsDir;
            RemoveFolderCb = removeFolderCb;
        }
        public string GetUnrealEssentialsPath() => essentialsDir;
        public string GetTargetTocDirectory() => Path.Combine(essentialsDir, "UTOC");
        public string GetFileIoStoreHookSig() => sigs.FileIoStoreOpenContainer;
        public void RemoveFolderOnFailure(string modsPath) => RemoveFolderCb(modsPath);
        public PakType GetPakVersion() => sigs.PakVersion;
    }
}
