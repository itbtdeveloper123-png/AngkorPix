@echo off
setlocal enabledelayedexpansion
title ArtPixels Suite - Release Builder & Packager
color 0B

:: Ensure cargo and rust tools are in PATH
set "PATH=E:\.cargo\bin;E:\cargo\bin;%USERPROFILE%\.cargo\bin;%PATH%"

:: Determine root directory and paths
if exist "%~dp0photocraft\Cargo.toml" (
    set "ROOT_DIR=%~dp0"
    set "PHOTOCRAFT_DIR=%~dp0photocraft"
) else if exist "%~dp0Cargo.toml" (
    set "ROOT_DIR=%~dp0..\"
    set "PHOTOCRAFT_DIR=%~dp0"
) else (
    echo [ERROR] Could not find project directory!
    pause
    exit /b 1
)

set "VECTOR_DIR=%PHOTOCRAFT_DIR%\vectorcraft"
set "LAUNCHER_DIR=%PHOTOCRAFT_DIR%\launcher"
set "RELEASES_DIR=%ROOT_DIR%RELEASES"

if not exist "%RELEASES_DIR%" mkdir "%RELEASES_DIR%"

:MENU
cls
echo ======================================================================
echo           ArtPixels Creative Suite - Release Builder & Packager
echo ======================================================================
echo.
echo   [1] Build and Pack AngkorPix         (v0.17.0  - Photo Editor)
echo   [2] Build and Pack AngkorVector      (v0.18.0  - Vector Illustrator)
echo   [3] Build and Pack ArtPixels         (v0.10.0  - Desktop Launcher)
echo   [4] Build and Pack ALL THREE APPS    (Complete Suite)
echo   [5] Open RELEASES Folder in Explorer
echo   [6] Clean Old Debug Caches           (Free up 100+ GB disk space)
echo   [0] Exit
echo.
echo ======================================================================
set /p CHOICE="Choose an option [1-6, or 0]: "

if "%CHOICE%"=="1" goto BUILD_ANGKORPIX
if "%CHOICE%"=="2" goto BUILD_ANGKORVECTOR
if "%CHOICE%"=="3" goto BUILD_ARTPIXELS
if "%CHOICE%"=="4" goto BUILD_ALL
if "%CHOICE%"=="5" (
    start explorer "%RELEASES_DIR%"
    goto MENU
)
if "%CHOICE%"=="6" goto CLEAN_DEBUG
if "%CHOICE%"=="0" exit /b 0
goto MENU

:CLOSE_RUNNING
echo [INFO] Closing running instances to prevent file locking...
taskkill /F /IM angkorpix.exe 2>nul
taskkill /F /IM angkorvector.exe 2>nul
taskkill /F /IM artpixels.exe 2>nul
timeout /t 1 /nobreak >nul
goto :eof

:CLEAN_DEBUG
cls
echo ======================================================================
echo           Cleaning Old Debug Build Caches to Free Up Disk Space
echo ======================================================================
echo.
call :CLOSE_RUNNING
echo [INFO] Cleaning AngkorPix debug cache...
if exist "%PHOTOCRAFT_DIR%\target\debug" rd /s /q "%PHOTOCRAFT_DIR%\target\debug"
echo [INFO] Cleaning AngkorVector debug cache...
if exist "%VECTOR_DIR%\target\debug" rd /s /q "%VECTOR_DIR%\target\debug"
echo [INFO] Cleaning ArtPixels Launcher debug cache...
if exist "%LAUNCHER_DIR%\target\debug" rd /s /q "%LAUNCHER_DIR%\target\debug"
echo.
echo [SUCCESS] Debug caches cleaned successfully!
pause
goto MENU

:BUILD_ANGKORPIX
call :CLOSE_RUNNING
echo.
echo ======================================================================
echo   [1/3] Building AngkorPix v0.17.0 in Release Mode...
echo ======================================================================
cd /d "%PHOTOCRAFT_DIR%"
cargo build --release -p angkorpix
if %ERRORLEVEL% NEQ 0 (
    color 0C
    echo [ERROR] Failed to build AngkorPix!
    pause
    color 0B
    goto MENU
)

