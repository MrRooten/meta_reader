@echo off
for /f "delims=" %%i in ('fsutil file queryFileId %1') do set output=%%i
set /A  hex=0x%output:~-12%
echo %hex%