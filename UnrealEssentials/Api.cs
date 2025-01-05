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

    internal Api(Action<string> addFolder)
    {
        _addFolder = addFolder;
    }

    public void AddFromFolder(string path) => _addFolder(path);
}
