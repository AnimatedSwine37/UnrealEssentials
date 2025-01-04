using UnrealEssentials.Interfaces;
using UnrealEssentials.Unreal;

namespace UnrealEssentials;
public unsafe class Api : IUnrealEssentials
{
    public void* Malloc(nuint Count, uint Alignment = 0) => UnrealMemory.Malloc(Count, Alignment);

    public void* TryMalloc(nuint Count, uint Alignment = 0) => UnrealMemory.TryMalloc(Count, Alignment);

    public void* Realloc(void* Original, nuint Count, uint Alignment = 0) => UnrealMemory.Realloc(Original, Count, Alignment);

    public void* TryRealloc(void* Original, nuint Count, uint Alignment = 0) => UnrealMemory.TryRealloc(Original, Count, Alignment);

    public void Free(void* Original) => UnrealMemory.Free(Original);

    public nuint QuantizeSize(nuint Count, uint Alignment) => UnrealMemory.QuantizeSize(Count, Alignment);

    public bool GetAllocationSize(void* Original, nuint* SizeOut) => UnrealMemory.GetAllocationSize(Original, SizeOut);

    public void Trim(bool bTrimThreadCaches) => UnrealMemory.Trim(bTrimThreadCaches);

    private Action<string> _addFolder;

    internal Api(Action<string> addFolder)
    {
        _addFolder = addFolder;
    }

    public void AddFromFolder(string path) => _addFolder(path);
}
