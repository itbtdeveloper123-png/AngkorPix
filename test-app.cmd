@echo off
title AngkorPix - Quick Test Runner
color 0A
echo ===================================================
echo           AngkorPix - Quick Dev Test
echo ===================================================
echo [INFO] Running AngkorPix in Fast Debug Mode...
echo [INFO] Cargo will reuse cached dependencies.
echo.

:: Optimize GPU VRAM usage for smooth rendering on 1GB/2GB GPUs
set PHOTOCRAFT_GPU_TILE=2048
set PHOTOCRAFT_CANVAS_F16=0

cargo run -p angkorpix

if %ERRORLEVEL% NEQ 0 (
    color 0C
    echo.
    echo [ERROR] Application exited with an error.
    pause
)
