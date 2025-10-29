import { create } from 'zustand';
import { temporal } from 'zundo';
import { trimAudioBlob, getFFmpegMode } from '../lib/ffmpeg';

// --- SHARED TYPES ---
export interface QPCWord {
  id: number;
  location: string; // "surah:ayah:word"
  surah: number;
  ayah: number;
  word: number;
  text: string; // HTML with tajweed tags
  rules: string[];
}

export interface WordInstance {
  id: number;
  qpc_location: string;
  text: string;
  full_text?: string;
  words?: QPCWord[]; // Array of words from backend
}

export interface TierRegion {
  id: string; // Use robust IDs like UUIDs or nanoid
  startTime: number;
  endTime: number;
  labelText: string;
  key?: string; // Stable identity key (e.g., "word___0")
}

export type AnnotationTiers = {
  ayahs: TierRegion[];
  words: TierRegion[];
};

export type ActiveTier = keyof AnnotationTiers;

interface AnnotationState {
  // --- STATE ---
  activeInstance: WordInstance | null;
  activeAudio: { blob: Blob; url: string; duration: number; mime?: string } | null;
  activeTiers: AnnotationTiers;
  activeTier: ActiveTier;
  trimRegion: { start: number; end: number } | null;
  isLoading: boolean;
  ffmpegError?: string;
  selectedRegion: { tier: ActiveTier; regionId: string } | null;
  wordToAnnotate: string | null;

  // --- ACTIONS ---
  setActiveInstance: (instance: WordInstance) => void;
  handleRecordingComplete: (blob: Blob, duration: number) => void;
  applyTrim: () => Promise<boolean>;
  setTrimRegion: (updates: { start: number; end: number }) => void;
  setActiveTier: (tier: ActiveTier) => void;
  addRegion: (tier: ActiveTier, regionData: Omit<TierRegion, 'id'>) => void;
  setSelectedRegion: (selection: { tier: ActiveTier; regionId: string } | null) => void;
  updateRegion: (tier: ActiveTier, regionId: string, updates: Partial<TierRegion>) => void;
  deleteRegion: (tier: ActiveTier, regionId: string) => void;
  setWordToAnnotate: (word: string | null) => void;
  resetActiveState: () => void;
}

const initialTiers: AnnotationTiers = { ayahs: [], words: [] };

export const useAnnotationStore = create(
  temporal<AnnotationState>((set, get) => ({
    // --- INITIAL STATE ---
    activeInstance: null,
    activeAudio: null,
    activeTiers: initialTiers,
    activeTier: 'ayahs',
    trimRegion: null,
    isLoading: false,
    ffmpegError: undefined,
    selectedRegion: null,
    wordToAnnotate: null,

    // --- ACTIONS ---
    setActiveInstance: (instance) => {
      get().resetActiveState();
      set({ activeInstance: instance });
    },

    handleRecordingComplete: (blob, duration) => {
      set({
        activeAudio: { blob, url: URL.createObjectURL(blob), duration, mime: blob.type },
        trimRegion: { start: 0, end: duration },
      });
    },

    applyTrim: async () => {
      const { activeAudio, trimRegion } = get();
      if (!activeAudio || !trimRegion) return false;

      set({ isLoading: true, ffmpegError: undefined });
      try {
        // Ensure we have a Blob (if only URL exists, fetch it)
        let inputBlob = activeAudio.blob;
        if (!inputBlob) {
          const res = await fetch(activeAudio.url);
          inputBlob = await res.blob();
        }

        const { blob, mime } = await trimAudioBlob(
          inputBlob!,
          trimRegion.start,
          trimRegion.end,
          /* timeoutMs */ 45000,
          /* onProgress */ (_p) => {
            // (Optional) route to a progress bar
            // console.debug('ffmpeg progress', _p)
          }
        );

        // Create a new object URL and replace activeAudio
        const newUrl = URL.createObjectURL(blob);
        const newDuration = trimRegion.end - trimRegion.start;
        const oldUrl = activeAudio.url;

        set({
          activeAudio: { blob, url: newUrl, duration: newDuration, mime },
          activeTiers: initialTiers,
          selectedRegion: null,
        });

        // Clean up old URL
        if (oldUrl && oldUrl.startsWith('blob:')) URL.revokeObjectURL(oldUrl);

        console.info('[FFmpeg] Trim OK. mode=', getFFmpegMode());
        return true;
      } catch (e: any) {
        console.warn('FFmpeg trim failed:', e);
        set({ ffmpegError: e?.message || 'FFmpeg failed' });
        return false; // UI already proceeds with original audio
      } finally {
        set({ isLoading: false });
      }
    },

    setTrimRegion: (updates) => set({ trimRegion: updates }),

    setActiveTier: (tier) => set({ selectedRegion: null, activeTier: tier }),

    addRegion: (tier, regionData) => {
      const newRegion: TierRegion = { ...regionData, id: self.crypto.randomUUID() };
      set((state) => ({
        activeTiers: { ...state.activeTiers, [tier]: [...state.activeTiers[tier], newRegion] },
        wordToAnnotate: tier === 'words' ? null : state.wordToAnnotate,
      }));
    },

    setSelectedRegion: (selection) => set({ selectedRegion: selection }),

    updateRegion: (tier, regionId, updates) => {
      set((state) => ({
        activeTiers: {
          ...state.activeTiers,
          [tier]: state.activeTiers[tier].map((r) => r.id === regionId ? { ...r, ...updates } : r),
        },
      }));
    },

    deleteRegion: (tier, regionId) => {
      set((state) => ({
        activeTiers: {
          ...state.activeTiers,
          [tier]: state.activeTiers[tier].filter((r) => r.id !== regionId),
        },
        selectedRegion: state.selectedRegion?.regionId === regionId ? null : state.selectedRegion,
      }));
    },

    setWordToAnnotate: (word) => set({ wordToAnnotate: word, activeTier: 'words', selectedRegion: null }),

    resetActiveState: () => {
      const { activeAudio } = get();
      if (activeAudio) URL.revokeObjectURL(activeAudio.url);
      set({
        activeInstance: null,
        activeAudio: null,
        activeTiers: initialTiers,
        trimRegion: null,
        selectedRegion: null,
      });
    },
  }))
);
