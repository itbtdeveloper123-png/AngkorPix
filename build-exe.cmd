@echo off
title AngkorPix - Release Exe Builder
color 0B
echo ===================================================
echo          AngkorPix - Build Release .exe
echo ===================================================
echo [INFO] Building optimized standalone release executable...
echo [INFO] This produces maximum performance and full optimization.
echo.

cargo build --release -p angkorpix

if %ERRORLEVEL% NEQ 0 (
    color 0C
    echo.
    echo [ERROR] Build failed! Check the error log above.
    pause
    exit /b %ERRORLEVEL%
)

echo.
echo [INFO] Copying AngkorPix.exe to dist\ folder...
if not exist "dist" mkdir "dist"
copy /Y "target\release\angkorpix.exe" "dist\AngkorPix.exe" >nul

color 0A
echo.
echo ===================================================
echo [SUCCESS] Build completed successfully!
echo Executable location:
echo   - dist\AngkorPix.exe
echo   - target\release\angkorpix.exe
echo ===================================================
echo.
pause
