use std::path::{Path, PathBuf};
use std::process::Command;

const SHORTCUT_NAME: &str = "HinaView.lnk";
const APP_USER_MODEL_ID: &str = "LazyGamerStudio.HinaView";

pub fn register_shortcut() -> Result<(), String> {
    let exe_path = current_exe_path()?;
    let app_dir = exe_path
        .parent()
        .ok_or_else(|| "Failed to resolve application directory".to_string())?;
    let shortcut_path = shortcut_path()?;

    if let Some(parent) = shortcut_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let script = format!(
        r#"
$shortcutPath = '{shortcut}'
$targetPath = '{target}'
$workingDirectory = '{workdir}'
$iconPath = '{icon}'
$appId = '{app_id}'

$source = @"
using System;
using System.Runtime.InteropServices;

[ComImport]
[Guid("00021401-0000-0000-C000-000000000046")]
class ShellLink {{ }}

[ComImport]
[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
[Guid("000214F9-0000-0000-C000-000000000046")]
interface IShellLinkW
{{
    void GetPath([Out, MarshalAs(UnmanagedType.LPWStr)] System.Text.StringBuilder pszFile, int cchMaxPath, IntPtr pfd, int fFlags);
    void GetIDList(out IntPtr ppidl);
    void SetIDList(IntPtr pidl);
    void GetDescription([Out, MarshalAs(UnmanagedType.LPWStr)] System.Text.StringBuilder pszName, int cchMaxName);
    void SetDescription([MarshalAs(UnmanagedType.LPWStr)] string pszName);
    void GetWorkingDirectory([Out, MarshalAs(UnmanagedType.LPWStr)] System.Text.StringBuilder pszDir, int cchMaxPath);
    void SetWorkingDirectory([MarshalAs(UnmanagedType.LPWStr)] string pszDir);
    void GetArguments([Out, MarshalAs(UnmanagedType.LPWStr)] System.Text.StringBuilder pszArgs, int cchMaxPath);
    void SetArguments([MarshalAs(UnmanagedType.LPWStr)] string pszArgs);
    void GetHotkey(out short pwHotkey);
    void SetHotkey(short wHotkey);
    void GetShowCmd(out int piShowCmd);
    void SetShowCmd(int iShowCmd);
    void GetIconLocation([Out, MarshalAs(UnmanagedType.LPWStr)] System.Text.StringBuilder pszIconPath, int cchIconPath, out int piIcon);
    void SetIconLocation([MarshalAs(UnmanagedType.LPWStr)] string pszIconPath, int iIcon);
    void SetRelativePath([MarshalAs(UnmanagedType.LPWStr)] string pszPathRel, int dwReserved);
    void Resolve(IntPtr hwnd, int fFlags);
    void SetPath([MarshalAs(UnmanagedType.LPWStr)] string pszFile);
}}

[ComImport]
[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
[Guid("886D8EEB-8CF2-4446-8D02-CDBA1DBDCF99")]
interface IPropertyStore
{{
    void GetCount(out int cProps);
    void GetAt(int iProp, out PROPERTYKEY pkey);
    void GetValue(ref PROPERTYKEY key, out PROPVARIANT pv);
    void SetValue(ref PROPERTYKEY key, ref PROPVARIANT propvar);
    void Commit();
}}

[ComImport]
[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
[Guid("0000010b-0000-0000-C000-000000000046")]
interface IPersistFile
{{
    void GetClassID(out Guid pClassID);
    void IsDirty();
    void Load([MarshalAs(UnmanagedType.LPWStr)] string pszFileName, uint dwMode);
    void Save([MarshalAs(UnmanagedType.LPWStr)] string pszFileName, bool fRemember);
    void SaveCompleted([MarshalAs(UnmanagedType.LPWStr)] string pszFileName);
    void GetCurFile([MarshalAs(UnmanagedType.LPWStr)] out string ppszFileName);
}}

[StructLayout(LayoutKind.Sequential, Pack = 4)]
struct PROPERTYKEY
{{
    public Guid fmtid;
    public uint pid;

    public PROPERTYKEY(Guid format, uint propertyId)
    {{
        fmtid = format;
        pid = propertyId;
    }}
}}

[StructLayout(LayoutKind.Explicit)]
struct PROPVARIANT
{{
    [FieldOffset(0)]
    public ushort vt;
    [FieldOffset(8)]
    public IntPtr pointerValue;

    public static PROPVARIANT FromString(string value)
    {{
        var prop = new PROPVARIANT();
        prop.vt = 31;
        prop.pointerValue = Marshal.StringToCoTaskMemUni(value);
        return prop;
    }}
}}

static class NativeMethods
{{
    [DllImport("ole32.dll")]
    public static extern int PropVariantClear(ref PROPVARIANT pvar);
}}

public static class ShortcutUtil
{{
    static readonly PROPERTYKEY PKEY_AppUserModel_ID =
        new PROPERTYKEY(new Guid("9F4C2855-9F79-4B39-A8D0-E1D42DE1D5F3"), 5);

    public static void CreateShortcut(
        string shortcutPath,
        string targetPath,
        string workingDirectory,
        string iconPath,
        string appId)
    {{
        var shellLink = (IShellLinkW)new ShellLink();
        shellLink.SetPath(targetPath);
        shellLink.SetWorkingDirectory(workingDirectory);
        shellLink.SetDescription("HinaView");
        shellLink.SetIconLocation(iconPath, 0);

        var propertyStore = (IPropertyStore)shellLink;
        SetStringProperty(propertyStore, PKEY_AppUserModel_ID, appId);
        propertyStore.Commit();
        ((IPersistFile)shellLink).Save(shortcutPath, true);
    }}

    static void SetStringProperty(IPropertyStore propertyStore, PROPERTYKEY key, string value)
    {{
        var prop = PROPVARIANT.FromString(value);
        try
        {{
            propertyStore.SetValue(ref key, ref prop);
        }}
        finally
        {{
            NativeMethods.PropVariantClear(ref prop);
        }}
    }}
}}
"@

Add-Type -TypeDefinition $source -Language CSharp
[ShortcutUtil]::CreateShortcut($shortcutPath, $targetPath, $workingDirectory, $iconPath, $appId)
"#,
        shortcut = ps_quote(shortcut_path.as_path()),
        target = ps_quote(exe_path.as_path()),
        workdir = ps_quote(app_dir),
        icon = ps_quote(exe_path.as_path()),
        app_id = APP_USER_MODEL_ID,
    );

    run_powershell(&script)
}

pub fn unregister_shortcut() -> Result<(), String> {
    let shortcut_path = shortcut_path()?;
    if !shortcut_path.exists() {
        return Ok(());
    }

    std::fs::remove_file(&shortcut_path).map_err(|e| e.to_string())
}

fn shortcut_path() -> Result<PathBuf, String> {
    let appdata = std::env::var_os("APPDATA")
        .ok_or_else(|| "APPDATA environment variable is missing".to_string())?;
    Ok(PathBuf::from(appdata)
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join(SHORTCUT_NAME))
}

fn current_exe_path() -> Result<PathBuf, String> {
    std::env::current_exe().map_err(|e| e.to_string())
}

fn run_powershell(script: &str) -> Result<(), String> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let message = if !stderr.is_empty() { stderr } else { stdout };
        Err(if message.is_empty() {
            "PowerShell command failed".to_string()
        } else {
            message
        })
    }
}

fn ps_quote(path: &Path) -> String {
    path.display().to_string().replace('\'', "''")
}