echo [INFO] Packaging AngkorPix into zip...
set "ZIP_PIX=%RELEASES_DIR%\angkorpix-0.17.0-windows-x64-portable.zip"
if exist "%ZIP_PIX%" del /f /q "%ZIP_PIX%"
tar -a -c -f "%ZIP_PIX%" -C "%PHOTOCRAFT_DIR%\target\release" "angkorpix.exe"

echo [SUCCESS] Created: %ZIP_PIX%
if "%RUN_ALL%"=="1" goto AFTER_PIX
pause
goto SUMMARY

:BUILD_ANGKORVECTOR
call :CLOSE_RUNNING
echo.
echo ======================================================================
echo   [2/3] Building AngkorVector v0.18.0 in Release Mode...
echo ======================================================================
cd /d "%VECTOR_DIR%"
cargo build --release -p angkorvector
if %ERRORLEVEL% NEQ 0 (
    color 0C
    echo [ERROR] Failed to build AngkorVector!
    pause
    color 0B
    goto MENU
)

echo [INFO] Packaging AngkorVector into zip...
set "ZIP_VEC=%RELEASES_DIR%\angkorvector-0.18.0-windows-x64-portable.zip"
if exist "%ZIP_VEC%" del /f /q "%ZIP_VEC%"
tar -a -c -f "%ZIP_VEC%" -C "%VECTOR_DIR%\target\release" "angkorvector.exe"

echo [SUCCESS] Created: %ZIP_VEC%
if "%RUN_ALL%"=="1" goto AFTER_VEC
pause
goto SUMMARY

:BUILD_ARTPIXELS
call :CLOSE_RUNNING
echo.
echo ======================================================================
echo   [3/3] Building ArtPixels Launcher v0.10.0 in Release Mode...
echo ======================================================================
cd /d "%LAUNCHER_DIR%"
cargo build --release -p artpixels
if %ERRORLEVEL% NEQ 0 (
    color 0C
    echo [ERROR] Failed to build ArtPixels Launcher!
    pause
    color 0B
    goto MENU
)

echo [INFO] Packaging ArtPixels Launcher into zip...
set "ZIP_LAUNCH=%RELEASES_DIR%\artpixels-0.10.0-windows-x64-portable.zip"
if exist "%ZIP_LAUNCH%" del /f /q "%ZIP_LAUNCH%"
tar -a -c -f "%ZIP_LAUNCH%" -C "%LAUNCHER_DIR%\target\release" "artpixels.exe"
copy /Y "%ZIP_LAUNCH%" "%RELEASES_DIR%\artcraft-launcher-0.10.0-windows-x64-portable.zip" >nul

echo [SUCCESS] Created: %ZIP_LAUNCH%
if "%RUN_ALL%"=="1" goto AFTER_LAUNCH
pause
goto SUMMARY

:BUILD_ALL
set "RUN_ALL=1"
goto BUILD_ANGKORPIX
:AFTER_PIX
goto BUILD_ANGKORVECTOR
:AFTER_VEC
goto BUILD_ARTPIXELS
:AFTER_LAUNCH
set "RUN_ALL=0"
goto SUMMARY

:SUMMARY
cls
color 0A
echo ======================================================================
echo               ALL PACKAGES READY FOR GITHUB RELEASE!
echo ======================================================================
echo.
echo Saved in folder: %RELEASES_DIR%
echo.
dir /b "%RELEASES_DIR%\*.zip"
echo.
echo ----------------------------------------------------------------------
echo Upload each package to its respective GitHub Release:
echo.
echo 1. AngkorPix:
echo    Tag:   v0.17.0
echo    File:  angkorpix-0.17.0-windows-x64-portable.zip
echo    URL:   https://github.com/itbtdeveloper123-png/AngkorPix/releases/new
echo.
echo 2. AngkorVector:
echo    Tag:   v0.18.0
echo    File:  angkorvector-0.18.0-windows-x64-portable.zip
echo    URL:   https://github.com/itbtdeveloper123-png/AngkorVector/releases/new
echo.
echo 3. ArtPixels (Launcher):
echo    Tag:   v0.10.0
echo    File:  artpixels-0.10.0-windows-x64-portable.zip
echo    URL:   https://github.com/itbtdeveloper123-png/ArtPixels/releases/new
echo ======================================================================
echo.
start explorer "%RELEASES_DIR%"
pause
color 0B
goto MENU
