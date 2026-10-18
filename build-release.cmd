@echo off
title AngkorPix - Release Packager
color 0B
echo ===================================================
echo           AngkorPix - Release Packager
echo ===================================================
echo [INFO] Building AngkorPix in Release Mode (Optimized)...
echo.

cargo build --release -p angkorpix

if %ERRORLEVEL% NEQ 0 (
    color 0C
    echo.
    echo [ERROR] Build failed! Please check the error messages above.
    pause
    exit /b %ERRORLEVEL%
)

echo.
echo [INFO] Packaging Windows Binary...

set VERSION=v0.10.0
set DIST_DIR=dist\release
set PKG_NAME=AngkorPix-%VERSION%-windows-x64
set PKG_DIR=%DIST_DIR%\%PKG_NAME%

if not exist "%DIST_DIR%" mkdir "%DIST_DIR%"
if exist "%PKG_DIR%" rd /s /q "%PKG_DIR%"
mkdir "%PKG_DIR%"

copy /y "target\release\angkorpix.exe" "%PKG_DIR%\"
if exist "packaging\windows\portable.txt" copy /y "packaging\windows\portable.txt" "%PKG_DIR%\"

powershell -NoProfile -Command "Compress-Archive -Path '%PKG_DIR%\*' -DestinationPath '%DIST_DIR%\%PKG_NAME%.zip' -Force"

color 0A
echo.
echo ===================================================
echo [SUCCESS] Package created successfully!
echo Location: %DIST_DIR%\%PKG_NAME%.zip
echo ===================================================
echo.
echo To release this file on GitHub:
echo 1. Open: https://github.com/itbtdeveloper123-png/AngkorPix/releases/new
echo 2. Choose tag: %VERSION%
echo 3. Drag and drop: %DIST_DIR%\%PKG_NAME%.zip
echo 4. Click 'Publish release'
echo.

explorer "%DIST_DIR%"
pause
