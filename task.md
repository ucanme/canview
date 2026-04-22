在github action上编译失败
0s
mac Run chmod +x scripts/package-macos.sh
/Users/runner/work/_temp/f585c0b1-0390-4c3c-a11d-5b529dd540d0.sh: ./scripts/package-macos.sh: /bin/bash^M: bad interpreter: No such file or directory
warning: `view` (bin "view") generated 381 warnings (run `cargo fix --bin "view" -p view` to apply 39 suggestions)

window
Finished `release` profile [optimized] target(s) in 0.93s
✅ 编译成功！

📁 步骤 3: 准备文件...
✅ 文件准备完成！

🔨 步骤 4: 构建安装程序...
Inno Setup 6 Command-Line Compiler
Copyright (C) 1997-2026 Jordan Russell. All rights reserved.
Portions Copyright (C) 2000-2026 Martijn Laan. All rights reserved.
Portions Copyright (C) 2001-2004 Alex Yackimoff. All rights reserved.
https://www.innosetup.com

Compiler engine version: Inno Setup 6.7.1
Non-commercial use only

Preprocessing
Reading file: C:\Program Files (x86)\Inno Setup 6\ISPPBuiltins.iss
Parsing [Setup] section, line 12
Parsing [Setup] section, line 13
Parsing [Setup] section, line 14
Parsing [Setup] section, line 15
Parsing [Setup] section, line 16
Parsing [Setup] section, line 17
Parsing [Setup] section, line 18
Parsing [Setup] section, line 19
Parsing [Setup] section, line 20
Parsing [Setup] section, line 21
Parsing [Setup] section, line 22
Parsing [Setup] section, line 23
Parsing [Setup] section, line 24
Parsing [Setup] section, line 25
Parsing [Setup] section, line 26
Parsing [Setup] section, line 27
Parsing [Setup] section, line 28
Parsing [Setup] section, line 29
Parsing [Setup] section, line 30
Parsing [Setup] section, line 31
Parsing [Setup] section, line 34
Parsing [Setup] section, line 35
Creating output directory: D:\a\canview\canview\installer-output
Reading file (LicenseFile)
Reading file (WizardImageFile)
Reading file (WizardSmallImageFile)
Preparing Setup program executable
Verification successful
Updating icons (Setup.e32)
Updating version info (Setup.e32)
Determining language code pages
Parsing [Languages] section, line 38
Reading file: D:\a\canview\canview\ChineseSimplified.isl
Parsing [Languages] section, line 39
Reading file: C:\Program Files (x86)\Inno Setup 6\Default.isl
Messages in script file
Reading default messages from Default.isl
Parsing [Languages] section, line 38
Reading file: D:\a\canview\canview\ChineseSimplified.isl
Parsing [Languages] section, line 39
Reading file: C:\Program Files (x86)\Inno Setup 6\Default.isl
Parsing [LangOptions], [Messages], and [CustomMessages] sections
Messages in script file
Reading [Code] section
Parsing [Tasks] section, line 42
Parsing [Tasks] section, line 43
Parsing [Dirs] section, line 68
Parsing [Dirs] section, line 69
Parsing [Dirs] section, line 70
Parsing [Icons] section, line 74
Parsing [Icons] section, line 75
Parsing [Icons] section, line 78
Parsing [Icons] section, line 81
Parsing [Run] section, line 85
Parsing [Files] section, line 47
Parsing [Files] section, line 50
Parsing [Files] section, line 53
Parsing [Files] section, line 54
Error on line 54 in D:\a\canview\canview\installer.iss: Source file "D:\a\canview\canview\sample.blf" does not exist.
Compile aborted.
❌ 安装程序构建失败！
Error: Process completed with exit co

linux:
Run chmod +x scripts/package-linux.sh
/home/runner/work/_temp/80a292fa-70c2-4600-b3f5-807446d77255.sh: line 2: ./scripts/package-linux.sh: cannot execute: required file not found
Error: Process completed with exit code 127.