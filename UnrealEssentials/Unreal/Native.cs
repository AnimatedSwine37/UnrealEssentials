using System.Runtime.InteropServices;
using static UnrealEssentials.Unreal.UnrealArray;
using static UnrealEssentials.Unreal.UnrealString;

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

        /// <summary>
        /// Setup empty signing keys
        /// </summary>
        /// <returns>An unmanaged pointer to a blank FPakSigningKeys instance</returns>
        internal static unsafe FPakSigningKeys* NewBlank()
        {
            var signingKeys = (FPakSigningKeys*)NativeMemory.Alloc((nuint)sizeof(FPakSigningKeys));
            signingKeys->Function = 0;
            signingKeys->Size = 0;
            return signingKeys;
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct FIoChunkId
    {
        private fixed byte Data[0xc];

        public override string ToString()
        {
            string key_out = "0x";
            
            for (int i = 0; i < 0xc; i++) key_out += $"{Data[i]:X2}";
            return key_out;
        }

        public FIoChunkId(ulong ChunkId, short ChunkIndex, byte IoChunkType)
        {
            fixed (FIoChunkId* self = &this)
            {
                *(ulong*)self = ChunkId;
                *(short*)((nint)self + 8) = ChunkIndex;
                *(short*)((nint)self + 11) = IoChunkType;
            }
        }
    }
    
    [StructLayout(LayoutKind.Explicit)]
    internal struct FAsyncPackage2
    {
        [FieldOffset(0x18)] internal ulong DiskPackageId;
        [FieldOffset(0x28)] internal UnrealName.FName DiskPackageName;
    }
    
    [StructLayout(LayoutKind.Explicit)]
    internal struct FAsyncPackage2_UE5_0
    {
        // offset: Self->Desc.PackageIdToLoad + 0x8
        [FieldOffset(0xb0)] internal UnrealName.FName PackagePathToLoad;
    }
    
    [StructLayout(LayoutKind.Explicit)]
    internal struct FAsyncPackage2_UE5_1
    {
        // offset: Self->Desc.PackageIdToLoad + 0x10
        [FieldOffset(0xe8)] internal UnrealName.FName PackagePathToLoad;
    }
    
    [StructLayout(LayoutKind.Explicit)]
    internal struct FAsyncPackage2_UE5_3
    {
        // offset: Self->Desc.PackageIdToLoad + 0x10
        [FieldOffset(0x100)] internal UnrealName.FName PackagePathToLoad;
    }
    
    [StructLayout(LayoutKind.Explicit)]
    internal struct FAsyncPackage2_UE5_4
    {
        // offset: Self->Desc.PackageIdToLoad + 0x10
        [FieldOffset(0x110)] internal UnrealName.FName PackagePathToLoad;
    }
    
    [StructLayout(LayoutKind.Explicit)]
    internal struct FAsyncPackage2_UE5_6
    {
        // offset: Self->Desc.PackageIdToLoad + 0x10
        [FieldOffset(0x1d0)] internal UnrealName.FName PackagePathToLoad;
    }
    
    [StructLayout(LayoutKind.Explicit)]
    internal struct FAsyncPackage2_UE5_7
    {
        // offset: Self->Desc.PackageIdToLoad + 0x10
        [FieldOffset(0x218)] internal UnrealName.FName PackagePathToLoad;
    }
}
