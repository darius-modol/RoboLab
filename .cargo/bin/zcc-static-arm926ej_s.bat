@echo off
setlocal DisableDelayedExpansion

set argfile=%~1
set argfile=%argfile:~1%

for /F "usebackq tokens=*" %%x in (%argfile%) do (
    echo %%x | findstr /i "self-contained.*crt" >nul
    if errorlevel 1 (
        echo %%x >> %argfile%_new
    )
)

zig cc -g -fno-sanitize=all -target arm-linux-musleabi -march=arm926ej_s -mcpu=arm926ej_s @%argfile%_new
