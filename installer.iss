; CANVIEW 安装程序脚本
; 使用 Inno Setup 编译此脚本以创建安装程序

#define MyAppName "CANVIEW"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "Your Company"
#define MyAppURL "https://github.com/yourusername/canview"
#define MyAppExeName "canview.exe"

[Setup]
; 基本信息
AppId={{A1B2C3D4-E5F6-4A5B-8C9D-0E1F2A3B4C5D}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
LicenseFile=LICENSE.txt
OutputDir=installer-output
OutputBaseFilename=CANVIEW-Setup-v{#MyAppVersion}
SetupIconFile=assets\ico\canview.ico
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

; 卸载信息
UninstallDisplayIcon={app}\bin\{#MyAppExeName}
UninstallDisplayName={#MyAppName}

[Languages]
Name: "chinesesimplified"; MessagesFile: "ChineseSimplified.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "quicklaunchicon"; Description: "{cm:CreateQuickLaunchIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked; OnlyBelowVersion: 6.1; Check: not IsAdminInstallMode

[Files]
; 主程序
Source: "target\release\view.exe"; DestDir: "{app}\bin"; DestName: "{#MyAppExeName}"; Flags: ignoreversion

; 配置目录
Source: "config\*"; DestDir: "{app}\config"; Flags: ignoreversion recursesubdirs createallsubdirs

; 示例文件
Source: "sample.dbc"; DestDir: "{app}\samples"; Flags: ignoreversion; Check: FileExists('sample.dbc')
Source: "sample.blf"; DestDir: "{app}\samples"; Flags: ignoreversion; Check: FileExists('sample.blf')

; 文档
Source: "README.md"; DestDir: "{app}\docs"; Flags: ignoreversion isreadme; Check: FileExists('README.md')
Source: "BUILD.md"; DestDir: "{app}\docs"; Flags: ignoreversion; Check: FileExists('BUILD.md')
Source: "PACKAGING_GUIDE.md"; DestDir: "{app}\docs"; Flags: ignoreversion; Check: FileExists('PACKAGING_GUIDE.md')
Source: "LIBRARY_MANAGEMENT_COMPLETE.md"; DestDir: "{app}\docs"; Flags: ignoreversion; Check: FileExists('LIBRARY_MANAGEMENT_COMPLETE.md')

; 资源文件
Source: "assets\*"; DestDir: "{app}\assets"; Flags: ignoreversion recursesubdirs createallsubdirs; Check: DirExists('assets')

; NOTE: Don't use "Flags: ignoreversion" on any shared system files

[Dirs]
; 创建必要的目录
Name: "{app}\config\signal_library"; Permissions: users-modify
Name: "{app}\logs"; Permissions: users-modify
Name: "{userappdata}\{#MyAppName}"; Permissions: users-modify

[Icons]
; 开始菜单图标
Name: "{group}\{#MyAppName}"; Filename: "{app}\bin\{#MyAppExeName}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"

; 桌面图标
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\bin\{#MyAppExeName}"; Tasks: desktopicon

; 快速启动图标
Name: "{userappdata}\Microsoft\Internet Explorer\Quick Launch\{#MyAppName}"; Filename: "{app}\bin\{#MyAppExeName}"; Tasks: quicklaunchicon

[Run]
; 安装完成后运行程序
Filename: "{app}\bin\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[Code]
// 自定义安装逻辑

// 检查是否已安装旧版本
function GetUninstallString(): String;
var
  sUnInstPath: String;
  sUnInstallString: String;
begin
  sUnInstPath := ExpandConstant('Software\Microsoft\Windows\CurrentVersion\Uninstall\{#emit SetupSetting("AppId")}_is1');
  sUnInstallString := '';
  if not RegQueryStringValue(HKLM, sUnInstPath, 'UninstallString', sUnInstallString) then
    RegQueryStringValue(HKCU, sUnInstPath, 'UninstallString', sUnInstallString);
  Result := sUnInstallString;
end;

// 卸载旧版本
function IsUpgrade(): Boolean;
begin
  Result := (GetUninstallString() <> '');
end;

function UnInstallOldVersion(): Integer;
var
  sUnInstallString: String;
  iResultCode: Integer;
begin
  Result := 0;
  sUnInstallString := GetUninstallString();
  if sUnInstallString <> '' then begin
    sUnInstallString := RemoveQuotes(sUnInstallString);
    if Exec(sUnInstallString, '/SILENT /NORESTART /SUPPRESSMSGBOXES','', SW_HIDE, ewWaitUntilTerminated, iResultCode) then
      Result := 3
    else
      Result := 2;
  end else
    Result := 1;
end;

// 初始化安装
procedure CurStepChanged(CurStep: TSetupStep);
begin
  if (CurStep=ssInstall) then
  begin
    if (IsUpgrade()) then
    begin
      UnInstallOldVersion();
    end;
  end;
end;

// 创建默认配置文件
procedure CurStepChanged(CurStep: TSetupStep);
var
  ConfigFile: String;
  DefaultConfig: String;
begin
  if CurStep = ssPostInstall then
  begin
    ConfigFile := ExpandConstant('{app}\config\default_config.json');
    if not FileExists(ConfigFile) then
    begin
      DefaultConfig := '{' + #13#10 +
        '  "libraries": [],' + #13#10 +
        '  "mappings": [],' + #13#10 +
        '  "active_library_id": null,' + #13#10 +
        '  "active_version_name": null' + #13#10 +
        '}';
      SaveStringToFile(ConfigFile, DefaultConfig, False);
    end;
  end;
end;
