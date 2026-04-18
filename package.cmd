::
:: package.cmd [-win7] [-dev] [-run]
::
@echo off
@setlocal
@set ERRORLEVEL=
@cd /D %~dp0

if "%1x" == "-win7x" (
    set PackageName=KFTun-win7
    set BuildOpts=--target x86_64-win7-windows-msvc -Zunstable-options -Zbuild-std
    set TargetDir=target\x86_64-win7-windows-msvc\release
    set RUSTFLAGS=
    shift /1
) else (
    set PackageName=KFTun
    set BuildOpts=
    set TargetDir=target\release
)
if "%1x" == "-cleanx" (
    rmdir /S /Q package
    shift /1
)
if "%1x" == "-devx" (
    set PackageName=%PackageName%-dev
    set RunCmd=run-dev.cmd
    shift /1
) else (
    set RunCmd=run.cmd
)
set PackageDir=package\%PackageName%

cargo build --release --offline %BuildOpts%

if ERRORLEVEL 1 (
    exit /B %ERRORLEVEL%
)

rmdir /S /Q %PackageDir%
mkdir %PackageDir%

copy /Y %TargetDir%\*.exe %PackageDir%\
copy /Y etc\%RunCmd% %PackageDir%\run.cmd

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

if "%1x" == "-runx" (
    call %PackageDir%\run.cmd %2 %3 %4 %5 %6 %7 %8 %9
)
