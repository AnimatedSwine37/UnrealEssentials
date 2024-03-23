namespace UnrealEssentials.Interfaces;
public interface IUnrealEssentials
{
    /// <summary>
    /// Adds files from the folder at <paramref name="path"/>. 
    /// This folder is treated like it was the UnrealEssentials folder inside of a mod.
    /// </summary>
    /// <param name="path">Path to the folder that contains files to be loaded.</param>
    public void AddFromFolder(string path);
}
