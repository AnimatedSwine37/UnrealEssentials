using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using FileEmulationFramework.Lib.Utilities;

namespace UTOC.Stream.Emulator;

public static class LogAdapter
{
    internal static Logger? LoggerInstance;

    private delegate void PrintReloaded(string message);

    private static PrintReloaded GetPrintDelegate(LogSeverity level) => level switch
    {
        LogSeverity.Debug => LoggerInstance!.Debug,
        LogSeverity.Information => LoggerInstance!.Info,
        LogSeverity.Warning => LoggerInstance!.Warning,
        LogSeverity.Error => LoggerInstance!.Error,
        LogSeverity.Fatal => LoggerInstance!.Fatal,
        _ => LoggerInstance!.Info
    };
    
    [UnmanagedCallersOnly(CallConvs = [ typeof(CallConvStdcall) ])]
    public static void ReloadedLoggerWrite(nint p, nint len, int level)
    {
        GetPrintDelegate((LogSeverity)level)($"[UtocEmulator] {Marshal.PtrToStringUTF8(p, (int)len)}");
    }
    
    public static void RegisterLogger(Logger _loggerInstance)
    {
        LoggerInstance = _loggerInstance;
        unsafe
        {
            RustApi.SetReloadedLogger(&ReloadedLoggerWrite);
            RustApiNew.set_reloaded_logger(&ReloadedLoggerWrite);
        }
    }
}