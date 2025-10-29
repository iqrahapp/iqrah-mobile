# Tajweed Annotation Tool - Frontend

React + TypeScript frontend with WaveSurfer.js for audio annotation.

## Quick Start

```bash
npm install
npm run dev
```

Frontend will be available at: http://localhost:5173/

## Features

- **Audio Upload & Waveform Visualization**: Interactive waveform with zoom
- **Region Annotation**: Click and drag to create annotation regions
- **Playback Controls**: Play, pause, stop with visual feedback
- **Region Editing**: Edit labels, confidence, notes
- **Recordings List**: View and filter all recordings
- **JSON Export**: Export annotations for ML training

## Tech Stack

- React 19 + TypeScript
- Vite (build tool)
- Material-UI (components)
- WaveSurfer.js 7 (waveform + regions)
- Axios (API client)

## Usage

1. **Create Recording**: Select rule, anti-pattern, upload audio
2. **Annotate**: Click-drag on waveform to create regions
3. **Edit Regions**: Click region or edit button to modify
4. **Export**: Go to Recordings tab and export JSON

## Configuration

Create `.env`:
```
VITE_API_URL=http://localhost:8000
```

## Project Structure

```
src/
├── api/client.ts          # API client
├── components/
│   └── WaveformPlayer.tsx # Waveform component
├── pages/
│   ├── AnnotationPage.tsx # Main UI
│   └── RecordingsListPage.tsx # List view
└── App.tsx                # Navigation
```

## Build

```bash
npm run build  # Output: dist/
```
