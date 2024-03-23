using UnrealEssentials.Interfaces;

namespace UnrealEssentials;
public class Api : IUnrealEssentials
{
    private Action<string> _addFolder;

    internal Api(Action<string> addFolder)
    {
        _addFolder = addFolder;
    }

    public void AddFromFolder(string path) => _addFolder(path);
}
