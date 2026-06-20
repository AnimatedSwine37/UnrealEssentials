using System.Runtime.InteropServices;

namespace UnrealEssentials.Types;

public static class Imports
{
    [DllImport("kernel32", CallingConvention = CallingConvention.Winapi, CharSet = CharSet.Ansi, SetLastError = true, ExactSpelling = true)]
    public static extern nint LoadLibraryA(string libFileName);
    
    [DllImport("kernel32", CallingConvention = CallingConvention.Winapi, CharSet = CharSet.Ansi, SetLastError = true, ExactSpelling = true)]
    public static extern nint GetProcAddress(nint module, string procName);
}