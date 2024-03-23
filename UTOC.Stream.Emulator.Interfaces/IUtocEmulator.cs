namespace UTOC.Stream.Emulator.Interfaces;
public interface IUtocEmulator
{
    public void Initialise(TocType? tocType, PakType pakType, string fileIoStoreSig, string readBlockSig, 
        Action<string> addPakFolder, Action<string> removePakFolder);

    public void AddFromFolder(string folder);
}
