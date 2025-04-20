@echo off

mkdir deep\deep\deep\deep >nul 2>nul

start cmd /k "echo locked > deep/locked.txt && echo File locked. && (>&2 pause) >> deep/locked.txt"
start cmd /k "echo locked > deep/deep/locked.txt && echo File locked. && (>&2 pause) >> deep/deep/locked.txt"
start cmd /k "echo locked > deep/deep/deep/locked.txt && echo File locked. && (>&2 pause) >> deep/deep/deep/locked.txt"
start cmd /k "echo locked > deep/deep/deep/deep/locked.txt && echo File locked. && (>&2 pause) >> deep/deep/deep/deep/locked.txt"
