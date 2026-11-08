# Rule: Privacy-by-Design & Antivirus Safety

Isfahan monitors typing activity purely as a rhythmic cadence to nurture a virtual companion.

## Strict Constraints
- **Never read which key was pressed**: Neither Rust (`rdev::Event`) nor the browser dev fallback (`KeyboardEvent`) may inspect key codes, character strings, or modifiers.
- **No persistence of keystroke timestamps**: Only aggregate counters or happiness scores may be kept in memory or saved in local preferences.
- **No network permissions**: Never introduce network calls, analytics, crash reporters, or remote asset URLs (including Google Fonts CDN). Everything must be 100% local and offline.
