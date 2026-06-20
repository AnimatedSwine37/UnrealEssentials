using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using UTOC.Stream.Emulator.Interfaces;

namespace UTOC.Stream.Emulator
{
    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct Array<T>
        where T : unmanaged
    {
        public T* Entries;
        public nint Len;
    }

    public static unsafe class RustApi
    {
        const string __DllName = "utoc_emulator";
        
        [DllImport(__DllName, EntryPoint = "add_from_folders", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
        public static extern void add_from_folders(nint mod_path, EngineVersion version);

        [DllImport(__DllName, EntryPoint = "add_from_folders_with_mount", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
        public static extern void add_from_folders_with_mount(nint mod_path, nint virtual_path, EngineVersion version);

        [DllImport(__DllName, EntryPoint = "build_toc", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
        public static extern bool build_toc(EngineVersion version, Array<byte>* toc, Array<PartitionBlock>* blocks, Array<byte>* header);
        
        [DllImport(__DllName, EntryPoint = "set_reloaded_logger", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
        internal static extern void set_reloaded_logger(delegate* unmanaged[Stdcall]<nint, nint, int, void> offset);
        
        [DllImport(__DllName, EntryPoint = "set_free_csharp_string", CallingConvention = CallingConvention.StdCall, ExactSpelling = true)]
        private static extern nuint set_free_csharp_string(delegate* unmanaged[Stdcall]<nint, void> offset);
        
        [UnmanagedCallersOnly(CallConvs = [ typeof(CallConvStdcall) ])]
        public static void FreeCSharpString(nint p) => Marshal.FreeHGlobal(p);

        public static void SetCallbacks()
        {
            set_free_csharp_string(&FreeCSharpString);
        }
    }
}
