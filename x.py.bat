@echo off

where /Q py.exe
IF %ERRORLEVEL% NEQ 0 (
  python x.py %*
) ELSE (
  py -3 x.py %*
)
