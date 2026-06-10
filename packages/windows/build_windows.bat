@echo off
setlocal enabledelayedexpansion

echo Building Windows Driver Package for libomid (Native)...

:: Navigate to project root
cd /d "%~dp0..\.."

:: Compile library
cargo build --release --all-features
if %ERRORLEVEL% neq 0 (
    echo Error: Compilation failed.
    exit /b %ERRORLEVEL%
)

:: Set up packaging folders
set PKG_DIR=target\windows-package
if exist "%PKG_DIR%" rd /s /q "%PKG_DIR%"
mkdir "%PKG_DIR%\bin"
mkdir "%PKG_DIR%\lib"
mkdir "%PKG_DIR%\include"

:: Copy dll, lib, and header assets
copy target\release\omid.dll "%PKG_DIR%\bin\omid.dll"
copy target\release\omid.dll.lib "%PKG_DIR%\lib\omid.lib"
copy include\omid.h "%PKG_DIR%\include\omid.h"
copy bindings\cpp\omid.hpp "%PKG_DIR%\include\omid.hpp"
copy LICENSE-MIT "%PKG_DIR%\LICENSE-MIT"
copy LICENSE-APACHE "%PKG_DIR%\LICENSE-APACHE"

echo Windows Driver Package structured successfully at target\windows-package\
pause
