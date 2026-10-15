@echo off
title AngkorPix - Compile & Check Dependencies
color 0B
echo ===================================================
echo     AngkorPix - Pre-compile New Dependencies
echo ===================================================
echo [INFO] Fetching and compiling new libraries/crates...
echo [INFO] This will prepare and cache all dependencies.
echo.

cargo check -p angkorpix

if %ERRORLEVEL% NEQ 0 (
    color 0C
    echo.
    echo [ERROR] Dependency check failed!
    pause
    exit /b %ERRORLEVEL%
)

color 0A
echo.
echo ===================================================
echo [SUCCESS] All libraries and dependencies compiled!
echo You can now run test-app.cmd or build-exe.cmd quickly.
echo ===================================================
echo.
pause
