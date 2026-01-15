@echo off
echo Starting SSO Platform in Production Mode...

REM Check if Rust is installed
rustc --version >nul 2>&1
if %errorlevel% neq 0 (
    echo Error: Rust is not installed or not in PATH
    echo Please install Rust from https://www.rust-lang.org/tools/install
    exit /b 1
)

REM Build the application
echo Building application...
cargo build --release
if %errorlevel% neq 0 (
    echo Error: Build failed
    exit /b 1
)

echo.
echo Starting SSO Platform...
echo Press Ctrl+C to stop the server
echo.

REM Run the application
cargo run --release --bin auth-platform