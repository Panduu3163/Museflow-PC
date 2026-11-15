# Compact layout and keyboard controls — 0.7.1 preview

Settings uses the available content height as one scrollable pane. Audio output, desktop/tray status and keyboard help stay in that pane rather than competing with it for fixed vertical space. The minimum supported content size remains 800 × 600 logical pixels. Startup uses Slint’s preferred size, allowing the backend to apply its scale factor. The native journey converts logical test sizes through Slint’s active scale factor and asserts the resulting physical dimensions before capturing. Scroll Settings to reach its remaining options on a small window.

| Input | Action |
| --- | --- |
| Ctrl+1 | Home |
| Ctrl+2 | Search |
| Ctrl+3 | Library |
| Ctrl+4 | Liked songs |
| Ctrl+5 | Settings |
| Tab / Shift+Tab | Move between focusable controls and visible track rows |
| Enter / Space | Activate the focused button or track |
| Escape | Close a side panel after its focused child declines the key |

Section shortcuts do not also accept Alt/Shift/Meta modifiers. Repeating a held activation key does not repeatedly activate a custom button/track. Typing into an editor and native widget keyboard behavior remain separate. A control can handle Escape first, for example when dismissing its own popup.

Track rows have visible keyboard focus outlines. Play/View labels include the song and artist; playlist editors and collection selection have explicit names. Button text elides visually while its full accessible name is retained. Shuffle exposes its on/off state in its name. Liking a song transfers focus to the stable application scope before the row model refreshes, keeping section shortcuts usable afterward. Arrow-key track selection and a roving list focus model are not part of this change.

## Isolated native check

With the documented environment and a fixture-only library:

```powershell
& "$env:CARGO_TARGET_DIR\release\museflow-pc.exe" --data-dir "G:\path\to\isolated-fixture-library" --null-audio --ui-smoke-test "E:\path\to\evidence"
```

The mode requires an explicit data directory; never point it at your personal library. It changes the isolated library’s theme and navigates through eight small/wide layout captures, then exercises native keyboard events for section navigation, modifier rejection, Escape, Return activation and held-key handling. A watchdog rejects stalled runs. Captures still need visual inspection; a PASS marker alone does not certify visual quality.

`SLINT_SCALE_FACTOR=1.5` or `2` can be set only in the child process environment for renderer-density coverage. This does not change Windows display settings or replace real OS-DPI/multiple-monitor testing. Full screen-reader acceptance remains pending: the automation accessibility tree exposes the new names, but its focused-element report did not reliably follow custom-control focus during the manual check. Keyboard behavior was checked through visible focus and resulting playback/state changes.
