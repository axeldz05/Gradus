# Gradus: Professional Music Notation & Rhythm Suite

**Gradus** is a native desktop application designed for musical notation, rhythm editing, and precision practice. Built with **Rust**, **GTK4**, and **Relm4**, it focuses on performance and a data-oriented approach to music theory and execution.

## Architecture
* **gradus-core:** Handles the audio engine (metronome/tuner), measure validation, and rhythm sequencing.
* **gradus-gui:** Interface using GTK4 and the Relm4 framework.

## Key Features
* **Dynamic Measure Editor:** Create custom rhythm patterns with support for various time signatures and note values (quarter, eighth, etc.).
* **Precision Metronome:** Using cpal to reproduce a low-latency metronome.
* **Visual Beat Tracking:** Real-time animation using `DrawingArea` for an intuitive rhythmic experience.
* **Verovio Integration:** Score rendering using the Rust bindings of the C++ Verovio engine.

---

## 🛠 Development & Usage

### Prerequisites
* Rust (latest stable)
* GTK4 development headers
* `pkg-config`

### How to run
```bash
# Run the GUI application
cargo run -p gradus-gui

# Run tests for the core logic
cargo test -p gradus-core
```

# To-Do
## Core Logic & Basic UI

    [x] Native Tuner using Pitch Detection.

    [x] Metronome UI.

    [x] Basic MusicalMeasure validation logic.

    [x] Implement a full Step Sequencer for complex rhythm patterns.

    [ ] Put all the tools together in the same window, with option to hide or show.

## Audio & Rendering

    [x] Stable Metronome audio sink using cpal.

    [x] Integration of Verovio for SVG score rendering using ABC notation.

    [ ] Saving and loading configuration of metronome and partiture.

## Other Features

    [ ] Exporting measures to standard music formats (MusicXML/MIDI).

    [ ] Dark/Light theme integration via libadwaita.

    [ ] A player to reproduce partiture thorugh the ABC parser.

## Further Testing
