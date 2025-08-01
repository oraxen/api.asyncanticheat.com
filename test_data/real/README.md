# Real captured test data

This folder contains **real batches** captured from the live Paper server via the AsyncAnticheat plugin and stored by the API in the object store.

Each subfolder is one “recording session” (typically started via `/aacdev start ...`) copied from the API server into the repo workspace for local/offline analysis.

## Folder format

- `temoin_*` folders: “clean” baseline runs (cheats OFF for the full duration).
- `cheat_*` folders: runs with toggled ON/OFF segments inside the same label (driven by `/aacdev` prompts).

Each folder should contain:

- `*.ndjson.gz`: raw captured batches (meta line + packet records).
- `manifest.json`: extracted metadata (label, start/stop markers, session/server/player identifiers).


