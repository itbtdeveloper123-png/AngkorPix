@echo off
title AngkorPix - Quick Test Runner
color 0A
echo ===================================================
echo           AngkorPix - Quick Dev Test
echo ===================================================
echo [INFO] Running AngkorPix in Fast Debug Mode...
echo [INFO] Cargo will reuse cached dependencies.
echo.

cargo run -p angkorpix

if %ERRORLEVEL% NEQ 0 (
    color 0C
    echo.
    echo [ERROR] Application exited with an error.
    pause
)
