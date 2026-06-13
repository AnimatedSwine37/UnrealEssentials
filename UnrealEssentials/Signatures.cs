using Reloaded.Mod.Interfaces;
using riri.yamlscans;
using UTOC.Stream.Emulator.Interfaces;
using YamlDotNet.RepresentationModel;

namespace UnrealEssentials;

public enum StartLoadingDelegateType
{
    NoArgs, // UE 4.25-4.27
    AddIoBatch, // UE 5.0
    PackageNodeArray, // UE 5.1
    AddThreadState, // UE 5.2-5.3
    DescAddInstancingContext, // UE 5.4-5.5
    Type5, // UE 5.6
    AsyncPackageInheritsRefCount, // UE 5.7+
}

public enum ObjectCommandExecutorType
{
    GlobalOnly,
    AddDevEditor,
    AddRuntime
}

public class Properties
{
    // used by UTOC Emulator 1.x
    public TocType? TocVersion { get; set; } = null;
    public PakType PakVersion { get; set; } = PakType.Fn64BugFix;
    // used by UTOC Emulator 2.x
    public EngineVersion EngineVersion { get; set; } = EngineVersion.UE_4_25;
    public StartLoadingDelegateType StartLoadDelegate { get; set; } = StartLoadingDelegateType.NoArgs;
    public bool AllowExecuteCommands { get; set; } = false;
    public ObjectCommandExecutorType CommandExecutorType { get; set; } = ObjectCommandExecutorType.GlobalOnly;
    public Signatures Signatures { get; private set; } = new();
    
    public Properties DeepCopy()
    {
        var result = (Properties)MemberwiseClone();
        result.TocVersion = TocVersion;
        result.PakVersion = PakVersion;
        result.StartLoadDelegate = StartLoadDelegate;
        result.AllowExecuteCommands = AllowExecuteCommands;
        result.CommandExecutorType = CommandExecutorType;
        result.Signatures = Signatures.DeepCopy();
        return result;
    }
}

public class Signatures 
{
    public List<Candidate> GetPakSigningKeys { get; set; } = []; // Function call to FCoreDelegates::GetPakSigningKeysDelegate in FIoStoreTocResource::Read (short jump)
    public List<Candidate> GetPakFolders { get; set; } = []; // FPakPlatformFile::GetPakFolders
    public List<Candidate> GMalloc { get; set; } = []; // during initializing GMalloc. Long Jump
    public List<Candidate> GetPakOrder { get; set; } = []; // FPakPlatformFile::GetPakOrderFromPakFilePath
    public List<Candidate> PakOpenRead { get; set; } = []; // FPakPlatformFile::OpenRead
    public List<Candidate> PakOpenAsyncRead { get; set; } = []; // FPakPlatformFile::OpenAsyncRead
    public List<Candidate> IsNonPakFilenameAllowed { get; set; } = []; // FPakPlatformFile::IsNonPakFilenameAllowed
    public List<Candidate> FileIoStoreOpenContainer { get; set; } = []; // FGenericFileIoStoreImpl::OpenContainer
    public List<Candidate> ReadBlocks { get; set; } = []; // FFileIoStore::ReadBlocks
    public List<Candidate> FileExists { get; set; } = []; // FPakPlatformFile::FileExists
    public List<Candidate> FIOBatch_ReadInternal { get; set; } = [];
    public List<Candidate> FAsyncPackage2_StartLoading { get; set; } = [];
    public List<Candidate> GFNamePool { get; set; } = [];

    public Signatures DeepCopy()
    {
        var result = (Signatures)MemberwiseClone();
        result.GetPakSigningKeys = GetPakSigningKeys.Select(x => new Candidate(x.Signature, x.Transformer)).ToList();
        result.GetPakFolders = GetPakFolders.Select(x => new Candidate(x.Signature, x.Transformer)).ToList();
        result.GMalloc = GMalloc.Select(x => new Candidate(x.Signature, x.Transformer)).ToList();
        result.GetPakOrder = GetPakOrder.Select(x => new Candidate(x.Signature, x.Transformer)).ToList();
        result.PakOpenRead = PakOpenRead.Select(x => new Candidate(x.Signature, x.Transformer)).ToList();
        result.IsNonPakFilenameAllowed = IsNonPakFilenameAllowed.Select(x => new Candidate(x.Signature, x.Transformer)).ToList();
        result.FileIoStoreOpenContainer = FileIoStoreOpenContainer.Select(x => new Candidate(x.Signature, x.Transformer)).ToList();
        result.ReadBlocks = ReadBlocks.Select(x => new Candidate(x.Signature, x.Transformer)).ToList();
        result.FAsyncPackage2_StartLoading = FAsyncPackage2_StartLoading.Select(x => new Candidate(x.Signature, x.Transformer)).ToList();
        result.GFNamePool = GFNamePool.Select(x => new Candidate(x.Signature, x.Transformer)).ToList();
        return result;
    }
}

