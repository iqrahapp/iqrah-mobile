#!/usr/bin/env python3
import asyncio
import aiohttp
import argparse
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Dict, Any, Optional, List

from tqdm import tqdm

# --------------- Config defaults (can be overridden by CLI) ---------------

DEFAULT_BASE_URL = "https://audios.quranwbw.com/words"
DEFAULT_VERSION_PARAM = "2"   # QuranWBW uses ?version=2 in your example
DEFAULT_CONCURRENCY = 20
DEFAULT_FLUSH_EVERY = 100      # write mapping to disk every N completed words
DEFAULT_MAPPING_FILE = "word_audio_map.json"
DEFAULT_URL_CACHE_FILE = "audio_url_cache.json"
DEFAULT_AUDIO_DIR = "audio"
DEFAULT_UTHMANI_FILE = "../text/wbw/uthmani-simple.json"


# ------------------------------ Helpers -----------------------------------

def build_audio_url(
    surah: str,
    ayah: str,
    word: str,
    base_url: str,
    version_param: Optional[str],
) -> str:
    """
    QuranWBW word audio URL format (based on your examples):

        https://audios.quranwbw.com/words/{surah-id}/{surah-id:03}_{ayah-id:03}_{word-id:03}.mp3[?version=2]
    """
    s = int(surah)
    a = int(ayah)
    w = int(word)

    path_part = f"{s:03d}_{a:03d}_{w:03d}.mp3"
    url = f"{base_url}/{s}/{path_part}"
    if version_param is not None:
        url += f"?version={version_param}"
    return url


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def atomic_write_json(path: Path, obj: Any) -> None:
    tmp_path = path.with_suffix(path.suffix + ".tmp")
    with tmp_path.open("w", encoding="utf-8") as f:
        json.dump(obj, f, ensure_ascii=False, indent=2)
    tmp_path.replace(path)


def is_ayah_number_word(entry: Dict[str, Any]) -> bool:
    """Return True if this word entry is just the ayah number (which has no audio)."""
    text = str(entry.get("text", "")).strip()
    if not text:
        return False
    digits = "0123456789٠١٢٣٤٥٦٧٨٩"
    return all(ch in digits for ch in text)


