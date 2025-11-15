// IndexedDB persistence layer for audio blobs
import Dexie, { type Table } from 'dexie';

export interface StoredRecording {
  id: string;
  blob: Blob;
  duration: number;
  createdAt: string;
}

class AnnotationDB extends Dexie {
  recordings!: Table<StoredRecording>;

  constructor() {
    super('TajweedAnnotations');
    this.version(1).stores({
      recordings: 'id, createdAt',
    });
  }
}

export const db = new AnnotationDB();

/**
 * Save audio blob to IndexedDB
 * @returns recording ID
 */
export async function saveRecording(blob: Blob, duration: number): Promise<string> {
  const id = crypto.randomUUID();
  await db.recordings.add({
    id,
    blob,
    duration,
    createdAt: new Date().toISOString(),
  });
  return id;
}

/**
 * Load audio blob from IndexedDB
 * @returns blob and object URL (caller must revoke URL when done)
 */
export async function loadRecording(id: string): Promise<{ blob: Blob; url: string; duration: number } | null> {
  const rec = await db.recordings.get(id);
  if (!rec) return null;

  return {
    blob: rec.blob,
    url: URL.createObjectURL(rec.blob),
    duration: rec.duration,
  };
}

/**
 * Delete recording from IndexedDB
 */
export async function deleteRecording(id: string): Promise<void> {
  await db.recordings.delete(id);
}

/**
 * List all recordings
 */
export async function listRecordings(): Promise<StoredRecording[]> {
  return db.recordings.orderBy('createdAt').reverse().toArray();
}

/**
 * Clean up old recordings (keep last N)
 */
export async function cleanupOldRecordings(keepCount: number = 10): Promise<number> {
  const all = await listRecordings();
  if (all.length <= keepCount) return 0;

  const toDelete = all.slice(keepCount);
  await Promise.all(toDelete.map(rec => deleteRecording(rec.id)));

  return toDelete.length;
}
