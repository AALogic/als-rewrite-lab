# Fixture Contract 026: External Provider Opener

Status: accepted
Date: 2026-08-05

A fake `BrowserOpenerPort` records calls without opening a browser. The trusted
`wetransfer_web` request records exactly one fixed HTTPS URL. Unknown provider
IDs fail before the port is called. Requests, results and errors contain no
native path, collection identity, payload identity or upload state.
