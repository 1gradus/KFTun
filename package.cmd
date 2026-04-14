@echo off
@setlocal
@set ERRORLEVEL=
@cd /D %~dp0

cargo build --release --offline

if ERRORLEVEL 1 (
    exit /B %ERRORLEVEL%
)

set PackageDir=.\package\KFTun

rmdir /S /Q package
mkdir %PackageDir%

copy /Y target\release\*.exe %PackageDir%\
copy /Y etc\run.cmd %PackageDir%\

pushd package
    if not exist KFTun.zip (
        where /Q 7z.exe
        if not ERRORLEVEL 1 (
            7z.exe a -tzip -sse -ssp KFTun.zip KFTun
        )
    )
    if not exist KFTun.zip (
        where /Q zip.exe
        if not ERRORLEVEL 1 (
            zip.exe -r KFTun.zip KFTun
        )
    )
    if not exist KFTun.zip (
        echo "Not creating the archive. No archivers available."
    )
popd

if "%1x" == "--runx" (
    echo ----------------------------------------
    call %PackageDir%\run.cmd
) else if "%1x" == "--run-devx" (
    echo ----------------------------------------
    call etc\run-dev.cmd
)
