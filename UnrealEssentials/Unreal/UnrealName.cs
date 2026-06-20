using System.Runtime.InteropServices;
using System.Text;
using Reloaded.Hooks.Definitions;

namespace UnrealEssentials.Unreal;
using static Utils;

// From Unreal Toolkit
// See https://github.com/RyoTune/UE.Toolkit/blob/main/UE.Toolkit.Core/Types/Unreal/UE5_4_4/FName.cs

internal static class UnrealName
{
    private static MultiSignature GFNamePoolSignature;
    
    [StructLayout(LayoutKind.Explicit, Size = 0x10)]
    public struct FNamePool
    {
        //[FieldOffset(0x8)] public uint PoolCount;
        //[FieldOffset(0xC)] public uint NameCount;

        public static unsafe void Initialize(IReloadedHooks _hooks, Signatures sigs)
        {
            GFNamePoolSignature = new("GFNamePool", sigs.GFNamePool, address => FName.GFNamePool = (FNamePool*)address);
        }
    }
    
    [StructLayout(LayoutKind.Sequential, Size = 8)]
    public struct FName
    {
        
        /// <summary>
        /// Holds a pointer to the global name pool, allowing for
        /// FNames to be properly viewed as strings.
        /// </summary>
        public static unsafe FNamePool* GFNamePool = null;
        
        internal uint ComparisonIndex;
        internal uint Number;
        
        private static unsafe nint GetPool(uint poolIdx) => *((nint*)(GFNamePool + 1) + poolIdx);

        public unsafe FNameEntry* GetEntry()
        {
            // Get appropriate pool
            var poolIdx = GetPool(ComparisonIndex >> 0x10); // 0xABB2B - pool 0xA
        
            // Go to name entry in pool.
            return (FNameEntry*)((ComparisonIndex & 0xFFFF) * 2 + poolIdx);
        }

        public override string ToString()
        {
            unsafe
            {
                if (GFNamePool != null)
                {
                    return GetEntry()->ToString();
                }
                return $"0x{Number:x}";   
            }
        }

        public bool IsNone() => ComparisonIndex == 0 && Number == 0;
    }
    
    [StructLayout(LayoutKind.Sequential, Size = 0x2)]
    public struct FNameEntryHeader
    {
        // Flags:
        // bIsWide : 1;
        // ProbeHashBits : 5;
        // Len : 10;
        private ushort _data;
    
        // Bit 0: bIsWide (1 bit)
        public bool bIsWide
        {
            get => (_data & 0x0001) != 0;
            set => _data = (ushort)(value ? (_data | 0x0001) : (_data & ~0x0001));
        }

        // Bits 1-5: ProbeHashBits (5 bits)
        public byte ProbeHashBits
        {
            get => (byte)((_data >> 1) & 0x1F); // 0x1F = 5 bits
            set => _data = (ushort)((_data & ~0x003E) | ((value & 0x1F) << 1));
        }

        // Bits 6-15: Len (10 bits)
        public ushort Len
        {
            get => (ushort)((_data >> 6) & 0x03FF); // 0x03FF = 10 bits
            set => _data = (ushort)((_data & ~0xFFC0) | ((value & 0x03FF) << 6));
        }
    }
    
    [StructLayout(LayoutKind.Explicit)]
    public unsafe struct FNameEntry
    {
        private const int NAME_SIZE = 1024;
    
        [FieldOffset(0x0)] private FNameEntryHeader _header;
        [FieldOffset(0x2)] private fixed byte _ansiName[NAME_SIZE];
        [FieldOffset(0x2)] private fixed char _wideName[NAME_SIZE];

        public void SetValue(string newValue)
        {
            const int maxStrLen = NAME_SIZE - 1;
            if (newValue.Length > maxStrLen)
            {
                throw new ArgumentException($"{nameof(SetValue)} || {nameof(newValue)} cannot be longer than {maxStrLen} characters.");
            }

            if (_header.bIsWide)
            {
                fixed (char* str = _wideName)
                {
                    var strBytes = Encoding.Unicode.GetBytes(newValue + '\0');
                    Marshal.Copy(strBytes, 0, (nint)str, strBytes.Length);
                }
            }
            else
            {
                fixed (byte* str = _ansiName)
                {
                    var strBytes = Encoding.Default.GetBytes(newValue + '\0');
                    Marshal.Copy(strBytes, 0, (nint)str, strBytes.Length);
                }
            }
        }

        public Span<char> ToSpanWide()
        {
            fixed (FNameEntry* self = &this)
            {
                return new(self->_wideName, _header.Len);
            }
        }
    
        public Span<byte> ToSpanAnsi()
        {
            fixed (FNameEntry* self = &this)
            {
                return new(self->_ansiName, _header.Len);
            }
        }

        public override string ToString()
        {
            if (_header.bIsWide)
            {
                fixed (char* str = _wideName)
                {
                    return new(str, 0, _header.Len);
                }
            }

            fixed (byte* str = _ansiName)
            {
                return Encoding.Default.GetString(str, _header.Len);
            }
        }
    }
}