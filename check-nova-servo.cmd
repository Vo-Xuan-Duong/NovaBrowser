@echo off
setlocal EnableExtensions
cd /d "%~dp0"

rem Local checks only. This script never starts GitHub Actions or modifies code.
where cargo >nul 2>&1
if errorlevel 1 (
  echo ERROR: Rust Cargo is not installed or not on PATH.
  exit /b 1
)
where node >nul 2>&1
if errorlevel 1 (
  echo ERROR: Node.js is needed for the browser chrome contract tests.
  exit /b 1
)

echo [1/5] Checking Servo/Obscura dependency isolation and IPC...
node "engines\servo-native\tests\isolation.cjs"
if errorlevel 1 exit /b 1

echo [2/5] Checking NovaBrowser HTML/JavaScript surface behavior...
node "crates\nova-desktop\ui\tests\servo-native-surface.cjs"
if errorlevel 1 exit /b 1

echo [3/5] Typechecking the original NovaBrowser with Servo IPC enabled...
cargo check -p nova-desktop --features servo-embed
if errorlevel 1 exit /b 1

echo [4/5] Typechecking the independent native Servo renderer...
cargo check --manifest-path "engines\servo-native\Cargo.toml"
if errorlevel 1 exit /b 1

echo [5/5] Running standalone Servo renderer protocol unit tests...
cargo test --manifest-path "engines\servo-native\Cargo.toml" --bin nova-servo-native
if errorlevel 1 exit /b 1

echo All local checks passed. Runtime GPU and website validation still require manual testing.
exit /b 0
