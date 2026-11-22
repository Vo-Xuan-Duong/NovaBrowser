@echo off
setlocal
cd /d "%~dp0"
rem Build the Servo renderer independently to avoid the V8/SpiderMonkey ICU conflict.
cargo build --manifest-path "engines\servo-native\Cargo.toml" --release
if errorlevel 1 exit /b %errorlevel%
cargo run -p nova-desktop --release --features servo-embed %*
exit /b %errorlevel%
