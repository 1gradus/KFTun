@echo off
@setlocal
@set ERRORLEVEL=
@cd /D %~dp0

set RUSTFLAGS=

cargo build --release --offline --target x86_64-win7-windows-msvc -Zunstable-options -Zbuild-std

if ERRORLEVEL 1 (
    exit /B %ERRORLEVEL%
)

set PackageDir=.\package\KFTun-win7

rmdir /S /Q %PackageDir%
mkdir %PackageDir%

copy /Y target\x86_64-win7-windows-msvc\release\*.exe %PackageDir%\
copy /Y etc\run.cmd %PackageDir%\

pushd package
zip -r KFTun-win7.zip KFTun-win7
popd

if "%1x" == "--runx" (
    echo ----------------------------------------
    call %PackageDir%\run.cmd
) else if "%1x" == "--run-devx" (
    echo ----------------------------------------
    call etc\run-dev.cmd -win7
)
