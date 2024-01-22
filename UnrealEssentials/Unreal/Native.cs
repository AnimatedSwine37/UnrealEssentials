using static UnrealEssentials.Unreal.UnrealMemory;

namespace UnrealEssentials.Unreal;
internal unsafe class Native
{
    /// <summary>
    /// This isn't neccessarily accurate to Unreal Engine source, 
    /// it's just good enough for removing signatures
    /// </summary>
    internal struct FPakSigningKeys
    {
        internal nuint Function;
        internal int Size;
    }

    internal struct TArray<T>
    {
        internal T* Values;
        internal int Length;
        internal int Capacity;

        internal void Add(T value)
        {
            if (Length + 1 <= Capacity)
            {
                Values[Length++] = value;
                return;
            }

            Resize(Capacity + 1);
            Values[Length++] = value;
        }

        internal void Resize(int newCapcity)
        {
            Values = (T*)Realloc(Values, (nuint)(sizeof(T) * newCapcity));
            Capacity = newCapcity;
        }
    }

    internal struct FString
    {
        TArray<char> Data; // characters are either ANSICHAR or WIDECHAR depending on platform. See definition in Core\Public\HAL\Platform.h

        internal FString(string str)
        {
            if(!str.EndsWith('\0'))
                str += '\0';

            Data = new TArray<char>();
            Data.Capacity = str.Length;
            Data.Length = Data.Capacity;

            char* chars = (char*)Malloc((nuint)Data.Length * sizeof(char));
            for (int i = 0; i < str.Length; i++)
                chars[i] = str[i];
            Data.Values = chars;
        }
    }

    internal delegate FPakSigningKeys* GetPakSigningKeysDelegate();
    internal delegate void GetPakFoldersDelegate(nuint cmdLine, TArray<FString>* outPakFolders);
}
