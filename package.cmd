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
zip -r KFTun.zip KFTun
popd

if "%1x" == "--runx" (
    echo ----------------------------------------
    call %PackageDir%\run.cmd
)
