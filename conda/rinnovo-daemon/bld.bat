@echo on
setlocal enabledelayedexpansion

REM Build the rnb_agent binary in release mode.
cargo build --release -p rnb_agent
IF ERRORLEVEL 1 EXIT /B 1

REM Install into the Conda prefix on Windows under the public-facing name rnb_daemon.exe.
IF NOT EXIST "%PREFIX%\Library\bin" (
  mkdir "%PREFIX%\Library\bin"
)
copy /Y "target\release\rnb_agent.exe" "%PREFIX%\Library\bin\rnb_daemon.exe"
IF ERRORLEVEL 1 EXIT /B 1

endlocal
EXIT /B 0
