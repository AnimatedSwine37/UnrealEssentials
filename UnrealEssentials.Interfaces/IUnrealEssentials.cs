namespace UnrealEssentials.Interfaces;
public unsafe interface IUnrealEssentials
{
    /// <summary>
    /// Adds files from the folder at <paramref name="path"/>. 
    /// This folder is treated like it was the UnrealEssentials folder inside of a mod.
    /// </summary>
    /// <param name="path">Path to the folder that contains files to be loaded.</param>
    void AddFromFolder(string path);

    //!!! WARNING FOR ALL MEMORY RELATED FUNCTIONS !!!
    //THEY SHOULD ONLY BE CALLED AFTER THE GAME HAS LAUNCHED. TRYING TO USE THEM BEFORE THE GAME HAS LAUNCHED WILL RESULT IN A CRASH.

    /// <summary>
    /// Allocates a piece of memory. <paramref name="Count" name="Alignment"/>. 
    /// </summary>
    /// <param name="Count">Number of bytes to allocate.</param>
    /// <param name="Alignment">Alignment of the allocation.</param>
    void* Malloc(nuint Count, uint Alignment = 0);

    /// <summary>
    /// Allocates a piece of memory. <paramref name="Count" name="Alignment"/>. 
    /// </summary>
    /// <param name="Count">Number of bytes to allocate.</param>
    /// <param name="Alignment">Alignment of the allocation.</param>
    void* TryMalloc(nuint Count, uint Alignment = 0);

    /// <summary>
    /// Reallocates a piece of memory. <paramref name="Count" name="Alignment" name="Original"/>. 
    /// </summary>
    /// <param name="Original">Pointer to the original memory.</param>
    /// <param name="Count">Number of bytes to allocate.</param>
    /// <param name="Alignment">Alignment of the allocation.</param>
    void* Realloc(void* Original, nuint Count, uint Alignment = 0);

    /// <summary>
    /// Reallocates a piece of memory. <paramref name="Count" name="Alignment" name="Original"/>. 
    /// </summary>
    /// <param name="Original">Pointer to the original memory.</param>
    /// <param name="Count">Number of bytes to allocate.</param>
    /// <param name="Alignment">Alignment of the allocation.</param>
    void* TryRealloc(void* Original, nuint Count, uint Alignment = 0);

    /// <summary>
    /// Frees a piece of memory. <paramref name="Original"/>. 
    /// </summary>
    /// <param name="Original">Pointer to the original memory.</param>
    void Free(void* Original);

    nuint QuantizeSize(nuint Count, uint Alignment);

    bool GetAllocationSize(void* Original, nuint* SizeOut);

    void Trim(bool bTrimThreadCaches);
}
