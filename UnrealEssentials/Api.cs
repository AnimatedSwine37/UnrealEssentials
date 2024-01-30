using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using UnrealEssentials.Interfaces;

namespace UnrealEssentials
{
    public class Api : IUtocUtilities
    {
        private Signatures sigs;
        public TocType? GetTocVersion() => sigs.TocVersion;
        public Api(Signatures sigs) { this.sigs = sigs; }
    }
}
