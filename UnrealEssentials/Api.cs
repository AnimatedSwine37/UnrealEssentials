using System.Diagnostics;
using UnrealEssentials.Interfaces;

namespace UnrealEssentials
{
    public class Api : IUtocUtilities
    {
        private Signatures sigs;
        private string essentialsDir;
        public TocType? GetTocVersion() => sigs.TocVersion;
        public Api(Signatures sigs, string essentialsDir) 
        { 
            this.sigs = sigs;
            this.essentialsDir = essentialsDir;
        }
        public string GetUnrealEssentialsPath() => essentialsDir;
        public string GetTargetTocDirectory() => Path.Combine(essentialsDir, "UTOC");
        public string GetFileIoStoreHookSig() => sigs.FileIoStoreOpenContainer;
    }
}
