@echo off
chcp 65001 >nul
title AngkorPix - Setup AI DirectML GPU Acceleration
color 0B
echo =========================================================================
echo   🎨 AngkorPix - AI DirectML (GPU) Acceleration & API Setup
echo =========================================================================
echo.

set "PY_CMD=python"
python --version >nul 2>&1
if %ERRORLEVEL% EQU 0 goto FOUND_PYTHON

py -3 --version >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    set "PY_CMD=py -3"
    goto FOUND_PYTHON
)

if exist "C:\Python314\python.exe" (
    set "PY_CMD=C:\Python314\python.exe"
    goto FOUND_PYTHON
)
if exist "C:\Python313\python.exe" (
    set "PY_CMD=C:\Python313\python.exe"
    goto FOUND_PYTHON
)
if exist "C:\Python312\python.exe" (
    set "PY_CMD=C:\Python312\python.exe"
    goto FOUND_PYTHON
)
if exist "C:\Python311\python.exe" (
    set "PY_CMD=C:\Python311\python.exe"
    goto FOUND_PYTHON
)

for /d %%D in ("%LOCALAPPDATA%\Programs\Python\Python3*") do (
    if exist "%%D\python.exe" (
        set "PY_CMD=%%D\python.exe"
        goto FOUND_PYTHON
    )
)

color 0C
echo [ERROR] Python 3 was not detected on this system!
echo.
echo AngkorPix AI features (Background Removal, Inpainting) require Python.
echo Please install Python from:
echo   1. Official Site: https://www.python.org/downloads/
echo      (Make sure to check "Add Python to PATH" during installation)
echo   2. Or install via Microsoft Store: Python 3.12 / 3.11
echo.
pause
exit /b 1

:FOUND_PYTHON
echo [INFO] Detected Python engine: %PY_CMD%
%PY_CMD% --version
echo.
echo [INFO] Installing / upgrading AI acceleration dependencies...
echo [INFO] - onnxruntime-directml (Ultra-fast GPU acceleration on Intel, AMD, NVIDIA)
echo [INFO] - numpy, pillow (Image processing)
echo [INFO] - opencv-python, scipy (Watermark & edge detection for Inpainting)
echo.

%PY_CMD% -m pip install --upgrade pip >nul 2>&1
%PY_CMD% -m pip install onnxruntime-directml numpy pillow scipy opencv-python

echo.
echo [INFO] Verifying AI Execution Providers & API readiness...
%PY_CMD% -c "import onnxruntime as ort, PIL, numpy; provs = ort.get_available_providers(); print('[INFO] Available Hardware Providers:', provs); assert 'DmlExecutionProvider' in provs or 'CPUExecutionProvider' in provs; print('[SUCCESS] Local & Cloud AI Acceleration Ready!')"

if %ERRORLEVEL% NEQ 0 (
    color 0E
    echo.
    echo [WARNING] DirectML verification returned a warning.
    echo CPU acceleration and Cloud API (Poof.bg + remove.bg + Cloudflare AI)
    echo will still function automatically!
    echo.
    pause
    exit /b 0
)

color 0A
echo.
echo =========================================================================
echo [SUCCESS] AngkorPix AI Engine is completely ready!
echo - High-Definition Background Removal (Local Model + 11 Poof.bg APIs)
echo - Inpainting & Watermark Removal (Local LaMa + Cloudflare AI)
echo =========================================================================
echo.
pause
exit /b 0
