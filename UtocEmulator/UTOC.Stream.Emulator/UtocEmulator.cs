using FileEmulationFramework.Interfaces;
using FileEmulationFramework.Interfaces.Reference;
using FileEmulationFramework.Lib;
using FileEmulationFramework.Lib.IO;
using FileEmulationFramework.Lib.IO.Struct;
using FileEmulationFramework.Lib.Utilities;
using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading.Tasks;
using UnrealEssentials.Interfaces;
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
        public bool DumpFiles { get; set; }
        public Logger _logger { get; init; }
        public TocType? TocVersion { get; set; }
        public Strim? TocStream { get; set; }

        private readonly ConcurrentDictionary<string, Strim?> _pathToStream = new(StringComparer.OrdinalIgnoreCase);

        public UtocEmulator(Logger logger, bool canDump) { _logger = logger; DumpFiles = canDump; }

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
            _logger.Info($"TOC TRY {path}");
            stream = null;
            _pathToStream[path] = null; // Avoid recursion into the same file
            if (!path.Contains($"{Constants.UnrealEssentialsName}\\Unreal")) return false;
            stream = TocStream;
            _logger.Info($"Stream details: length {stream.Length}, position {stream.Position}");
            emulated = new EmulatedFile<Strim>(stream);
            _logger.Info($"[UtocEmulator] Created Emulated Table of Contents with Path {path}");
            if (DumpFiles)
                DumpFile(path, stream);
            return true;
        }

        public List<StreamOffsetPair<Strim>> CreateContainerStream(nint blockPtr, int blockCount, nint headerPtr, int headerSize)
        {
            _logger.Info($"Block: 0x{blockPtr:X}, count {blockCount}, header 0x{headerPtr:X}, size 0x{headerSize:X}");
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
                var diff = Mathematics.RoundUp(containerBlockEnd, Constants.DefaultCompressionBlockAlignment) - containerBlockEnd;
                if (diff > 0)
                    streams.Add(new(new PaddingStream(0, (int)diff), OffsetRange.FromStartAndLength(containerBlockEnd, diff)));
                unsafe { blockPtr += sizeof(PartitionBlock); }
                streamEnd = Mathematics.RoundUp(containerBlockEnd, Constants.DefaultCompressionBlockAlignment);
            }
            unsafe
            {
                streams.Add(new(
                    new UnmanagedMemoryStream((byte*)headerPtr, headerSize),
                    OffsetRange.FromStartAndLength(streamEnd, (long)headerSize)
                ));
            }
            return streams;
        }

        public bool TryCreateIoStoreContainer(string path, ref IEmulatedFile? emulated, out Strim? stream, bool bEmulatedFile)
        {
            stream = null;
            nint blockCount = 0;
            nint blockPtr = 0;
            nint headerSize = 0;
            nint headerPtr = 0;
            if (bEmulatedFile) _pathToStream[path] = null; // emulated file currently produces invalid size values, write to file for now
            if (!RustApi.GetContainerBlocks(path, ref blockPtr, ref blockCount, ref headerPtr, ref headerSize)) return false;
            stream = new MultiStream(CreateContainerStream(blockPtr, (int)blockCount, headerPtr, (int)headerSize), _logger);
            if (bEmulatedFile)
            {
                _pathToStream.TryAdd(path, stream);
                emulated = new EmulatedFile<Strim>(stream);
                _logger.Info($"[UtocEmulator] Created Emulated Container File with Path {path}");
            } else
            {
                WriteContainer(path, stream);
            }
            if (DumpFiles)
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
            if (TocVersion == null) return false; // This game's version is too old for IO Store, quit here
            if (srcDataPath.Contains(Constants.DumpFolderParent)) return false;
            string? ext = Path.GetExtension(srcDataPath);
            if (srcDataPath.EndsWith(Constants.UtocExtension, StringComparison.OrdinalIgnoreCase))
                if (TryCreateIoStoreTOC(srcDataPath, ref emulated!, out _)) return true;
            return false;
        }

        private void DumpFile(string filepath, Strim stream)
        {
            var filePath = Path.GetFullPath($"{Path.Combine(Constants.DumpFolderParent, Constants.DumpFolderToc, Path.GetFileName(filepath))}");
            Directory.CreateDirectory(Path.Combine(Constants.DumpFolderParent, Constants.DumpFolderToc));
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

        public void MakeFilesOnInit(string essentialsPath)
        {
            if (TocVersion == null) return;
            var targetDirectory = Path.Combine(essentialsPath, "Unreal");
            nint tocLength = 0;
            nint tocData = 0;
            nint blockPtr = 0;
            nint blockCount = 0;
            nint headerPtr = 0;
            nint headerSize = 0;
            var result = RustApi.BuildTableOfContentsEx(
                targetDirectory, (uint)TocVersion, ref tocData, ref tocLength,
                ref blockPtr, ref blockCount, ref headerPtr, ref headerSize
            );
            if (!result)
            {
                _logger.Info($"[UtocEmulator] An error occurred while making IO Store data");
                return;
            }
            unsafe
            {
                _logger.Info($"Stream for {Path.Combine(targetDirectory, $"{Constants.UnrealEssentialsName}.utoc")}");
                TocStream = new UnmanagedMemoryStream((byte*)tocData, (long)tocLength);
                var casStreams = CreateContainerStream(blockPtr, (int)blockCount, headerPtr, (int)headerSize);
                WriteContainer(Path.Combine(targetDirectory, $"{Constants.UnrealEssentialsName}.ucas"), new MultiStream(casStreams, _logger));
            }
        }
        public void OnLoaderInit(string essentialsPath)
        {
            RustApi.PrintAssetCollectorResults();
            MakeFilesOnInit(essentialsPath);
        }
    }
}