public class GameRegistry
{
    internal static readonly string[] DistributionTypes = ["Win64", "WinGDK"];

    internal static readonly string DistVersion = "<DistVersion>";

    public Dictionary<string, Properties> ExecutableName { get; } = new();
    public Dictionary<string, Properties> ExecutableNameStartsWith { get; } = new();
    public Dictionary<string, Properties> ProductName { get; } = new();
}

public class SignaturePropertyFactory
{
    public Dictionary<string, Properties> EngineVersions { get; }
    public Dictionary<string, string> FileToBranchName { get; }
    public GameRegistry GameRegistry;
    
    private static string HandleScalar(string Name, YamlNode value)
        => value.Cast<YamlScalarNode>()?.Value ?? throw new Exception($"Value for {Name} must be a string");

    private static PakType HandlePakVersion(YamlNode value)
    {
        var str = value.Cast<YamlScalarNode>()?.Value ?? throw new Exception("Value for PakVersion must be a string");
        return Enum.TryParse<PakType>(str, out var Value) ? Value : throw new Exception($"Value \"{str}\" is not in PakType");
    }
    
    private static TocType HandleTocVersion(YamlNode value)
    {
        var str = value.Cast<YamlScalarNode>()?.Value ?? throw new Exception("Value for TocVersion must be a string");
        return Enum.TryParse<TocType>(str, out var Value) ? Value : throw new Exception($"Value \"{str}\" is not in TocType");
    }
    
    private static StartLoadingDelegateType HandleStartLoadDelegate(YamlNode value)
    {
        var str = value.Cast<YamlScalarNode>()?.Value ?? throw new Exception("Value for StartLoadDelegate must be a string");
        return Enum.TryParse<StartLoadingDelegateType>(str, out var Value) ? Value : throw new Exception($"Value \"{str}\" is not in StartLoadingDelegateType");
    }
    
    private static bool HandleAllowExecuteCommands(YamlNode value)
    {
        var str = value.Cast<YamlScalarNode>()?.Value ?? throw new Exception("Value for AllowExecuteCommands must be a string");
        return str.ToLower() switch
        {
            "true" => true,
            "false" => false,
            _ => throw new Exception($"Value \"{str}\" is not true or false")
        };
    }
    
    private static ObjectCommandExecutorType HandleCommandExecutorType(YamlNode value)
    {
        var str = value.Cast<YamlScalarNode>()?.Value ?? throw new Exception("Value for CommandExecutorType must be a string");
        return Enum.TryParse<ObjectCommandExecutorType>(str, out var Value) ? Value : throw new Exception($"Value \"{str}\" is not in CommandExecutorType");
    }

    private static void TryGetSignature(string key, Dictionary<string, List<Candidate>> signatures, Action<List<Candidate>> callback)
    {
        if (signatures.TryGetValue(key, out var source))
            // callback = source;
            callback(source);
    }

    private static void SetSignatures(Properties properties, YamlNode value)
    {
        // Signatures is scalar if it has no children
        if (value.NodeType == YamlNodeType.Scalar) return;
        var sigSeq = value.Cast<YamlMappingNode>() ?? throw new Exception("Expected a sequence for signatures");
        var model = ScanModel.FromNode(sigSeq).ToDictionary();
        TryGetSignature("GetPakSigningKeys", model, x => properties.Signatures.GetPakSigningKeys = x);
        TryGetSignature("GetPakFolders", model, x => properties.Signatures.GetPakFolders = x);
        TryGetSignature("GMalloc", model, x => properties.Signatures.GMalloc = x);
        TryGetSignature("GetPakOrder", model, x => properties.Signatures.GetPakOrder = x);
        TryGetSignature("PakOpenRead", model, x => properties.Signatures.PakOpenRead = x);
        TryGetSignature("PakOpenAsyncRead", model, x => properties.Signatures.PakOpenAsyncRead = x);
        TryGetSignature("IsNonPakFilenameAllowed", model, x => properties.Signatures.IsNonPakFilenameAllowed = x);
        TryGetSignature("FileIoStoreOpenContainer", model, x => properties.Signatures.FileIoStoreOpenContainer = x);
        TryGetSignature("ReadBlocks", model, x => properties.Signatures.ReadBlocks = x);
        TryGetSignature("FileExists", model, x => properties.Signatures.FileExists = x);
        TryGetSignature("FIOBatch_ReadInternal", model, x => properties.Signatures.FIOBatch_ReadInternal = x);
        TryGetSignature("FAsyncPackage2_StartLoading", model, x => properties.Signatures.FAsyncPackage2_StartLoading = x);
        TryGetSignature("GFNamePool", model, x => properties.Signatures.GFNamePool = x);
        
    }
    
