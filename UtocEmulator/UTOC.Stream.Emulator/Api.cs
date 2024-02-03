using UTOC.Stream.Emulator.Interfaces;

namespace UTOC.Stream.Emulator;

public class Api : IUtocEmulator
{
    private InitialiseDelegate _initialise;
    private Action<string, string> _addFromFolder;

    internal Api(InitialiseDelegate initialise, Action<string, string> addFromFolder)
    {
        _initialise = initialise;
        _addFromFolder = addFromFolder;
    }

    public void AddFromFolder(string modId, string folder)
    {
        _addFromFolder(modId, folder);
    }

    public void Initialise(TocType? tocType, PakType pakType, string fileIoStoreSig, string readBlockSig, Action<string> addPakFolder, Action<string> removePakFolder)
    {
        _initialise(tocType, pakType, fileIoStoreSig, readBlockSig, addPakFolder, removePakFolder);
    }

    internal delegate void InitialiseDelegate(TocType? tocType, PakType pakType, string fileIoStoreSig, string readBlockSig, Action<string> addPakFolder, Action<string> removePakFolder);
}
