Launch process with arguments, new process can spawn as hidden or with own new window. Spawned processes are logged into log file.

LOG:       Create log file in %TEMP%\{exe-base-name}-{date-time-stamp}.log, older than 30 days logs will be pruned
Arguments: 1st argument {hide|normal} - hidden or normal window for new process
           2nd process to start
           3rd ... last are passed to new process where all %env-variables% are expanded
Examples:

spawn hide "%ProgramFiles%\Powershell\7\pwsh.exe" -nologo -noprofile -file "%USERPROFILE%\somescript.ps1" # Will run script in background without any console window
spawn normal "%ProgramFiles%\Powershell\7\pwsh.exe" -nologo # Will run shell with console window

Target system windows.
