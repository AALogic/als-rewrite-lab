# Quick Copy Worker Asset Source

Status: generated source retained for reproducibility
Date: 2026-08-05

The original character source was generated with the built-in OpenAI ImageGen
tool. It is not derived from or bundled with the user's mood-reference image.
The production strips are in:

```text
apps/rescue-desktop/public/quick-worker/
```

The retained `quick-worker-master-alpha.png` contains a 5 by 7 source grid. The
green source background was removed with the Codex ImageGen chroma-key helper.
Production cells were cropped and resized with nearest-neighbor sampling into
fixed 96 by 128 frames.

The later `quick-worker-share-source.png` is a separately retained three-frame
source for the same worker holding one heavy parcel. Its production strip is
also chroma-keyed, nearest-neighbor scaled and kept inside the same 96 by 128
logical box. The first frame is packaged as the native AppKit drag image.

## Final Prompt Summary

Create a distinct, compact, full-body pixel-art maintenance worker for a small
desktop assistant. He is middle-aged, tired and grumpy rather than angry, with
a dark teal work cap and jacket, mustard accents, rust-red gloves and boots,
and heavy eyebrows. Keep identity, proportions, scale and pixel density stable
across an exact 5-column by 7-row action grid. Rows show waiting, checking,
ready, working, completion, incomplete shrug and unable/head-shake states. Use
crisp 32-bit-style hard pixels on a flat removable green background. No police
styling, badges, logos, text, tools, boxes, scenery, running, shadows, blur or
watermark.

## Production Strips

```text
appearing.png              3 frames
destination-required.png   2 frames
preparing.png              3 frames
ready.png                  2 frames
working.png                5 frames
complete.png               3 frames
incomplete.png             3 frames
unable.png                 3 frames
share-armed.png            3 frames
```

CSS owns playback through `steps()`. JavaScript selects only the stable product
state and never advances animation frames.

The native drag image is stored at:

```text
apps/rescue-desktop/src-tauri/assets/quick-worker-share-drag.png
```
