# Fixture Contract 025: QuickCopyAssistant v0.2

Status: accepted
Date: 2026-08-05

Use three copied regular ALS fixtures. Route one through cold Open With, two
through ordered warm launch and one duplicate through Finder drop. The adapter
must produce one queue item per distinct accepted path in event order. Invalid
URLs, non-ALS files, directories and links fail without starting module 024.

Packaged smoke evidence must include intake before Play, intake during work,
ready state, source byte comparison and a successful Ableton-open check for at
least one generated Project.
