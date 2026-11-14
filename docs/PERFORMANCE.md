# Performance targets and measurement

These are initial engineering targets on the user's PC, not measured results or guarantees:

| Journey | Target |
| --- | --- |
| Cold launch to usable window | Within 2 seconds |
| Warm launch to usable window | Within 1 second |
| Steady idle | Below 120 MiB RAM |
| Steady audio playback | Below 220 MiB RAM |

Define a usable window as a rendered, responsive sidebar/content shell and playback controls; online data must not block it. Record process start -> first usable frame with a monotonic clock. Keep cold-after-reboot and warm repeat samples separate. Use an optimized build outside the debugger; record hardware, OS, power mode, build hash and dependency/runtime versions. Report the median, slow samples and conditions rather than one favorable run.

Record summed working set and private bytes for the complete process tree. Compare the targets against total working set; report private bytes alongside it. Measure after startup settles and after playback settles, including the audio engine. Report resolver/JavaScript helper peak RAM separately and also include those children in the full-journey peak. Report GPU memory separately; do not hide it in a low CPU-RAM claim.

Use the same fixture library (including a 10,000-track synthetic metadata set), artwork sizes, viewport and actions for comparisons. Record idle, local playback, streaming, search, fast skips, large-list scrolling and queue/lyrics transitions. Measure frame-time distribution, CPU use, repeated-search/skip memory growth and first-use stalls. Repeated idle samples should reveal leaked or needlessly persistent helper processes.

Startup must not wait for scanning, network checks, lyric lookup, full artwork decoding or a resolver. Keep caches bounded and high-frequency progress work separate from library rendering. Record actual measurements before declaring a target passed. No measurements have been performed during workspace preparation.

Shell and 0.2.0 local-player measurements are now available in STATUS.md and SHELL-CHECKPOINT.md. They do not establish cold startup, GPU memory or long-session playback acceptance.
