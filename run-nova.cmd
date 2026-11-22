@echo off
setlocal
cd /d "%~dp0"
cargo run --manifest-path "apps\nova-browser\Cargo.toml" --release %*
exit /b %ERRORLEVEL%
