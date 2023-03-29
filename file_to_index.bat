@echo off
for /f "delims=" %%i in ('fsutil file queryFileId %1') do set output=%%i
echo 0x%output:~-12%