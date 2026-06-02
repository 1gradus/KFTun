::
:: package.cmd [-win7] [-dev | -rel] [-run] [-clean] [-zip]
::
@echo off
@setlocal
@set ERRORLEVEL=
@cd /D %~dp0

set -win7=
set -dev=
set -rel=
set -run=
set -clean=
set -zip=
set --=

set RunArgs=
:args
if "%~1x" == "--x" (
    set --=1
    shift /1
    goto :args
) else if "%~1x" == "-win7x" (
    set -win7=1
    shift /1
    goto :args
) else if "%~1x" == "-devx" (
    set -dev=1
    set -rel=
    shift /1
    goto :args
) else if "%~1x" == "-relx" (
    set -dev=
    set -rel=1
    shift /1
    goto :args
) else if "%~1x" == "-runx" (
    set -run=1
    shift /1
    goto :args
) else if "%~1x" == "-cleanx" (
    set -clean=1
    shift /1
    goto :args
) else if "%~1x" == "-zipx" (
    set -zip=1
    shift /1
    goto :args
) else if not "%~1x" == "x" (
    if defined -- (
        set "RunArgs=%RunArgs%%1 "
    ) else (
        echo [ ERROR ] unknown option '%~1'
        cmd /D /C exit /B 1
    )
    shift /1
    goto :args
)
if ERRORLEVEL 1 (
    exit /B %ERRORLEVEL%
)

if defined -win7 (
    set PackageName=KFTun-win7
    set BuildOpts=--target x86_64-win7-windows-msvc -Zunstable-options -Zbuild-std
    set TargetDir=target\x86_64-win7-windows-msvc\release
    set RUSTFLAGS=
) else (
    set PackageName=KFTun
    set BuildOpts=
    set TargetDir=target\release
)

if defined -dev (
    set PackageName=%PackageName%-dev
    set RunCmd=run-dev.cmd
) else if defined -rel (
    set RunCmd=run.cmd
) else (
    set RunCmd=
)

set PackageDir=package\%PackageName%

cargo build --release --offline %BuildOpts%

if ERRORLEVEL 1 (
    exit /B %ERRORLEVEL%
)

if defined -clean (
    rmdir /S /Q package
) else (
    rmdir /S /Q %PackageDir%
)
mkdir %PackageDir%

copy /Y %TargetDir%\*.exe %PackageDir%\
if defined RunCmd (
    copy /Y etc\%RunCmd% %PackageDir%\run.cmd
)

if defined -zip (
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
)

if defined -run (
    call %PackageDir%\run.cmd %RunArgs%
)
