namespace UnrealEssentials.Interfaces;
public unsafe interface IUnrealEssentials
{
    /// <summary>
    /// Adds files from the folder at <paramref name="path"/> and bind it to a custom game path using <paramref name="gamePath"/> 
    /// This folder is treated like it was the UnrealEssentials folder inside of a mod
    /// </summary>
    /// <param name="path">Path to the folder that contains files to be loaded</param>
    /// <param name="gamePath">The folder in the game's files to which the contents of the folder added by this api will be added to.</param>
    void AddFolderWithCustomPath(string path, string gamePath);

    /// <summary>
    /// Adds a file at <paramref name="path"/> and bind it to a custom game path using <paramref name="gamePath"/> 
    /// This file is treated like it was inside the UnrealEssentials folder inside of a mod
    /// </summary>
    /// <param name="path">Path to the file to be loaded</param>
    /// <param name="gamePath">The folder in the game's files to which the contents of the folder added by this api will be added to.</param>
    void AddFromFile(string path, string? gamePath);

    /// <summary>
    /// Adds files from the folder at <paramref name="path"/> 
    /// This folder is treated like it was the UnrealEssentials folder inside of a mod
    /// </summary>
    /// <param name="path">Path to the folder that contains files to be loaded</param>
    void AddFromFolder(string path);

    /// <summary>
    /// Allocates a piece of memory
    /// </summary>
    /// <param name="count">Number of bytes to allocate</param>
    /// <param name="alignment">Alignment of the allocation</param>
    /// <remarks>
    /// Should only be used once the game has finished launching
    /// </remarks>
    /// <returns>Returns a pointer to the beginning of the new memory block</returns>
    void* Malloc(nuint count, uint alignment = 0);

    /// <summary>
    /// Similar to Malloc(), but may return a nullptr result if the allocation request cannot be satisfied
    /// </summary>
    /// <param name="count">Number of bytes to allocate.</param>
    /// <param name="alignment">Alignment of the allocation.</param>
    /// <remarks>
    /// Should only be used once the game has finished launching.
    /// </remarks>
    /// <returns>Returns a pointer to the beginning of the new memory block. If the allocation fails, returns a nullptr</returns>
    void* TryMalloc(nuint count, uint alignment = 0);

    /// <summary>
    /// Resizes a previously allocated block of memory, preserving its contents
    /// </summary>
    /// <param name="original">Pointer to the original memory.</param>
    /// <param name="count">Number of bytes to allocate.</param>
    /// <param name="alignment">Alignment of the allocation.</param>
    /// <remarks>
    /// Should only be used once the game has finished launching.
    /// </remarks>
    /// <returns>Returns a pointer to the beginning of the new memory block</returns>
    void* Realloc(void* original, nuint count, uint alignment = 0);

    /// <summary>
    /// Similar to Realloc(), but may return a nullptr if the allocation request cannot be satisfied
    /// </summary>
    /// <param name="original">Pointer to the original memory.</param>
    /// <param name="count">Number of bytes to allocate.</param>
    /// <param name="alignment">Alignment of the allocation.</param>
    /// <remarks>
    /// Should only be used once the game has finished launching.
    /// </remarks>
    /// <returns>Returns a pointer to the beginning of the new memory block. If the allocation fails, returns a nullptr</returns>
    void* TryRealloc(void* original, nuint count, uint alignment = 0);

    /// <summary>
    /// Deallocates a piece of memory
    /// </summary>
    /// <param name="original">Pointer to the original memory.</param>
    /// <remarks>
    /// Should only be used once the game has finished launching.
    /// </remarks>
    void Free(void* original);

    ///<summary>
    ///If possible, determines the size of the memory allocated at the given address
    /// </summary>
    /// <param name="original">Pointer to memory we are checking the size of</param>
    /// <param name="sizeOut">If possible, this value is set to the size of the passed in pointer</param>
    /// <remarks>
    /// Should only be used once the game has finished launching.
    /// </remarks>
    /// <returns>Returns true if it succeeds in determining the size of the memory allocated at the given address</returns>
    bool GetAllocationSize(void* original, nuint* sizeOut);
}
