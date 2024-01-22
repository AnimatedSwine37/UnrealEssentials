using static UnrealEssentials.Unreal.UnrealMemory;
using static UnrealEssentials.Unreal.UnrealArray;
using System.Runtime.InteropServices;

namespace UnrealEssentials.Unreal;
internal unsafe class UnrealString
{
    internal struct FString
    {
        TArray<char> Data; // characters are either ANSICHAR or WIDECHAR depending on platform. See definition in Core\Public\HAL\Platform.h

        internal FString(string str)
        {
            if (!str.EndsWith('\0'))
                str += '\0';

            Data = new TArray<char>();
            Data.Capacity = str.Length;
            Data.Length = Data.Capacity;

            char* chars = (char*)Malloc((nuint)Data.Length * sizeof(char));
            for (int i = 0; i < str.Length; i++)
                chars[i] = str[i];
            Data.Values = chars;
        }

        public override string ToString()
        {
            return Marshal.PtrToStringUni((nint)Data.Values, Data.Length);
        }
    }
}
