using FileEmulationFramework.Interfaces;
using FileEmulationFramework.Interfaces.Reference;
using FileEmulationFramework.Lib;
using FileEmulationFramework.Lib.IO;
using FileEmulationFramework.Lib.IO.Struct;
using FileEmulationFramework.Lib.Utilities;
using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading.Tasks;

using Strim = System.IO.Stream;

namespace UTOC.Stream.Emulator
{

    // Must be kept in sync with PartitionBlock in toc_factory.rs
    public struct PartitionBlock
    {
        public IntPtr osPath; // *const u8
        public long start; // u64
        public long length; // u64
    }
    public class UtocEmulator : IEmulator
    {
        public static readonly string UtocExtension = ".utoc";
        public static readonly string UcasExtension = ".ucas";
        public static readonly string DumpFolderParent = "FEmulator-Dumps";
        public static readonly string DumpFolderToc = "UTOCEmulator";
        public static readonly int DefaultCompressionBlockAlignment = 0x800;
        public static int TestInt = 0;
        public bool DumpFiles { get; set; }
        public Logger _logger { get; init; }

        private readonly ConcurrentDictionary<string, Strim?> _pathToStream = new(StringComparer.OrdinalIgnoreCase);

        public bool CanDump { get; init; }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvStdcall)} )]
        public static void Test()
        {
            TestInt += 1;
        }

        public UtocEmulator(Logger logger, bool canDump) { _logger = logger; CanDump = canDump; }

        public bool TryCreateFile(IntPtr handle, string filepath, string route, out IEmulatedFile emulated)
        {
            // Check if we've already made a custom UTOC
            emulated = null!;
            if (_pathToStream.TryGetValue(filepath, out var stream))
            {
                if (stream == null) return false; // Avoid recursion into the same file
                return false;
            }
            // Check extension
            if (!TryCreateEmulatedFile(handle, filepath, filepath, filepath, ref emulated!, out _)) return false;
            return true;
        }

        public bool TryCreateIoStoreTOC(string path, ref IEmulatedFile? emulated, out Strim? stream)
        {
            stream = null;
            long length = 0;
            _pathToStream[path] = null; // Avoid recursion into the same file
            var result = RustApi.BuildTableOfContents(path, IntPtr.Zero, 0, ref length);
            if (result == IntPtr.Zero) return false;
            unsafe { stream = new UnmanagedMemoryStream((byte*)result, length); }
            _pathToStream.TryAdd(path, stream);
            emulated = new EmulatedFile<Strim>(stream);
            _logger.Info($"[UtocEmulator] Created Emulated Table of Contents with Path {path}");
            IEmulatedFile? emulatedCas = null;
            TryCreateIoStoreContainer(Path.ChangeExtension(path, UcasExtension), ref emulatedCas, out _, false);
            if (CanDump)
                DumpFile(path, stream);
            return true;
        }

        public bool TryCreateIoStoreContainer(string path, ref IEmulatedFile? emulated, out Strim? stream, bool bEmulatedFile)
        {
            stream = null;
            nint blockCount = 0;
            nint blockPtr = 0;
            nint headerSize = 0;
            nint headerPtr = 0;
            if (bEmulatedFile) _pathToStream[path] = null; // emulated file locks up at FPackageStore::LoadContainers, write to file for now
            if (!RustApi.GetContainerBlocks(path, ref blockPtr, ref blockCount, ref headerPtr, ref headerSize)) return false;
            //_logger.Info($"[UtocEmulator] Created Container File, {blockCount} blocks at 0x{blockPtr:X}, header size {headerSize} at 0x{headerPtr:X}");
            var streams = new List<StreamOffsetPair<Strim>>();
            long streamEnd = 0;
            for (int i = 0; i < blockCount; i++)
            {
                var containerBlock = Marshal.PtrToStructure<PartitionBlock>(blockPtr);
                streams.Add(new(
                    new FileStream(Marshal.PtrToStringAnsi(containerBlock.osPath), FileMode.Open),
                    OffsetRange.FromStartAndLength(containerBlock.start, containerBlock.length)
                ));
                var containerBlockEnd = containerBlock.start + containerBlock.length;
                var diff = Mathematics.RoundUp(containerBlockEnd, DefaultCompressionBlockAlignment) - containerBlockEnd;
                if (diff > 0)
                    streams.Add(new(new PaddingStream(0, (int)diff), OffsetRange.FromStartAndLength(containerBlockEnd, diff)));
                unsafe { blockPtr += sizeof(PartitionBlock); }
                streamEnd = Mathematics.RoundUp(containerBlockEnd, DefaultCompressionBlockAlignment);
            }
            unsafe
            {
                streams.Add(new(
                    new UnmanagedMemoryStream((byte*)headerPtr, headerSize),
                    OffsetRange.FromStartAndLength(streamEnd, (long)headerSize)
                ));
            }
            stream = new MultiStream(streams, _logger);
            if (bEmulatedFile)
            {
                _pathToStream.TryAdd(path, stream);
                emulated = new EmulatedFile<Strim>(stream);
                _logger.Info($"[UtocEmulator] Created Emulated Container File with Path {path}");
            } else
            {
                WriteContainer(path, stream);
            }
            if (CanDump)
                DumpFile(path, stream);
            return true;
        }

        /// <summary>
        /// Tries to create an emulated file from a given file handle.
        /// </summary>
        /// <param name="handle">Handle of the file where the data is sourced from.</param>
        /// <param name="srcDataPath">Path of the file where the handle refers to.</param>
        /// <param name="outputPath">Path where the emulated file is stored.</param>
        /// <param name="route">The route of the emulated file, for builder to pick up.</param>
        /// <param name="emulated">The emulated file.</param>
        /// <param name="stream">The created stream under the hood.</param>
        /// <returns>True if an emulated file could be created, false otherwise</returns>
        public bool TryCreateEmulatedFile(IntPtr handle, string srcDataPath, string outputPath, string route, ref IEmulatedFile? emulated, out Strim? stream)
        {
            stream = null;
            if (srcDataPath.Contains(DumpFolderParent)) return false;
            string? ext = Path.GetExtension(srcDataPath);
            if (srcDataPath.EndsWith(UtocExtension, StringComparison.OrdinalIgnoreCase))
            {
                if (TryCreateIoStoreTOC(srcDataPath, ref emulated!, out _)) return true;
            } 
            else if (srcDataPath.EndsWith(UcasExtension, StringComparison.OrdinalIgnoreCase))
            {
                //if (TryCreateIoStoreContainer(srcDataPath, ref emulated!, out _, true)) return true;
            }
            return false;
        }

        private void DumpFile(string filepath, Strim stream)
        {
            var filePath = Path.GetFullPath($"{Path.Combine(DumpFolderParent, DumpFolderToc, Path.GetFileName(filepath))}");
            Directory.CreateDirectory(Path.Combine(DumpFolderParent, DumpFolderToc));
            _logger.Info($"[UtocEmulator] Dumping {filepath}");
            using var fileStream = new FileStream(filePath, FileMode.Create);
            stream.CopyTo(fileStream);
            _logger.Info($"[UtocEmulator] Written To {filePath}");
        }

        private void WriteContainer(string path, Strim stream)
        {
            _logger.Info($"[UtocEmulator] Writing container {path}, size {stream.Length}");
            using var fileStream = new FileStream(path, FileMode.Create);
            stream.CopyTo(fileStream);
            _logger.Info($"[UtocEmulator] Container Written To {path}");
        }

        public void OnModLoading(string mod_id, string dir_path) => RustApi.AddFromFolders(mod_id, dir_path);
        public void OnLoaderInit() => RustApi.PrintAssetCollectorResults();
    }
}
