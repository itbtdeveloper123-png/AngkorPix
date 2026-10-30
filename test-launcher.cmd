@echo off
title ArtPixels Launcher
color 0B
echo ===================================================
echo               ArtPixels Launcher
echo ===================================================
echo [INFO] Launching ArtPixels Launcher...
echo.

cargo run --manifest-path launcher/Cargo.toml -p artpixels

if %ERRORLEVEL% NEQ 0 (
    color 0C
    echo.
    echo [ERROR] Launcher exited with an error.
    pause
)

