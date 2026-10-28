@echo off
setlocal enabledelayedexpansion
title AngkorPix - Build Windows Installer & Release Package
color 0B

echo ========================================================
echo     AngkorPix - Windows Installer & Package Builder
echo ========================================================
echo.

:: 1. Ensure Release Binary Exists
if not exist "target\release\angkorpix.exe" (
    echo [INFO] Release binary not found. Building AngkorPix in release mode...
    cargo build --release -p angkorpix
    if %ERRORLEVEL% NEQ 0 (
        color 0C
        echo.
        echo [ERROR] Build failed! Please check cargo errors above.
        pause
        exit /b %ERRORLEVEL%
    )
) else (
    echo [OK] Found target\release\angkorpix.exe
)

:: 2. Create Portable Release Distribution Bundle
set DIST_DIR=target\AngkorPix-v0.11.1-Windows-x64
echo.
echo [INFO] Preparing portable distribution directory: %DIST_DIR%
if exist "%DIST_DIR%" rmdir /s /q "%DIST_DIR%"
mkdir "%DIST_DIR%"
mkdir "%DIST_DIR%\assets"
mkdir "%DIST_DIR%\scripts"
mkdir "%DIST_DIR%\models"

copy "target\release\angkorpix.exe" "%DIST_DIR%\" >nul
xcopy "assets" "%DIST_DIR%\assets" /E /I /Y >nul
xcopy "scripts" "%DIST_DIR%\scripts" /E /I /Y >nul
if exist "models" xcopy "models" "%DIST_DIR%\models" /E /I /Y >nul
copy "setup-ai-directml.cmd" "%DIST_DIR%\" >nul
copy "README.md" "%DIST_DIR%\" >nul
copy "LICENSE" "%DIST_DIR%\" >nul

echo [OK] Portable bundle ready at %DIST_DIR%

:: 3. Check for Inno Setup Compiler (ISCC)
echo.
echo [INFO] Looking for Inno Setup Compiler (ISCC.exe)...
set ISCC_PATH=

where iscc >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    set ISCC_PATH=iscc
) else if exist "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" (
    set "ISCC_PATH=C:\Program Files (x86)\Inno Setup 6\ISCC.exe"
) else if exist "C:\Program Files\Inno Setup 6\ISCC.exe" (
    set "ISCC_PATH=C:\Program Files\Inno Setup 6\ISCC.exe"
) else if exist "%LOCALAPPDATA%\Programs\Inno Setup 6\ISCC.exe" (
    set "ISCC_PATH=%LOCALAPPDATA%\Programs\Inno Setup 6\ISCC.exe"
)

if defined ISCC_PATH (
    echo [OK] Found Inno Setup: "%ISCC_PATH%"
    echo [INFO] Compiling setup installer...
    if not exist "target\installer" mkdir "target\installer"
    "%ISCC_PATH%" "packaging\windows\angkorpix.iss"
    if %ERRORLEVEL% EQU 0 (
        color 0A
        echo.
        echo ========================================================
        echo [SUCCESS] Windows Installer created successfully!
        echo Location: target\installer\AngkorPix-Setup-v0.11.1.exe
        echo ========================================================
        echo.
        pause
        exit /b 0
    ) else (
        echo [WARNING] ISCC compilation failed with code %ERRORLEVEL%.
    )
) else (
    echo [NOTE] Inno Setup is not installed on this system.
    echo To build AngkorPix-Setup.exe installer, you can install Inno Setup by running:
    echo     winget install JRSoftware.InnoSetup
    echo.
    echo Meanwhile, your portable distribution package is fully prepared at:
    echo     %DIST_DIR%
)

color 0A
echo.
echo ========================================================
echo [SUCCESS] AngkorPix packaging completed!
echo ========================================================
echo.
pause

