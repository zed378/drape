@echo off
setlocal
rem Run from the repository root. Rust/MSVC changes are local to this process.
if exist ".tools\cargo\bin\cargo.exe" (
    set "CARGO_HOME=%CD%\.tools\cargo"
    set "RUSTUP_HOME=%CD%\.tools\rustup"
    set "PATH=%CD%\.tools\cargo\bin;%PATH%"
)
set "DRAPE_VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
if not exist "%DRAPE_VSWHERE%" (
    echo Visual Studio C++ Build Tools and Windows SDK are required. 1>&2
    exit /b 1
)
rem Include installations awaiting setup completion, but require the C++ component.
rem VsDevCmd and the linker/SDK checks below remain the build authority.
for /f "usebackq delims=" %%i in (`"%DRAPE_VSWHERE%" -latest -all -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) do set "DRAPE_VS=%%i"
if not defined DRAPE_VS (
    echo No Visual Studio installation with x64 C++ tools was found. 1>&2
    exit /b 1
)
call "%DRAPE_VS%\Common7\Tools\VsDevCmd.bat" -no_logo -arch=x64 -host_arch=x64
if errorlevel 1 exit /b %ERRORLEVEL%
where link.exe >nul 2>&1
if errorlevel 1 (
    echo MSVC linker unavailable. Install the C++ x64 tools and Windows SDK. 1>&2
    exit /b 1
)
rtk proxy cargo %*
exit /b %ERRORLEVEL%
