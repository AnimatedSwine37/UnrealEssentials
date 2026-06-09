using System.Reflection;
using riri.yamlscans;
using UTOC.Stream.Emulator.Interfaces;

namespace UnrealEssentials.Tests;

[TestClass]
public sealed class SignatureTests 
{
    private string SignatureFolder = Path.GetFullPath(Path.Combine(Assembly.GetExecutingAssembly().Location, "../../../../../UnrealEssentials/Signatures"));
    
    [TestMethod]
    public void TestEngineSignatures()
    {
            var engineSigs = Path.Combine(SignatureFolder, "Engine");
            var (ver_4_18, prop_4_18) = SignaturePropertyFactory.ParseEngineYamlStatic(Path.Combine(engineSigs, "UE_4_18.yaml"));
            Assert.AreEqual("++UE4+Release-4.18", ver_4_18);
            Assert.AreEqual("48 89 6C 24 ?? 48 89 74 24 ?? 57 48 83 EC 40 41 0F B6 E8 48 C7 44 24 ?? 00 00 00 00", prop_4_18.Signatures.PakOpenRead[0].Signature);
            Assert.AreEqual(new GetDirectAddress(), prop_4_18.Signatures.PakOpenRead[0].Transformer);
            
            var (ver_4_19, prop_4_19) = SignaturePropertyFactory.ParseEngineYamlStatic(Path.Combine(engineSigs, "UE_4_19.yaml"));
            Assert.AreEqual("++UE4+Release-4.19", ver_4_19);
            Assert.AreEqual("48 89 6C 24 ?? 48 89 74 24 ?? 57 48 83 EC 40 41 0F B6 E8", prop_4_19.Signatures.PakOpenRead[0].Signature);
            Assert.AreEqual(new GetDirectAddress(), prop_4_19.Signatures.PakOpenRead[0].Transformer);
            
            var (ver_4_20, prop_4_20) = SignaturePropertyFactory.ParseEngineYamlStatic(Path.Combine(engineSigs, "UE_4_20.yaml"));
            Assert.AreEqual("++UE4+Release-4.20", ver_4_20);
            
            var (ver_4_21, prop_4_21) = SignaturePropertyFactory.ParseEngineYamlStatic(Path.Combine(engineSigs, "UE_4_21.yaml"));
            Assert.AreEqual("++UE4+Release-4.21", ver_4_21);
            Assert.AreEqual("48 89 5C 24 ?? 55 56 57 41 54 41 55 41 56 41 57 48 8D 6C 24 ?? 48 81 EC B0 00 00 00 48 8B 05 ?? ?? ?? ?? 48 33 C4 48 89 45 ?? 66 0F 6F 05 ?? ?? ?? ?? 48 8D 59 ??", prop_4_21.Signatures.PakOpenRead[0].Signature);
            Assert.AreEqual(new GetDirectAddress(), prop_4_21.Signatures.PakOpenRead[0].Transformer);
            
            var (ver_4_22, prop_4_22) = SignaturePropertyFactory.ParseEngineYamlStatic(Path.Combine(engineSigs, "UE_4_22.yaml"));
            Assert.AreEqual("++UE4+Release-4.22", ver_4_22);
            
            var (ver_4_23, prop_4_23) = SignaturePropertyFactory.ParseEngineYamlStatic(Path.Combine(engineSigs, "UE_4_23.yaml"));
            Assert.AreEqual("++UE4+Release-4.23", ver_4_23);
            
            var (ver_4_24, prop_4_24) = SignaturePropertyFactory.ParseEngineYamlStatic(Path.Combine(engineSigs, "UE_4_24.yaml"));
            Assert.AreEqual("++UE4+Release-4.24", ver_4_24);
            
            var (ver_4_25, prop_4_25) = SignaturePropertyFactory.ParseEngineYamlStatic(Path.Combine(engineSigs, "UE_4_25.yaml"));
            Assert.AreEqual("++UE4+Release-4.25", ver_4_25);
            Assert.AreEqual("E8 ?? ?? ?? ?? 48 8B D8 39 78 ??", prop_4_25.Signatures.GetPakSigningKeys[0].Signature);
            Assert.AreEqual(new GetIndirectAddressShort(), prop_4_25.Signatures.GetPakSigningKeys[0].Transformer);
            Assert.AreEqual("48 89 5C 24 ?? 48 89 74 24 ?? 48 89 7C 24 ?? 4C 89 74 24 ?? 55 48 8B EC 48 83 EC 40 48 8D 4D ??", prop_4_25.Signatures.GetPakFolders[0].Signature);
            Assert.AreEqual(new GetDirectAddress(), prop_4_25.Signatures.GetPakFolders[0].Transformer);
            Assert.AreEqual("48 89 05 ?? ?? ?? ?? E8 ?? ?? ?? ?? E8 ?? ?? ?? ?? 84 C0 74 ??", prop_4_25.Signatures.GMalloc[0].Signature);
            Assert.AreEqual(new GetIndirectAddressLong(), prop_4_25.Signatures.GMalloc[0].Transformer);
            Assert.AreEqual("48 89 5C 24 ?? 57 48 83 EC 40 48 8B D9 48 8D 4C 24 ?? E8 ?? ?? ?? ?? 83 78 08 00", prop_4_25.Signatures.GetPakOrder[0].Signature);
            Assert.AreEqual(new GetDirectAddress(), prop_4_25.Signatures.GetPakOrder[0].Transformer);
            Assert.AreEqual("48 89 5C 24 ?? 55 56 57 41 54 41 55 41 56 41 57 48 8D 6C 24 ?? 48 81 EC D0 00 00 00 48 8B 05 ?? ?? ?? ?? 48 33 C4 48 89 45 ?? 66 0F 6F 05 ?? ?? ?? ??", prop_4_25.Signatures.PakOpenRead[0].Signature);
            Assert.AreEqual(new GetDirectAddress(), prop_4_25.Signatures.PakOpenRead[0].Transformer);
            Assert.AreEqual("40 55 57 41 56 41 57 48 81 EC 98 00 00 00", prop_4_25.Signatures.PakOpenAsyncRead[0].Signature);
            Assert.AreEqual(new GetDirectAddress(), prop_4_25.Signatures.PakOpenAsyncRead[0].Transformer);
            Assert.AreEqual("48 8B C4 55 41 55 48 8D 68 ?? 48 81 EC 98 00 00 00", prop_4_25.Signatures.IsNonPakFilenameAllowed[0].Signature);
            Assert.AreEqual(new GetDirectAddress(), prop_4_25.Signatures.IsNonPakFilenameAllowed[0].Transformer);
            Assert.AreEqual("48 89 6C 24 ?? 57 48 83 EC 30 45 33 C9 45 33 C0 48 8B FA 48 8B E9 E8 ?? ?? ?? ?? 84 C0 74 ?? B0 01 48 8B 6C 24 ?? 48 83 C4 30 5F C3 33 C9 48 89 5C 24 ?? 48 89 74 24 ?? 8B D1 40 32 F6 48 89 4C 24 ?? 48 89 4C 24 ?? 48 85 FF 74 ?? 66 39 0F 74 ?? 48 C7 C3 FF FF FF FF 0F 1F 84 ?? 00 00 00 00 48 FF C3 66 39 0C ?? 75 ?? FF C3 85 DB 7E ?? 8B D3 48 8D 4C 24 ?? E8 ?? ?? ?? ?? 8B 54 24 ?? 8B 4C 24 ?? 8D 04 ?? 89 44 24 ?? 3B C2 7E ?? 8B D1 48 8D 4C 24 ?? E8 ?? ?? ?? ?? 48 8B 4C 24 ?? 48 8B D7 4C 63 C3 4D 03 C0 E8 ?? ?? ?? ?? 48 8D 54 24 ?? 48 8B CD E8 ?? ?? ?? ?? 48 8B 4C 24 ?? 0F B6 D8 48 85 C9 74 ?? E8 ?? ?? ?? ?? 84 DB 48 8B 5C 24 ?? 74 ?? 48 8B 4D ?? 48 8B D7 48 8B 01 FF 50 ??", prop_4_25.Signatures.FileExists[0].Signature);
            Assert.AreEqual(new GetDirectAddress(), prop_4_25.Signatures.FileExists[0].Transformer);
    }

