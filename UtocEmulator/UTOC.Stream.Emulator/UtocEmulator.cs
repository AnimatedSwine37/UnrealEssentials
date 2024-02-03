using FileEmulationFramework.Interfaces;
using FileEmulationFramework.Interfaces.Reference;
using FileEmulationFramework.Lib.IO;
using FileEmulationFramework.Lib.IO.Struct;
using FileEmulationFramework.Lib.Utilities;
using System.Collections.Concurrent;
using System.Runtime.InteropServices;
using UTOC.Stream.Emulator.Interfaces;
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
        public PakType PakVersion { get; set; }
        public Strim? TocStream { get; set; }
        public Strim? CasStream { get; set; }
        private string ModPath { get; init; }
        private string ModTargetFilesDirectory { get; init; }
        private string ModDummyPakFilesDirectory { get; init; }
        public Action<string> AddPakFolderCb { get; set; }

        private readonly ConcurrentDictionary<string, Strim?> _pathToStream = new(StringComparer.OrdinalIgnoreCase);

        public UtocEmulator(Logger logger, bool canDump, string modPath, Action<string> addPakFolderCb) 
        { 
            _logger = logger; 
            DumpFiles = canDump;
            ModPath = modPath;
            ModTargetFilesDirectory = Path.Combine(ModPath, Constants.TargetDir);
            ModDummyPakFilesDirectory = Path.Combine(ModPath, Constants.DummyPakDir);
            AddPakFolderCb = addPakFolderCb;
        }

        public bool TryCreateFile(IntPtr handle, string filepath, string route, out IEmulatedFile emulated)
        {
            emulated = null!;
            if (TocVersion == null) return false; // This game's version is too old for IO Store, quit here
            // Check if we've already made a custom UTOC
            if (_pathToStream.TryGetValue(filepath, out var stream))
            {
                if (stream == null) return false; // Avoid recursion into the same file
                emulated = new EmulatedFile<Strim>(stream);
                return true;

            }
            // Check extension and path
            if (!TryCreateEmulatedFile(handle, filepath, filepath, filepath, ref emulated!, out _)) return false;
            return true;
        }
        public bool TryCreateIoStoreTOC(string path, ref IEmulatedFile? emulated, out Strim? stream)
        {
            stream = null;
            _pathToStream[path] = null; // Avoid recursion into the same file
            if (!path.Contains(ModTargetFilesDirectory) || TocStream == null) return false;
            stream = TocStream;
            _pathToStream.TryAdd(path, stream);
            emulated = new EmulatedFile<Strim>(stream);
            _logger.Info($"[UtocEmulator] Created Emulated Table of Contents with Path {path}");
            if (DumpFiles)
                DumpFile(path, stream);
            return true;
        }

        public List<StreamOffsetPair<Strim>> CreateContainerStream(nint blockPtr, int blockCount, nint headerPtr, int headerSize)
        {
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

        public bool TryCreateIoStoreContainer(IntPtr handle, string path, ref IEmulatedFile? emulated, out Strim? stream)
        {
            stream = null;
            _pathToStream[path] = null;
            if (!path.Contains(ModTargetFilesDirectory) || CasStream == null) return false;
            stream = CasStream;
            _pathToStream.TryAdd(path, stream);
            emulated = new EmulatedFile<Strim>(stream);
            _logger.Info($"[UtocEmulator] Created Emulated Container with Path {path}");
            if (DumpFiles)
                DumpFile(path, stream);
            return true;
        }

        private Strim GetDummyPak()
        {
            // Assumed to only be FrozenIndex or Fn64BugFix (only Pak versions to have IO Store support)
            if (PakVersion == PakType.FrozenIndex)
            {
                _logger.Info($"[UtocEmulator] Using Pak Type FrozenIndex");
                return new FileStream(Path.Combine(ModDummyPakFilesDirectory, $"FrozenIndex{Constants.PakExtension}"), FileMode.Open);
            }
            _logger.Info($"[UtocEmulator] Using Pak Type Fn64BugFix");
            return new FileStream(Path.Combine(ModDummyPakFilesDirectory, $"Fn64BugFix{Constants.PakExtension}"), FileMode.Open);
        }

        public bool TryCreateDummyPak(string path, ref IEmulatedFile? emulated, out Strim? stream)
        {
            stream = null;
            _pathToStream[path] = null;
            if (!path.Contains(ModTargetFilesDirectory)) return false;
            stream = GetDummyPak();
            _pathToStream.TryAdd(path, stream);
            emulated = new EmulatedFile<Strim>(stream);
            _logger.Info($"[UtocEmulator] Created Emulated IO Store PAK with Path {path}");
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
            {
                if (TryCreateIoStoreTOC(srcDataPath, ref emulated!, out _)) return true;
            }
            else if (srcDataPath.EndsWith(Constants.UcasExtension, StringComparison.OrdinalIgnoreCase))
            {
                if (TryCreateIoStoreContainer(handle, srcDataPath, ref emulated!, out _)) return true;
            }
            else if (srcDataPath.EndsWith(Constants.PakExtension, StringComparison.OrdinalIgnoreCase))
            {
                if (TryCreateDummyPak(srcDataPath, ref emulated!, out _)) return true;
            }
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
            using var fileStream = new FileStream(path, FileMode.Create);
            stream.CopyTo(fileStream);
            _logger.Info($"[UtocEmulator] Container Written To {path}");
        }

        public void OnModLoading(string mod_id, string dir_path) => RustApi.AddFromFolders(mod_id, Path.Combine(dir_path, "UTOC", "UnrealEssentials.utoc"));

        public void AddFromFolder(string mod_id, string dir_path) => RustApi.AddFromFolders(mod_id, dir_path);

        public void MakeFilesOnInit() // from base Unreal Essentials path
        {
            if (TocVersion == null)
            {
                _logger.Info($"[UtocEmulator] Toc Version was not provided, stopping here");
                return;
            }
            if (PakVersion != PakType.FrozenIndex && PakVersion != PakType.Fn64BugFix)
            {
                _logger.Info($"[UtocEmulator] Pak version {PakVersion} is too old, stopping here");
                return;
            }
            nint tocLength = 0;
            nint tocData = 0;
            nint blockPtr = 0;
            nint blockCount = 0;
            nint headerPtr = 0;
            nint headerSize = 0;
            var result = RustApi.BuildTableOfContentsEx(
                ModTargetFilesDirectory, (uint)TocVersion, ref tocData, ref tocLength,
                ref blockPtr, ref blockCount, ref headerPtr, ref headerSize
            );
            if (!result)
            {
                _logger.Info($"[UtocEmulator] An error occurred while making IO Store data");
                return;
            }
            unsafe
            {
                TocStream = new UnmanagedMemoryStream((byte*)tocData, (long)tocLength);
                CasStream = new MultiStream(CreateContainerStream(blockPtr, (int)blockCount, headerPtr, (int)headerSize), _logger);
            }
            AddPakFolderCb(ModTargetFilesDirectory);
        }
        public void OnLoaderInit()
        {
            RustApi.PrintAssetCollectorResults();
            MakeFilesOnInit();
        }
    }
}
