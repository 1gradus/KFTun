::
:: package.cmd [-win7] [--run|--run-dev]
::
@echo off
@setlocal
@set ERRORLEVEL=
@cd /D %~dp0

if "%1x" == "-win7x" (
    set PackageName=KFTun-win7
    set BuildOpts=--target x86_64-win7-windows-msvc -Zunstable-options -Zbuild-std
    set TargetDir=target\x86_64-win7-windows-msvc\release
    set RunDevOpts=-win7
    set RUSTFLAGS=
    shift /1
) else (
    set PackageName=KFTun
    set BuildOpts=
    set TargetDir=target\release
    set RunDevOpts=
)
set PackageDir=.\package\%PackageName%

cargo build --release --offline %BuildOpts%

if ERRORLEVEL 1 (
    exit /B %ERRORLEVEL%
)

rmdir /S /Q %PackageDir%
mkdir %PackageDir%

copy /Y %TargetDir%\*.exe %PackageDir%\
copy /Y etc\run.cmd %PackageDir%\

pushd package
    if not exist %PackageName%.zip (
        where /Q 7z.exe
        if not ERRORLEVEL 1 (
            7z.exe a -tzip -sse -ssp %PackageName%.zip %PackageName%
        )
    )
    if not exist %PackageName%.zip (
        where /Q zip.exe
        if not ERRORLEVEL 1 (
            zip.exe -r %PackageName%.zip %PackageName%
        )
    )
    if not exist %PackageName%.zip (
        echo "Not creating the archive. No archivers available."
    )
popd

if "%1x" == "--runx" (
    echo ----------------------------------------
    call %PackageDir%\run.cmd
) else if "%1x" == "--run-devx" (
    echo ----------------------------------------
    call etc\run-dev.cmd %RunDevOpts%
)
