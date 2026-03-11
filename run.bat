@echo off
setlocal

cd /d "%~dp0"

set "PROFILE=debug"

if "%~1"=="--release" (
    set "PROFILE=release"
    shift
)
if "%~1"=="--debug" (
    set "PROFILE=debug"
    shift
)

if "%PROFILE%"=="release" (
    cargo run --release --package schematic_editor -- %*
) else (
    cargo run --package schematic_editor -- %*
)
