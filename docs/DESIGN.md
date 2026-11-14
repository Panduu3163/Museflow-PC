# Visual direction

Aim for an attractive, art-led desktop player with the clarity of a mature music application and MuseFlow's own identity. Spotify is a quality/layout reference, not branding/assets to copy.

- Left sidebar: Home, Search and Library, plus user playlists. Center: clear title/search, open track rows and album cards. Bottom: persistent artwork/title/artist, transport, seek and volume. Right: optional Queue or Lyrics, never both competing for the same space.
- Use consistent text/surface/accent roles, quiet content surfaces, readable selected/disabled states and a restrained accent derived from artwork. Provide complete light and dark themes; do not hardcode white text on arbitrary artwork.
- Use the system UI font by default with fallback for Bengali and other scripts. Doto is optional display typography, with its OFL notice preserved; do not use it for dense song lists.
- Prefer short interaction-driven transitions. Respect reduced motion and avoid decorative redraw loops while idle. Do not block actions until animations finish.
- Virtualize large lists; scale images to their actual display size. Reflow the right panel/content at smaller window widths rather than compressing transport controls or hiding essential actions.
- Design keyboard focus, screen-reader names and scaling together with the visual components. Include loading, empty, partial, offline and recoverable-error states.

The copied screenshots are historical Android design references, not current acceptance evidence or desktop mockups. The user reported build 66's liquid-glass implementation buggy. Do not port its navigation lens, moving-pill gestures or glass widgets. Elaborate glass is deferred until basic usability, stability and measured performance pass.
