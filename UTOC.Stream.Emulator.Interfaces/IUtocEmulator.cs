namespace UTOC.Stream.Emulator.Interfaces;
public interface IUtocEmulator
{
    public void Initialise(TocType? tocType, PakType pakType, Action<string> addPakFolder, Action<string> removePakFolder);

    public void AddFromFolder(string folder);

    public void AddFromFolderWithMount(string folder, string virtualfolder);
}