    [TestMethod]
    public void TestGameSignatures()
    {
        var factory = new SignaturePropertyFactory(SignatureFolder);
        // Persona 3 Reload
        Assert.IsTrue(factory.GameRegistry.ExecutableName.TryGetValue("P3R", out var P3RProps));
        Assert.AreEqual(PakType.Fn64BugFix, P3RProps.PakVersion);
        Assert.AreEqual(StartLoadingDelegateType.NoArgs, P3RProps.StartLoadDelegate);
        Assert.AreEqual("E8 ?? ?? ?? ?? 48 8B F8 39 70 ??", P3RProps.Signatures.GetPakSigningKeys[0].Signature);
        Assert.AreEqual(new GetIndirectAddressShort(), P3RProps.Signatures.GetPakSigningKeys[0].Transformer);
        Assert.AreEqual("48 89 5C 24 ?? 48 89 74 24 ?? 48 89 7C 24 ?? 4C 89 74 24 ?? 55 48 8B EC 48 83 EC 40 48 8D 4D ??", P3RProps.Signatures.GetPakFolders[0].Signature);
        Assert.AreEqual(new GetDirectAddress(), P3RProps.Signatures.GetPakFolders[0].Transformer);
        Assert.AreEqual("48 8B 0D ?? ?? ?? ?? 48 8B 01 FF 50 ?? 33 F6", P3RProps.Signatures.GMalloc[0].Signature);
        Assert.AreEqual(new GetIndirectAddressLong(), P3RProps.Signatures.GMalloc[0].Transformer);
        Assert.AreEqual("48 89 5C 24 ?? 57 48 83 EC 40 48 8B D9 48 8D 4C 24 ?? E8 ?? ?? ?? ?? 83 78 ?? 00", P3RProps.Signatures.GetPakOrder[0].Signature);
        Assert.AreEqual(new GetDirectAddress(), P3RProps.Signatures.GetPakOrder[0].Transformer);
        Assert.AreEqual("4C 8B DC 55 53 57 41 54 49 8D 6B ?? 48 81 EC B8 00 00 00 48 8B 05 ?? ?? ?? ?? 48 33 C4 48 89 45 ?? 66 0F 6F 05 ?? ?? ?? ??", P3RProps.Signatures.PakOpenRead[0].Signature);
        Assert.AreEqual(new GetDirectAddress(), P3RProps.Signatures.PakOpenRead[0].Transformer);
        Assert.AreEqual("40 53 55 56 41 56 41 57 48 81 EC 90 00 00 00", P3RProps.Signatures.PakOpenAsyncRead[0].Signature);
        Assert.AreEqual(new GetDirectAddress(), P3RProps.Signatures.PakOpenAsyncRead[0].Transformer);
        Assert.AreEqual("48 89 5C 24 ?? 48 89 6C 24 ?? 56 57 41 56 48 83 EC 30 48 8B F1 45 33 C0", P3RProps.Signatures.IsNonPakFilenameAllowed[0].Signature);
        Assert.AreEqual(new GetDirectAddress(), P3RProps.Signatures.IsNonPakFilenameAllowed[0].Transformer);
        Assert.AreEqual("48 89 5C 24 ?? 48 89 6C 24 ?? 48 89 74 24 ?? 48 89 7C 24 ?? 41 56 48 83 EC 20 49 8B F1 4D 8B F0", P3RProps.Signatures.FileIoStoreOpenContainer[0].Signature);
        Assert.AreEqual(new GetDirectAddress(), P3RProps.Signatures.FileIoStoreOpenContainer[0].Transformer);
        Assert.AreEqual("48 89 6C 24 ?? 57 48 83 EC 30 45 33 C9 45 33 C0 48 8B FA 48 8B E9 E8 ?? ?? ?? ?? 84 C0 74 ?? B0 01 48 8B 6C 24 ?? 48 83 C4 30 5F C3 33 C9 48 89 5C 24 ?? 48 89 74 24 ?? 8B D1 40 32 F6 48 89 4C 24 ?? 48 89 4C 24 ?? 48 85 FF 74 ?? 66 39 0F 74 ?? 48 C7 C3 FF FF FF FF 0F 1F 84 ?? 00 00 00 00 48 FF C3 66 39 0C ?? 75 ?? FF C3 85 DB 7E ?? 8B D3 48 8D 4C 24 ?? E8 ?? ?? ?? ?? 8B 54 24 ?? 8B 4C 24 ?? 8D 04 ?? 89 44 24 ?? 3B C2 7E ?? 8B D1 48 8D 4C 24 ?? E8 ?? ?? ?? ?? 48 8B 4C 24 ?? 48 8B D7 4C 63 C3 4D 03 C0 E8 ?? ?? ?? ?? 48 8D 54 24 ?? 48 8B CD E8 ?? ?? ?? ?? 48 8B 4C 24 ?? 0F B6 D8 48 85 C9 74 ?? E8 ?? ?? ?? ?? 84 DB 48 8B 5C 24 ?? 74 ?? 48 8B 4D ?? 48 8B D7 48 8B 01 FF 50 ??", P3RProps.Signatures.FileExists[0].Signature);
        Assert.AreEqual(new GetDirectAddress(), P3RProps.Signatures.FileExists[0].Transformer);
        Assert.AreEqual("48 89 5C 24 ?? 57 48 83 EC 40 41 0F 10 00 48 8B F9", P3RProps.Signatures.FIOBatch_ReadInternal[0].Signature);
        Assert.AreEqual(new GetDirectAddress(), P3RProps.Signatures.FIOBatch_ReadInternal[0].Transformer);
    }
}