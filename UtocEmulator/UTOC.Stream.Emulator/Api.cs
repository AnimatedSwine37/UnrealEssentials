using UTOC.Stream.Emulator.Interfaces;

namespace UTOC.Stream.Emulator;

public class Api : IUtocEmulator
{
    private InitialiseDelegate _initialise;
    private Action<string> _addFromFolder;
    private Action<string, string> _addFromFolderWithMount;

    internal Api(InitialiseDelegate initialise, Action<string> addFromFolder, Action<string, string> addFromFolderWithMount)
    {
        _initialise = initialise;
        _addFromFolder = addFromFolder;
        _addFromFolderWithMount = addFromFolderWithMount;
    }

    public void AddFromFolder(string folder) => _addFromFolder(folder);

    public void AddFromFolderWithMount(string folder, string virtualPath) => _addFromFolderWithMount(folder, virtualPath);

    public void Initialise(EngineVersion engineVersion, bool hasUtocs, Action<string> addPakFolder, Action<string> removePakFolder)
    {
        _initialise(engineVersion, hasUtocs, addPakFolder, removePakFolder);
    }

    internal delegate void InitialiseDelegate(EngineVersion engineVersion, bool hasUtocs, Action<string> addPakFolder, Action<string> removePakFolder);
}
