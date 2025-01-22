using UnrealEssentials.Interfaces;
using UnrealEssentials.Unreal;

namespace UnrealEssentials;
public unsafe class Api : IUnrealEssentials
{
    public void* Malloc(nuint count, uint alignment = 0) => UnrealMemory.Malloc(count, alignment);

    public void* TryMalloc(nuint count, uint alignment = 0) => UnrealMemory.TryMalloc(count, alignment);

    public void* Realloc(void* original, nuint count, uint alignment = 0) => UnrealMemory.Realloc(original, count, alignment);

    public void* TryRealloc(void* original, nuint count, uint alignment = 0) => UnrealMemory.TryRealloc(original, count, alignment);

    public void Free(void* original) => UnrealMemory.Free(original);

    public bool GetAllocationSize(void* original, nuint* sizeOut) => UnrealMemory.GetAllocationSize(original, sizeOut);

    private Action<string> _addFolder;

    private Action<string, string?> _addFile;

    private Action<string, string> _addFolderWithCustomPath;

    internal Api(Action<string> addFolder, Action<string, string> addFolderWithCustomPath, Action<string, string> addFile)
    {
        _addFolder = addFolder;
        _addFolderWithCustomPath = addFolderWithCustomPath;
        _addFile = addFile;
    }

    public void AddFromFolder(string path) => _addFolder(path);

    public void AddFolderWithCustomPath(string path, string gamePath) => _addFolderWithCustomPath(path, gamePath);

    public void AddFromFile(string path, string? gamePath) => _addFile(path, gamePath);
}
