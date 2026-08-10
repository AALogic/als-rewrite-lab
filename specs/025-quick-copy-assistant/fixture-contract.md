# Fixture Contract 025: QuickCopyAssistant v0.8

Status: accepted
Date: 2026-08-10

Use three copied regular ALS fixtures. Route one through cold Open With, two
through ordered warm launch and one duplicate through Finder drop. The adapter
must produce one queue item per distinct accepted path in event order. Invalid
URLs, non-ALS files, directories and links fail without starting module 024.

Packaged smoke evidence must include intake before Play, intake during work,
ready state, source byte comparison and a successful Ableton-open check for at
least one generated Project.

The macOS overlay matrix uses copied regular ALS fixtures only. It must prove:

1. five cold Open With launches remain alive and preserve the requested ALS;
2. five consecutive Finder drops are accepted exactly once each;
3. the Courier remains on screen and accepts a drop after Safari activation;
4. the Courier remains on screen and accepts a drop after Safari enters a
   full-screen Space;
5. after terminal handoff and `Nowe zlecenie`, one new Finder drop creates a
   fresh one-item collection;
6. outbound native parcel dragging still completes or cancels according to its
   existing contract;
7. opening the full application restores Regular activation policy.

Source ALS and source audio hashes are recorded before and after any fixture
that executes module 024. Intake-only tests do not start copying.
