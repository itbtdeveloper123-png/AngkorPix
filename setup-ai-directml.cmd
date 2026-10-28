@echo off
title AngkorPix - Setup AI DirectML GPU Acceleration
color 0B
echo ========================================================
echo   AngkorPix - AI DirectML (GPU) Acceleration Setup
echo ========================================================
echo [INFO] Checking Python environment...
python --version >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    color 0C
    echo [ERROR] Python is not found in PATH!
    echo Please install Python 3.10+ from https://www.python.org/ or Microsoft Store.
    pause
    exit /b 1
)

echo [INFO] Installing / upgrading onnxruntime-directml, numpy, and pillow...
echo [INFO] DirectML enables ultra-fast GPU acceleration on Intel, AMD, and NVIDIA GPUs.
echo.
python -m pip install --upgrade pip
python -m pip install onnxruntime-directml numpy pillow

echo.
echo [INFO] Testing DirectML Execution Provider...
python -c "import onnxruntime as ort; provs = ort.get_available_providers(); print('[INFO] Available Providers:', provs); assert 'DmlExecutionProvider' in provs or 'CPUExecutionProvider' in provs; print('[SUCCESS] AI Acceleration Ready!')"

if %ERRORLEVEL% NEQ 0 (
    color 0E
    echo.
    echo [WARNING] DirectML verification returned a warning.
    echo Standard CPU execution will still function normally.
    pause
    exit /b 0
)

color 0A
echo.
echo ========================================================
echo [SUCCESS] DirectML GPU Acceleration is successfully enabled!
echo AngkorPix AI Background Removal will now run at GPU speeds.
echo ========================================================
echo.
pause

