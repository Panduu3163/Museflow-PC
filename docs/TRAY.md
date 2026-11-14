# Tray and portable packaging — 0.5.0

The tray icon appears when the library and Windows notification area are ready. Left-click it to show MuseFlow; right-click for **Show MuseFlow**, **Play/Pause**, **Previous**, **Next** and **Exit MuseFlow**. Windows may put the icon under its notification-area overflow arrow.

Closing the title-bar window **exits by default**. Settings offers **Close window to tray** as a saved opt-in, plus explicit **Hide to tray** and **Exit MuseFlow** buttons. Minimize behaves normally. Show restores either a hidden or minimized player and requests a fresh frame.

Hiding keeps the native player window/handle alive, so playback and Windows media controls continue. Lyrics requests are suspended when hidden or minimized. Tray actions use the same playback owner and normal shutdown path as the app. Exit saves state, cancels work, releases Windows media controls and removes the tray icon.

Close-to-tray applies only while the tray is available. A failed creation or unavailable taskbar leaves Close as Exit. If an already-hidden player loses its tray availability, it restores the window. The tray dependency handles the Windows taskbar-recreated message; an actual Explorer restart is not part of the completed tests. Settings reports readiness/failure. No autostart or registry preferences were added.

Windows playback status now updates immediately after a core change. This avoids a race where a quick Pause→Play could be ignored while Windows still held the previous Playing status. Progress updates remain bounded and unchanged metadata is not republished.

## Local preview and portable copy

The development preview lives under `G:\Essential Softwares\MuseFlow-PC\Preview\0.5.0-tray`. Its marker continues using `G:\Essential Softwares\MuseFlow-PC\Data`, preserving the existing library. Source remains on E:.

The portable ZIP has one top-level application folder. Extract the whole folder to your SSD and run MuseFlow-PC.exe. The archive's relative `portable-data-dir.txt` selects an adjacent `data/` folder; this portable copy has its own library/settings. Keep that data folder when updating a portable installation. Archives do not contain or erase existing data folders.

`tools/package_windows.py` checks the actual Windows executable version against Cargo.toml, requires x64, verifies staged binary hashes, uses an explicit input allowlist, rewrites the marker for the portable copy and records content hashes. It checks archive CRC/content hashes and emits a ZIP SHA-256. It refuses an existing output path and does not install, sign or publish anything. Python 3.11+ with its standard library is sufficient.

Example from the source workspace:

```powershell
& 'G:\Essential Softwares\Python\bin\python.exe' .\tools\package_windows.py `
  --staging 'G:\Essential Softwares\MuseFlow-PC\Preview\0.5.0-tray' `
  --output 'G:\Essential Softwares\MuseFlow-PC\Distribution\MuseFlow-PC-0.5.0-windows-x64-portable.zip'
```

This is a local portable preview, not a signed installer or public redistribution package. Windows 10/clean-machine checks, physical media keys/accessories, complete accessibility/scaling and native/runtime license/source-fulfillment review remain pending. See STATUS.md for actual tests and measurements.

[tray-icon documentation](https://docs.rs/tray-icon/0.26.1/tray_icon/) describes the platform/event-loop requirements. Slint and tray menus use their respective pinned Muda versions; the tray has its own event handlers.
