@echo off
title AngkorPix - Sync with Upstream GitHub
color 0E
echo ===================================================
echo     AngkorPix - Sync Upstream GitHub Updates
echo ===================================================
echo [1/4] Fetching latest updates from upstream...
git fetch upstream

echo.
echo [2/4] Merging upstream/main into main...
git checkout main
git pull upstream main
git push origin main

echo.
echo [3/4] Returning to AngkorPix working branch...
git checkout AngkorPix

color 0A
echo.
echo ===================================================
echo [SUCCESS] Upstream sync completed!
echo ===================================================
pause
