# Compact layout and keyboard controls — 0.7.2 preview

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
| Up / Down | Move the library cursor one track, scrolling when needed |
| Home / End | Move the library cursor to the first/last result |
| Page Up / Page Down | Move the library cursor by roughly one visible page |
| Escape | Close a side panel after its focused child declines the key |

Section shortcuts do not also accept Alt/Shift/Meta modifiers. Repeating a held activation key does not repeatedly activate a custom button/track. Typing into an editor and native widget keyboard behavior remain separate. A control can handle Escape first, for example when dismissing its own popup.

Track rows have visible keyboard focus outlines. Play/View labels include the song and artist; playlist editors and collection selection have explicit names. Button text elides visually while its full accessible name is retained. Shuffle exposes its on/off state in its name. Liking a song transfers focus to the stable application scope before the row model refreshes, keeping section shortcuts usable afterward. In 0.7.2, unmodified arrow/Home/End/Page keys move the library cursor without changing the playing track. Enter/Space activates its current recording ID. Tab still follows visible controls; this is not a multi-selection model. Search editors retain their own navigation keys. Filtering/model replacement resets the transient cursor, while playback selection is preserved.

## Isolated native check

With the documented environment and a fixture-only library:

```powershell
& "$env:CARGO_TARGET_DIR\release\museflow-pc.exe" --data-dir "G:\path\to\isolated-fixture-library" --null-audio --ui-smoke-test "E:\path\to\evidence"
```

The mode requires an explicit data directory; never point it at your personal library. It changes the isolated library’s theme and navigates through eight small/wide layout captures, then exercises native keyboard events for section navigation, modifier rejection, Escape, Return activation and held-key handling. A watchdog rejects stalled runs. Captures still need visual inspection; a PASS marker alone does not certify visual quality.

`SLINT_SCALE_FACTOR=1.5` or `2` can be set only in the child process environment for renderer-density coverage. This does not change Windows display settings or replace real OS-DPI/multiple-monitor testing. Full screen-reader acceptance remains pending: the automation accessibility tree exposes the new names, but its focused-element report did not reliably follow custom-control focus during the manual check. Keyboard behavior was checked through visible focus and resulting playback/state changes.

## Large-library keyboard journey

```powershell
& "$env:CARGO_TARGET_DIR\release\museflow-pc.exe" --list-keyboard-smoke-test "E:\path\to\evidence"
```

This opt-in mode runs only against 10,000 synthetic demo records. It does not open the personal SQLite library, initialize audio or resolve streams. Add `--list-keyboard-light` to exercise the light theme. It verifies Tab entry, Up/Down, Home/End, Page Up/Down, held-arrow input, filtered recording IDs, empty results and a minimum-size Home window. Rendered top/bottom focus-border checks catch a correct cursor ID whose row is nevertheless clipped. Renderer density can be overridden in the child process as described above.

The list remains virtualized. One stable focus scope holds keyboard focus while a distant row is materialized. A coalesced deferred correction runs after row focus changes, then stops; it prevents construction-time focus scrolling from leaving a row partially clipped. No new idle polling loop, worker thread, dependency or database setting was added. Short Home windows use a smaller hero and omit its secondary decorative text to leave room for complete rows. Full screen-reader and real OS-DPI/multi-monitor acceptance remain separate.