    // For unit testing
    public static (string, Properties) ParseEngineYamlStatic(string filePath)
    {
        var Value = File.ReadAllText(filePath);
        var reader = new YamlStream();
        reader.Load(new StringReader(Value));
        var root = reader.Documents[0].RootNode.Cast<YamlMappingNode>() 
                   ?? throw new Exception("Expected a mapping at the top-level");
        var properties = new Properties();
        var filename = Path.GetFileNameWithoutExtension(filePath);
        properties.EngineVersion = Enum.TryParse<EngineVersion>(filename, out var engineVersion)
            ? engineVersion : throw new Exception($"Unrecognised engine version {filename}, must be defined in EngineVersion enum!");
        var branchName = string.Empty;
        foreach (var child in root.Children)
        {
            var key = child.Key.Cast<YamlScalarNode>()?.Value ?? throw new Exception("Expected a string for the key");
            switch (key)
            {
                case "VersionIdentifier":
                    branchName = HandleScalar("VersionIdentifier", child.Value);
                    break;
                case "PakVersion":
                    properties.PakVersion = HandlePakVersion(child.Value);
                    break;
                case "TocVersion":
                    properties.TocVersion = HandleTocVersion(child.Value);
                    break;
                case "StartLoadDelegate":
                    properties.StartLoadDelegate = HandleStartLoadDelegate(child.Value);
                    break;
                case "AllowExecuteCommands":
                    properties.AllowExecuteCommands = HandleAllowExecuteCommands(child.Value);
                    break;
                case "CommandExecutorType":
                    properties.CommandExecutorType = HandleCommandExecutorType(child.Value);
                    break;
                case "Signatures":
                    SetSignatures(properties, child.Value);
                    break;
                default:
                    throw new Exception($"Unrecognised property {key}");
            }
        }
        return (branchName, properties);
    }
    
    private void ParseEngineYaml(string filePath)
    {
        var (branchName, properties) = ParseEngineYamlStatic(filePath);
        var fileName = Path.GetFileNameWithoutExtension(filePath);
        EngineVersions.Add(branchName, properties);
        FileToBranchName.Add(fileName, branchName);
    }

    private Properties? HandleEngineVersion(YamlNode value)
    {
        var str = value.Cast<YamlScalarNode>()?.Value ?? throw new Exception("Value for EngineVersion must be a string");
        return FileToBranchName.TryGetValue(str, out var BranchName) && 
               EngineVersions.TryGetValue(BranchName, out var props)
            ? props.DeepCopy() : null;
    }

    private void ParseGameYaml(string filePath)
    {
        var Value = File.ReadAllText(filePath);
        var reader = new YamlStream();
        reader.Load(new StringReader(Value));
        var root = reader.Documents[0].RootNode.Cast<YamlMappingNode>() 
                   ?? throw new Exception("Expected a mapping at the top-level");
        Properties? properties = null;
        string? ExecutableName = null;
        string? ExecutableNameStartsWith = null;
        string? ProductName = null;
        foreach (var child in root.Children)
        {
            var key = child.Key.Cast<YamlScalarNode>()?.Value ?? throw new Exception("Expected a string for the key");
            switch (key)
            {
                case "EngineVersion":
                    properties ??= HandleEngineVersion(child.Value);
                    break;
                case "ExecutableName":
                    ExecutableName ??= HandleScalar("ExecutableName", child.Value);
                    break;
                case "ExecutableNameStartsWith":
                    ExecutableNameStartsWith ??= HandleScalar("ExecutableNameStartsWith", child.Value);
                    break;
                case "ProductName":
                    ProductName ??= HandleScalar("ProductName", child.Value);
                    break;
                case "Signatures":
                    if (properties == null)
                        throw new Exception("EngineVersion must be declared before defining signature overrides!");
                    SetSignatures(properties, child.Value);
                    break;
                default:
                    throw new Exception($"Unrecognised property {key}");
            }
        }
        if (ExecutableName != null)
        {
            if (ExecutableName.Contains(GameRegistry.DistVersion))
            {
                foreach (var variant in GameRegistry.DistributionTypes.Select(x =>
                             ExecutableName.Replace(GameRegistry.DistVersion, x)))
                    GameRegistry.ExecutableName.Add(variant, properties!);
            }
            else
            {
                GameRegistry.ExecutableName.Add(ExecutableName, properties!);
            }
        }
        if (ExecutableNameStartsWith != null)
            GameRegistry.ExecutableNameStartsWith.Add(ExecutableNameStartsWith, properties!);
        if (ProductName != null)
            GameRegistry.ProductName.Add(ProductName, properties!);
    }

    public SignaturePropertyFactory(string sigDir)
    {
        EngineVersions = new();
        FileToBranchName = new();
        GameRegistry = new();
        foreach (var engineYaml in Directory.GetFiles(Path.Combine(sigDir, "Engine"), "*.yaml",
                     SearchOption.TopDirectoryOnly))
            ParseEngineYaml(engineYaml);
        foreach (var gameYaml in Directory.GetFiles(Path.Combine(sigDir, "Game"), "*.yaml",
                     SearchOption.TopDirectoryOnly))
            ParseGameYaml(gameYaml);
    }
}