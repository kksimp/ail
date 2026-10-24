@echo off
rem Lets a program (ail test, tools) start `ail` on Windows: the PowerShell wrapper has no .exe of its own.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0ail.ps1" %*
exit /b %ERRORLEVEL%
