@echo off
echo locked > locked.txt
echo File locked.
( >&2 pause ) >> locked.txt
