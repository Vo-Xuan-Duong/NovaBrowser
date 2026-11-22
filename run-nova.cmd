@echo off
setlocal
cd /d "%~dp0"
cargo run -p nova-desktop --release %*
exit /b %ERRORLEVEL%
