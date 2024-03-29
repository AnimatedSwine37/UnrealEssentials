﻿using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace UTOC.Stream.Emulator
{
    internal static class Constants
    {
        public static readonly string UtocExtension = ".utoc";
        public static readonly string UcasExtension = ".ucas";
        public static readonly string PakExtension = ".pak";
        public static readonly string DumpFolderParent = "FEmulator-Dumps";
        public static readonly string DumpFolderToc = "UTOCEmulator";
        public static readonly int DefaultCompressionBlockAlignment = 0x800;
        public static readonly string UnrealEssentialsName = "UnrealEssentials";
        public static readonly string TargetDir = "TargetFiles";
        public static readonly string DummyPakDir = "DummyPaks";
    }
}
