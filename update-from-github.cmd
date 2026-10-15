@echo off
title AngkorPix - Update from GitHub
color 0B
echo ===================================================
echo       AngkorPix - Update Latest Code from GitHub
echo ===================================================
echo.

:: 1. Ensure we are on the AngkorPix working branch
git branch --show-current > "%TEMP%\angkorpix_branch.txt"
set /p CURRENT_BRANCH=<"%TEMP%\angkorpix_branch.txt"
del "%TEMP%\angkorpix_branch.txt" 2>nul

if not "%CURRENT_BRANCH%"=="AngkorPix" (
    echo [INFO] Switching to AngkorPix branch...
    git checkout AngkorPix
)

echo [1/3] Pulling latest AngkorPix updates from origin...
git pull origin AngkorPix

if %ERRORLEVEL% NEQ 0 (
    color 0C
    echo.
    echo [WARNING] Could not pull from origin AngkorPix.
    echo Please check your internet connection or git status.
    pause
    exit /b %ERRORLEVEL%
)

echo.
echo [2/3] Fetching upstream updates (if available)...
git fetch upstream main 2>nul
if %ERRORLEVEL% EQU 0 (
    echo [INFO] Merging upstream updates into AngkorPix...
    git merge upstream/main -m "Merge upstream updates into AngkorPix" --no-edit 2>nul
)

echo.
echo [3/3] Compiling and caching dependencies...
call cargo check -p angkorpix

color 0A
echo.
echo ===================================================
echo [SUCCESS] AngkorPix is fully updated and ready!
echo You can run test-app.cmd or build-exe.cmd now.
echo ===================================================
echo.
pause

