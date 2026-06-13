using FileEmulationFramework.Interfaces;
using FileEmulationFramework.Interfaces.Reference;
using FileEmulationFramework.Lib.IO;
using FileEmulationFramework.Lib.IO.Struct;
using FileEmulationFramework.Lib.Utilities;
using System.Collections.Concurrent;
using System.Diagnostics;
using System.Runtime.InteropServices;
using UTOC.Stream.Emulator.Configuration;
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
        public Config _configuration { get; set; }
        public Logger _logger { get; init; }
        // public TocType? TocVersion { get; set; }
        // public PakType PakVersion { get; set; }
        public EngineVersion EngineVersion { get; set; }
        public bool HasUtocs { get; set; }
        public Strim? TocStream { get; set; }
        public Strim? CasStream { get; set; }
        private string ModPath { get; init; }
        private string ModTargetFilesDirectory { get; init; }
        private string ModDummyPakFilesDirectory { get; init; }
        public Action<string> AddPakFolderCb { get; set; }

        private readonly ConcurrentDictionary<string, Strim?> _pathToStream = new(StringComparer.OrdinalIgnoreCase);

        public UtocEmulator(Logger logger, Config configuration, string modPath, Action<string> addPakFolderCb) 
        {
            _logger = logger; 
            _configuration = configuration;
            ModPath = modPath;
            ModTargetFilesDirectory = Path.Combine(ModPath, Constants.TargetDir);
            ModDummyPakFilesDirectory = Path.Combine(ModPath, Constants.DummyPakDir);
            AddPakFolderCb = addPakFolderCb;
        }

        public bool TryCreateFile(IntPtr handle, string filepath, string route, out IEmulatedFile emulated)
        {
            emulated = null!;
            if (!HasUtocs) return false; // This game's version is too old for IO Store, quit here
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
            _pathToStream[path] = stream;
            emulated = new EmulatedFile<Strim>(stream);
            _logger.LogInfo($"Created Emulated Table of Contents with Path {path}");
            if (_configuration.DumpFiles)
                DumpFile(path, stream);
            return true;
        }

        public List<StreamOffsetPair<Strim>> CreateContainerStream(nint blockPtr, int blockCount, nint headerPtr, int headerSize)
        {
            var streams = new List<StreamOffsetPair<Strim>>();
            long streamEnd = 0;
            for (var i = 0; i < blockCount; i++)
            {
                var containerBlock = Marshal.PtrToStructure<PartitionBlock>(blockPtr);
                // _logger.LogInfo($"{Marshal.PtrToStringUni(containerBlock.osPath)}: {containerBlock.start:x} -> {containerBlock.start + containerBlock.length:x}");
                streams.Add(new(
                    new FileStream(Marshal.PtrToStringUni(containerBlock.osPath), FileMode.Open),
                    OffsetRange.FromStartAndLength(containerBlock.start, containerBlock.length)
                ));
                var containerBlockEnd = containerBlock.start + containerBlock.length;
                if (!_configuration.UseNewEmulator)
                {
                    var diff = Mathematics.RoundUp(containerBlockEnd, Constants.DefaultCompressionBlockAlignment) - containerBlockEnd;
                    if (diff > 0)
                        streams.Add(new(new PaddingStream(0, (int)diff), OffsetRange.FromStartAndLength(containerBlockEnd, diff)));   
                }
                unsafe { blockPtr += sizeof(PartitionBlock); }

                streamEnd = _configuration.UseNewEmulator ? containerBlockEnd : Mathematics.RoundUp(containerBlockEnd, Constants.DefaultCompressionBlockAlignment);
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
            _pathToStream[path] = stream;
            emulated = new EmulatedFile<Strim>(stream);
            _logger.LogInfo($"Created Emulated Container with Path {path}");
            if (_configuration.DumpFiles)
                DumpFile(path, stream);
            return true;
        }

        private Strim GetDummyPak()
        {
            // Assumed to only be FrozenIndex or Fn64BugFix (only Pak versions to have IO Store support)
            if (EngineVersion == EngineVersion.UE_4_25)
            {
                _logger.LogInfo("Using Pak Type FrozenIndex");
                return new FileStream(Path.Combine(ModDummyPakFilesDirectory, $"FrozenIndex{Constants.PakExtension}"), FileMode.Open);
            }
            _logger.LogInfo("Using Pak Type Fn64BugFix");
            return new FileStream(Path.Combine(ModDummyPakFilesDirectory, $"Fn64BugFix{Constants.PakExtension}"), FileMode.Open);
        }

        public bool TryCreateDummyPak(string path, ref IEmulatedFile? emulated, out Strim? stream)
        {
            stream = null;
            _pathToStream[path] = null;
            if (!path.Contains(ModTargetFilesDirectory)) return false;
            stream = GetDummyPak();
            _pathToStream[path] = stream;
            emulated = new EmulatedFile<Strim>(stream);
            _logger.LogInfo($"Created Emulated IO Store PAK with Path {path}");
            if (_configuration.DumpFiles)
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
            if (!HasUtocs) return false; // This game's version is too old for IO Store, quit here
            if (srcDataPath.Contains(Constants.DumpFolderParent)) return false;
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
            _logger.LogInfo($"Dumping {filepath}");
            using var fileStream = new FileStream(filePath, FileMode.Create);
            stream.CopyTo(fileStream);
            _logger.LogInfo($"Written To {filePath}");
        }

        private void AddFromFolderInner(string mod_path)
        {
            var mod_path_unicode = Marshal.StringToHGlobalUni(mod_path);
            if (_configuration.UseNewEmulator)
                RustApiNew.add_from_folders(mod_path_unicode, EngineVersion);
            else
            {
                RustApi.AddFromFolders(mod_path_unicode, mod_path.Length);
                Marshal.FreeHGlobal(mod_path_unicode);   
            }
        }

        public void OnModLoading(string dir_path)
            => AddFromFolderInner(Path.Combine(dir_path, "UTOC", "UnrealEssentials.utoc"));

        public void AddFromFolder(string dir_path)
            => AddFromFolderInner(dir_path);

        public void AddFromFolderWithMount(string dir_path, string virtual_path)
        {
            var mod_path_unicode = Marshal.StringToHGlobalUni(dir_path);
            var virtual_path_unicode = Marshal.StringToHGlobalUni(virtual_path);
            if (_configuration.UseNewEmulator)
                RustApiNew.add_from_folders_with_mount(mod_path_unicode, virtual_path_unicode, EngineVersion);
            else
            {
                RustApi.AddFromFoldersWithMount(mod_path_unicode, dir_path.Length, virtual_path_unicode, virtual_path.Length);
                Marshal.FreeHGlobal(mod_path_unicode);
                Marshal.FreeHGlobal(virtual_path_unicode);
            }
        }

        private void MakeFilesOnInitOld()
        {
            var PakVersion = EngineVersion switch
            {
                EngineVersion.UE_4_25 => PakType.FrozenIndex,
                EngineVersion.UE_4_26 => PakType.Fn64BugFix,
                EngineVersion.UE_4_27 => PakType.Fn64BugFix,
                _ => PakType.NoTimestamps
            };
            TocType? TocVersion = EngineVersion switch
            {
                EngineVersion.UE_4_25 => TocType.Initial,
                EngineVersion.UE_4_26 => TocType.DirectoryIndex,
                EngineVersion.UE_4_27 => TocType.PartitionSize,
                _ => null
            };
            if (PakVersion != PakType.FrozenIndex && PakVersion != PakType.Fn64BugFix)
            {
                _logger.LogInfo($"Pak version {PakVersion} is too old, stopping here");
                return;
            }
            if (TocVersion == null)
            {
                _logger.LogInfo("TocVesrion unavailable, stopping here");
                return;               
            }
            nint tocLength = 0;
            nint tocData = 0;
            nint blockPtr = 0;
            nint blockCount = 0;
            nint headerPtr = 0;
            nint headerSize = 0;
            nint mod_path_unicode = Marshal.StringToHGlobalUni(ModTargetFilesDirectory);
            var result = RustApi.BuildTableOfContentsEx(mod_path_unicode, ModTargetFilesDirectory.Length
                , (uint)TocVersion, ref tocData, ref tocLength,
                ref blockPtr, ref blockCount, ref headerPtr, ref headerSize
            );
            Marshal.FreeHGlobal(mod_path_unicode);
            if (!result)
            {
                _logger.LogError("An error occurred while making IO Store data");
                return;
            }
            if(blockCount == 0)
            {
                _logger.LogInfo("No IO store files found, not creating emulated file.");
                return;
            }
            unsafe
            {
                TocStream = new UnmanagedMemoryStream((byte*)tocData, (long)tocLength);
                CasStream = new MultiStream(CreateContainerStream(blockPtr, (int)blockCount, headerPtr, (int)headerSize), _logger);
            }
            AddPakFolderCb(ModTargetFilesDirectory);
        }

        private unsafe void MakeFilesOnInitNew()
        {
            var toc = (Array<byte>*)NativeMemory.AlignedAlloc((nuint)(3 * sizeof(Array<byte>)), (nuint)sizeof(nint));
            var blocks = (Array<PartitionBlock>*)(toc + 1);
            var header = toc + 2;
            var result = RustApiNew.build_toc(
                    // Marshal.StringToHGlobalUni(ModTargetFilesDirectory),
                    EngineVersion, toc, blocks, header);
            if (!result)
            {
                _logger.LogError("An error occurred while making IO Store data");
                return;
            }

            if (blocks->Len == 0)
            {
                _logger.LogInfo("No IO store files found, not creating emulated file.");
                return;   
            }
            TocStream = new UnmanagedMemoryStream(toc->Entries, toc->Len);
            CasStream = new MultiStream(CreateContainerStream((nint)blocks->Entries, (int)blocks->Len,
                (nint)header->Entries, (int)header->Len));
            AddPakFolderCb(ModTargetFilesDirectory);
        }

        public void MakeFilesOnInit() // from base Unreal Essentials path
        {
            if (!HasUtocs)
            {
                _logger.LogInfo("Game is not using IO Store, stopping here");
                return;
            }

            if (_configuration.UseNewEmulator)
                MakeFilesOnInitNew();
            else
                MakeFilesOnInitOld();
        }
        public void OnLoaderInit()
        {
            if (!_configuration.UseNewEmulator)
                RustApi.PrintAssetCollectorResults();
            MakeFilesOnInit();
        }
    }
}
