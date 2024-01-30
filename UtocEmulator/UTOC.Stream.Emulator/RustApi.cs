using System;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading.Tasks;

namespace UTOC.Stream.Emulator
{
    public static class RustApi
    {

        [DllImport("fileemu_utoc_stream_emulator")] // Collect assets
        public static extern void AddFromFolders(string mod_id, string mod_path);

        [DllImport("fileemu_utoc_stream_emulator")] // Build UTOC
        public static extern IntPtr BuildTableOfContents(string tocPath, IntPtr settings, uint settingsLength, ref long length);

        [DllImport("fileemu_utoc_stream_emulator")] // Build UCAS
        public static extern bool GetContainerBlocks(string casPath, ref nint blocks, ref nint blockCount, ref nint header, ref nint headerSize);

        [DllImport("fileemu_utoc_stream_emulator")]
        public static extern void SafeToDropContainerMetadata(); // Container entry data was copied over to managed C#, drop on Rust side

        [DllImport("fileemu_utoc_stream_emulator")]
        public static extern void PrintAssetCollectorResults();
    }
}
