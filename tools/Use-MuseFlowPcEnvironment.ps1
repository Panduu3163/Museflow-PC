# Process-scoped environment; does not change machine/user PATH or registry.
$env:RUSTUP_HOME = 'G:\Essential Softwares\Rust\Rustup'
$env:CARGO_HOME = 'G:\Essential Softwares\Rust\Cargo'
$env:CARGO_TARGET_DIR = 'G:\Essential Softwares\MuseFlow-PC\Build'
$env:TEMP = 'G:\Essential Softwares\Temp'
$env:TMP = $env:TEMP
$env:PYTHONPYCACHEPREFIX = 'G:\Essential Softwares\MuseFlow-PC\PythonCache'
$env:PATH = "$env:CARGO_HOME\bin;$env:PATH"
$taskCompilerRoot = 'G:\Essential Softwares\MuseFlow-PC\MinGW'
# GCC's built-in manifest path does not survive spaces. M: is an alias to G:, not storage.
$taskCompilerMapping = @(& subst) | Where-Object { $_ -match '^M:\\:' }
if ($taskCompilerMapping -and ($taskCompilerMapping -notmatch (([regex]::Escape($taskCompilerRoot)) + '$'))) {
    throw 'M: is already mapped elsewhere; choose a free compiler alias before building.'
}
if (-not $taskCompilerMapping) {
    if (Test-Path 'M:\') { throw 'M: is already in use.' }
    & subst M: $taskCompilerRoot
    if ($LASTEXITCODE -ne 0) { throw 'Could not create temporary compiler alias M:.' }
}
$env:PATH = "M:\mingw64\bin;$env:PATH"
$env:GCC_EXEC_PREFIX = 'M:/mingw64/lib/gcc/'
$env:CARGO_BUILD_JOBS = '4'
if (-not $env:MUSEFLOW_MPV_LIBRARY) { $env:MUSEFLOW_MPV_LIBRARY = 'G:\Essential Softwares\MuseFlow-PC\libmpv\20261111\libmpv-2.dll' }
if (-not $env:MUSEFLOW_DATA_DIR) { $env:MUSEFLOW_DATA_DIR = 'G:\Essential Softwares\MuseFlow-PC\Data' }
if (-not $env:MUSEFLOW_YTDLP) { $env:MUSEFLOW_YTDLP = 'G:\Essential Softwares\MuseFlow-PC\yt-dlp\2026.10.26\yt-dlp.exe' }
if (-not $env:MUSEFLOW_DENO) { $env:MUSEFLOW_DENO = 'G:\Essential Softwares\MuseFlow-PC\JS-Runtime\v2.9.8\deno.exe' }