def mp3_bytes_to_pcm_s16le(
    mp3_data: bytes,
    sample_rate: int = 16000,
    channels: int = 1,
) -> bytes:
    """
    Convert MP3 bytes to raw PCM (s16le) for hashing.

    We normalize everything to mono 16kHz so that acoustically identical audio
    yields identical PCM bytes, even if the original MP3 encoding differs.
    """
    process = subprocess.Popen(
        [
            "ffmpeg",
            "-loglevel", "error",
            "-y",
            "-i", "pipe:0",
            "-f", "s16le",
            "-acodec", "pcm_s16le",
            "-ac", str(channels),
            "-ar", str(sample_rate),
            "pipe:1",
        ],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    out, err = process.communicate(mp3_data)
    if process.returncode != 0:
        raise RuntimeError(f"ffmpeg PCM conversion failed: {err.decode('utf-8', 'ignore')}")
    return out


def mp3_bytes_to_ogg_opus(mp3_data: bytes, bitrate: str = "24k") -> bytes:
    """
    Convert MP3 bytes to Ogg Opus bytes using ffmpeg.

    Requires `ffmpeg` binary available in PATH and built with libopus support.
    """
    process = subprocess.Popen(
        [
            "ffmpeg",
            "-loglevel", "error",
            "-y",
            "-i", "pipe:0",
            "-f", "ogg",
            "-c:a", "libopus",
            "-b:a", bitrate,
            "pipe:1",
        ],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    out, err = process.communicate(mp3_data)
    if process.returncode != 0:
        raise RuntimeError(f"ffmpeg opus conversion failed: {err.decode('utf-8', 'ignore')}")
    return out


# --------------------------- Async download logic --------------------------

async def fetch_audio(
    session: aiohttp.ClientSession,
    url: str,
    location: str,
) -> Optional[bytes]:
    """Download one audio file, return bytes or None on error."""
    try:
        async with session.get(url) as resp:
            if resp.status == 200:
                return await resp.read()
            else:
                tqdm.write(f"[WARN] {location}: HTTP {resp.status} for {url}")
                return None
    except Exception as e:
        tqdm.write(f"[ERROR] {location}: {e} ({url})")
        return None


async def worker(
    name: int,
    queue: "asyncio.Queue[Optional[tuple[str, Dict[str, Any]]]]",
    session: aiohttp.ClientSession,
    base_url: str,
    version_param: Optional[str],
    audio_dir: Path,
    mapping: Dict[str, str],
    url_cache: Dict[str, str],
    text_audio_map: Dict[str, List[str]],
    progress: "tqdm",
    flush_every: int,
    flush_state: Dict[str, int],
    mapping_path: Path,
    url_cache_path: Path,
    mapping_lock: asyncio.Lock,
):
    """
    Worker that consumes word entries from queue, downloads + stores audio.

    - Uses a URL→hash cache so we don't re-download the same audio URL across runs.
    - Maintains a text→audio map for later patching of missing words.
    - Stores audio as Ogg Opus (.ogg) on disk, using PCM-hash as filename.
    """
    while True:
        item = await queue.get()
        if item is None:  # sentinel
            queue.task_done()
            break

        location, entry = item

        try:
            # Skip if already processed
            if location in mapping:
                progress.update(1)
                queue.task_done()
                continue

            # Derive IDs from location key "s:a:w"
            surah, ayah, word = location.split(":")

            url = build_audio_url(
                surah=surah,
                ayah=ayah,
                word=word,
                base_url=base_url,
                version_param=version_param,
            )

            # 1) Check URL cache first (no HTTP)
            cached_hash = url_cache.get(url)
            if cached_hash is not None:
                audio_path = audio_dir / f"{cached_hash}.ogg"
                if audio_path.exists():
                    rel_path = audio_path.as_posix()
                    mapping[location] = rel_path

                    text = str(entry.get("text", ""))
                    async with mapping_lock:
                        if text:
                            paths = text_audio_map.setdefault(text, [])
                            if rel_path not in paths:
                                paths.append(rel_path)

                        flush_state["counter"] += 1
                        if flush_state["counter"] >= flush_every:
                            atomic_write_json(mapping_path, mapping)
                            atomic_write_json(url_cache_path, url_cache)
                            flush_state["counter"] = 0

                    progress.update(1)
                    queue.task_done()
                    continue
                # If file is missing, fall through to re-download

            # 2) Download from remote (MP3 bytes)
            data = await fetch_audio(session, url, location)
            if data is None:
                # Skip, will retry on next run
                progress.update(1)
                queue.task_done()
                continue

            # Decode to normalized PCM and hash that (true audio-level dedup)
            try:
                pcm_bytes = mp3_bytes_to_pcm_s16le(data)
            except Exception as e:
                tqdm.write(f"[ERROR] {location}: failed to convert to PCM: {e}")
                progress.update(1)
                queue.task_done()
                continue

            audio_hash = sha256_bytes(pcm_bytes)

            # Convert to Ogg Opus for storage/playback
            try:
                ogg_bytes = mp3_bytes_to_ogg_opus(data)
            except Exception as e:
                tqdm.write(f"[ERROR] {location}: failed to convert to Opus: {e}")
                progress.update(1)
                queue.task_done()
                continue

            audio_path = audio_dir / f"{audio_hash}.ogg"

            # Save audio if not already saved (deduplication by hash)
            if not audio_path.exists():
                audio_path.write_bytes(ogg_bytes)

            # Update mapping, URL cache, and text→audio map
            rel_path = audio_path.as_posix()
            mapping[location] = rel_path
            url_cache[url] = audio_hash

            text = str(entry.get("text", ""))
            async with mapping_lock:
                if text:
                    paths = text_audio_map.setdefault(text, [])
                    if rel_path not in paths:
                        paths.append(rel_path)

                flush_state["counter"] += 1
                if flush_state["counter"] >= flush_every:
                    atomic_write_json(mapping_path, mapping)
                    atomic_write_json(url_cache_path, url_cache)
                    flush_state["counter"] = 0

            progress.update(1)

        finally:
            queue.task_done()


async def process_all_words(
    uthmani_path: Path,
    mapping_path: Path,
    audio_dir: Path,
    base_url: str,
    version_param: Optional[str],
    concurrency: int,
    flush_every: int,
):
    # Load uthmani-simple.json
    with uthmani_path.open("r", encoding="utf-8") as f:
        uthmani_data: Dict[str, Dict[str, Any]] = json.load(f)

    # Load existing mapping if it exists (for crash recovery)
    if mapping_path.exists():
        with mapping_path.open("r", encoding="utf-8") as f:
            mapping: Dict[str, str] = json.load(f)
    else:
        mapping = {}

    # Load URL→hash cache (to avoid re-downloading same URL across runs)
    url_cache_path = mapping_path.parent / DEFAULT_URL_CACHE_FILE
    if url_cache_path.exists():
        with url_cache_path.open("r", encoding="utf-8") as f:
            url_cache: Dict[str, str] = json.load(f)
    else:
        url_cache = {}

    # Filter out ayah-number tokens (last word in ayah like "١", "٢", etc.)
    all_items = list(uthmani_data.items())
    normal_items: List[tuple[str, Dict[str, Any]]] = [
        (loc, e) for loc, e in all_items if not is_ayah_number_word(e)
    ]

    # Initialize text→audio map from existing mapping for normal items
    text_audio_map: Dict[str, List[str]] = {}
    for loc, entry in normal_items:
        if loc in mapping:
            text = str(entry.get("text", ""))
            if not text:
                continue
            rel_path = mapping[loc]
            paths = text_audio_map.setdefault(text, [])
            if rel_path not in paths:
                paths.append(rel_path)

    # Ensure audio dir
    audio_dir.mkdir(parents=True, exist_ok=True)

    # Only count non-number words for progress
    total_words = len(normal_items)
    already_done = sum(1 for loc, _ in normal_items if loc in mapping)
    to_do = total_words - already_done

    print(f"Total (non-number) words: {total_words}")
    print(f"Already mapped: {already_done}")
    print(f"Remaining: {to_do}")
    print()

    # Build queue of work items (only missing locations, excluding ayah numbers)
    queue: asyncio.Queue = asyncio.Queue()
    for location, entry in normal_items:
        if location not in mapping:
            queue.put_nowait((location, entry))

    # Prepare progress bar
    progress = tqdm(total=total_words, initial=already_done, desc="Processing words")

    # Shared flush state
    flush_state = {"counter": 0}
    mapping_lock = asyncio.Lock()

    timeout = aiohttp.ClientTimeout(total=None, sock_connect=30, sock_read=60)

    async with aiohttp.ClientSession(timeout=timeout) as session:
        # Start workers
        workers = [
            asyncio.create_task(
                worker(
                    name=i,
                    queue=queue,
                    session=session,
                    base_url=base_url,
                    version_param=version_param,
                    audio_dir=audio_dir,
                    mapping=mapping,
                    url_cache=url_cache,
                    text_audio_map=text_audio_map,
                    progress=progress,
                    flush_every=flush_every,
                    flush_state=flush_state,
                    mapping_path=mapping_path,
                    url_cache_path=url_cache_path,
                    mapping_lock=mapping_lock,
                )
            )
            for i in range(concurrency)
        ]

        # Add sentinel items to stop workers
        for _ in range(concurrency):
            queue.put_nowait(None)

        # Wait for all work to finish
        await queue.join()

        # Cancel workers
        for w in workers:
            w.cancel()
        # Ensure cancellation is processed
        await asyncio.gather(*workers, return_exceptions=True)

    # After downloads: patch missing words using exact text matches
    patched = 0
    for loc, entry in normal_items:
        if loc in mapping:
            continue
        text = str(entry.get("text", ""))
        if not text:
            continue
        paths = text_audio_map.get(text)
        if paths:
            mapping[loc] = paths[0]
            patched += 1

    # Final flush
    atomic_write_json(mapping_path, mapping)
    atomic_write_json(url_cache_path, url_cache)
    progress.close()

    print("Done.")
    print(f"Mapping saved to: {mapping_path}")
    print(f"URL cache saved to: {url_cache_path}")
    print(f"Unique audio files stored in: {audio_dir}")
    print(f"Patched (text-matched) words: {patched}")


# ---------------------------------- CLI -----------------------------------

def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description="Download QuranWBW word audio, deduplicate by PCM hash, and map locations to unique Ogg Opus audio files.",
    )
    p.add_argument(
        "--uthmani",
        type=str,
        default=DEFAULT_UTHMANI_FILE,
        help="Path to uthmani-simple.json (default: uthmani-simple.json)",
    )
    p.add_argument(
        "--mapping",
        type=str,
        default=DEFAULT_MAPPING_FILE,
        help="Output mapping JSON file (default: word_audio_map.json)",
    )
    p.add_argument(
        "--audio-dir",
        type=str,
        default=DEFAULT_AUDIO_DIR,
        help="Directory to store audio files (default: audio)",
    )
    p.add_argument(
        "--base-url",
        type=str,
        default=DEFAULT_BASE_URL,
        help=f"Base URL for audio (default: {DEFAULT_BASE_URL})",
    )
    p.add_argument(
        "--version",
        type=str,
        default=DEFAULT_VERSION_PARAM,
        help=f"Version query param, e.g. 2 → ?version=2 (default: {DEFAULT_VERSION_PARAM}). Use '' to disable.",
    )
    p.add_argument(
        "--concurrency",
        type=int,
        default=DEFAULT_CONCURRENCY,
        help=f"Number of concurrent downloads (default: {DEFAULT_CONCURRENCY})",
    )
    p.add_argument(
        "--flush-every",
        type=int,
        default=DEFAULT_FLUSH_EVERY,
        help=f"Flush mapping to disk every N processed words (default: {DEFAULT_FLUSH_EVERY})",
    )
    return p.parse_args()


def main():
    args = parse_args()

    uthmani_path = Path(args.uthmani)
    mapping_path = Path(args.mapping)
    audio_dir = Path(args.audio_dir)

    if not uthmani_path.exists():
        raise SystemExit(f"uthmani file not found: {uthmani_path}")

    version_param = args.version if args.version != "" else None

    asyncio.run(
        process_all_words(
            uthmani_path=uthmani_path,
            mapping_path=mapping_path,
            audio_dir=audio_dir,
            base_url=args.base_url,
            version_param=version_param,
            concurrency=args.concurrency,
            flush_every=args.flush_every,
        )
    )


if __name__ == "__main__":
    main()

