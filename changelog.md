# 🎵 Tenuto Compiler (tenutoc) v2.1.1 Release Notes

**Release Date:** February 23, 2026  
**Version:** 2.1.1  
**Status:** Stable / Feature Release  

## Executive Summary
Version 2.1.1 introduces **Phase IV: The Interchange Layer**, officially bringing visual sheet music generation to the Tenuto ecosystem. 

Building upon the deterministic LL(1) parser introduced in v2.1.0, this release adds the **Rebarring Engine** and the **MusicXML 4.0 Exporter**. `tenutoc` can now mathematically slice absolute-time IR events into discrete visual measures, allowing developers to compile raw Tenuto code directly into `.musicxml` files that render flawlessly in industry-standard engraving software like MuseScore, Dorico, and Sibelius.

---

## ✨ Key Features

### 1. The Rebarring Engine (`tenutoc::rebar`)
Translates the continuous 1-dimensional audio timeline into a 2D visual grid.
*   **The Guillotine Algorithm:** Automatically detects when a note crosses a measure boundary (barline) and mathematically slices it into two distinct events connected by a visual tie (`tie_start`, `tie_stop`).
*   **The Void Filler:** Automatically pads empty measure space with mathematically precise rests to ensure every measure perfectly satisfies its active Time Signature.

### 2. MusicXML 4.0 Exporter (`tenutoc::xml`)
A high-performance, zero-DOM string builder that maps the Visual IR into standard XML tags.
*   **Tuplet Resolution:** Accurately calculates and injects `<time-modification>` and bracket `<tuplet>` tags for complex irrational rhythms.
*   **Visual Type Inference:** Reverse-engineers duration ticks to output proper visual types (`quarter`, `eighth`, `16th`) and augmentation dots (`<dot/>`).
*   **Polyphony Management:** Automatically detects multi-voice brackets (`<[ ]>`) and injects `<backup>` and `<forward>` tags to sync independent time streams within a single measure.
*   **Chord Grouping:** Identifies simultaneous note attacks and injects the `<chord/>` tag to bundle stems correctly.

### 3. Tuplet State Tracking (`tenutoc::ir`)
*   The Inference Engine now tracks active Tuplet ratios natively within the `Cursor`, attaching a `TupletState` to individual `AtomicEvent`s. This allows the backend to know exactly when to draw open and closing tuplet brackets.

---

## 🛠️ Usage Updates

The CLI now dynamically routes output based on the provided file extension.

```bash
# Compiles to Audio (MIDI)
tenutoc --input score.ten --output score.mid

# Compiles to Sheet Music (MusicXML 4.0)
tenutoc --input score.ten --output score.musicxml