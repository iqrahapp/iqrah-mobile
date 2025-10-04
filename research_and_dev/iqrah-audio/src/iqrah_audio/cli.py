"""
Command-Line Interface for Iqrah Audio
======================================

CLI tool for processing reference audio and analyzing recitations.
"""

import click
from pathlib import Path
from rich.console import Console
from rich.table import Table
from rich.progress import track
import json

from .reference import ReferenceProcessor
from .pitch import PitchExtractor
from .denoise import AudioDenoiser
from .dtw import DTWAligner
from .scorer import RecitationScorer

console = Console()


@click.group()
@click.version_option(version="0.1.0")
def main():
    """Iqrah Audio - Qur'an Recitation Analysis Tool"""
    pass


@main.command()
@click.argument("audio_path", type=click.Path(exists=True, path_type=Path))
@click.argument("output_path", type=click.Path(path_type=Path))
@click.option("--denoise/--no-denoise", default=True, help="Apply noise reduction")
@click.option("--pitch-method", type=click.Choice(["auto", "crepe", "yin"]), default="auto")
@click.option("--metadata", type=str, help="JSON metadata (e.g., '{\"ayah\": \"1:1\", \"qari\": \"husary\"}')")
def process_reference(audio_path, output_path, denoise, pitch_method, metadata):
    """
    Process reference qari audio to CBOR format.

    \b
    Example:
        iqrah-audio process-reference husary_001001.wav output/001001.cbor.zst \\
            --metadata '{"ayah": "1:1", "qari": "husary"}'
    """
    console.print(f"[bold blue]Processing reference audio:[/bold blue] {audio_path}")

    # Parse metadata
    meta_dict = json.loads(metadata) if metadata else {}

    # Process
    processor = ReferenceProcessor(
        sample_rate=22050,
        pitch_method=pitch_method,
        denoise=denoise
    )

    with console.status("[bold green]Extracting pitch contour..."):
        ref_data = processor.process_audio_file(audio_path, meta_dict)

    # Save
    processor.save_cbor(ref_data, output_path, compress=True)

    # Display info
    contour_info = ref_data["contour"]
    processing_info = ref_data["processing"]

    table = Table(title="Reference Processing Complete")
    table.add_column("Property", style="cyan")
    table.add_column("Value", style="magenta")

    table.add_row("Duration", f"{processing_info['duration']:.2f}s")
    table.add_row("Frames", str(processing_info['n_frames']))
    table.add_row("Sample Rate", f"{processing_info['sample_rate']} Hz")
    table.add_row("Pitch Method", processing_info['pitch_method'])
    table.add_row("Denoised", "Yes" if processing_info['denoised'] else "No")
    table.add_row("Output", str(output_path))

    console.print(table)


@main.command()
@click.argument("user_audio", type=click.Path(exists=True, path_type=Path))
@click.argument("reference_cbor", type=click.Path(exists=True, path_type=Path))
@click.option("--denoise/--no-denoise", default=True)
@click.option("--output-json", type=click.Path(path_type=Path), help="Save results to JSON")
@click.option("--pitch-method", type=click.Choice(["auto", "crepe", "yin"]), default="auto")
def analyze(user_audio, reference_cbor, denoise, output_json, pitch_method):
    """
    Analyze user recitation against reference.

    \b
    Example:
        iqrah-audio analyze my_recitation.wav output/001001.cbor.zst \\
            --output-json results.json
    """
    console.print("[bold blue]Analyzing recitation...[/bold blue]")

    # Load reference
    with console.status("[bold green]Loading reference..."):
        processor = ReferenceProcessor(pitch_method=pitch_method)
        ref_contour = processor.get_contour_from_cbor(reference_cbor)

    # Process user audio
    console.print(f"[bold green]Processing user audio:[/bold green] {user_audio}")

    import soundfile as sf
    user_audio_data, sr = sf.read(user_audio)

    if user_audio_data.ndim > 1:
        user_audio_data = user_audio_data.mean(axis=1)

    # Denoise if requested
    if denoise:
        denoiser = AudioDenoiser(sample_rate=22050)
        with console.status("[bold yellow]Denoising audio..."):
            user_audio_data = denoiser.denoise_adaptive(user_audio_data)

    # Extract pitch
    extractor = PitchExtractor(sample_rate=22050, method=pitch_method)
    with console.status("[bold yellow]Extracting pitch..."):
        user_contour = extractor.extract_stable_pitch(user_audio_data, sr=sr)

    # Align
    aligner = DTWAligner()
    with console.status("[bold yellow]Aligning with reference..."):
        alignment = aligner.align(user_contour.f0_cents, ref_contour.f0_cents)

    # Score
    scorer = RecitationScorer()
    with console.status("[bold yellow]Calculating scores..."):
        score = scorer.score(user_contour, ref_contour, alignment)

    # Display results
    table = Table(title="Recitation Analysis Results", show_header=True)
    table.add_column("Metric", style="cyan", width=25)
    table.add_column("Score", justify="right", style="magenta")

    # Color code overall score
    overall_color = "green" if score.overall_score >= 80 else "yellow" if score.overall_score >= 60 else "red"

    table.add_row("Overall Score", f"[{overall_color}]{score.overall_score:.1f}/100[/{overall_color}]")
    table.add_row("Alignment Score", f"{score.alignment_score:.1f}/100")
    table.add_row("On-Note %", f"{score.on_note_percent:.1f}%")
    table.add_row("Pitch Stability", f"{score.pitch_stability:.1f}/100")
    table.add_row("Tempo Score", f"{score.tempo_score:.1f}/100")
    table.add_row("Voiced Ratio", f"{score.voiced_ratio:.1%}")

    console.print(table)

    # Detailed metrics
    console.print("\n[bold]Detailed Metrics:[/bold]")
    for key, value in score.metrics.items():
        console.print(f"  {key}: {value:.2f}" if isinstance(value, float) else f"  {key}: {value}")

    # Save to JSON if requested
    if output_json:
        results = score.to_dict()
        results['user_audio'] = str(user_audio)
        results['reference'] = str(reference_cbor)

        output_json.write_text(json.dumps(results, indent=2))
        console.print(f"\n[bold green]Results saved to:[/bold green] {output_json}")


@main.command()
@click.argument("input_dir", type=click.Path(exists=True, path_type=Path))
@click.argument("output_dir", type=click.Path(path_type=Path))
@click.option("--pattern", default="*.wav", help="File pattern to match")
@click.option("--denoise/--no-denoise", default=True)
@click.option("--pitch-method", type=click.Choice(["auto", "crepe", "yin"]), default="auto")
def batch_process(input_dir, output_dir, pattern, denoise, pitch_method):
    """
    Batch process directory of reference audio files.

    \b
    Example:
        iqrah-audio batch-process qari_audio/ output/ --pattern "*.mp3"
    """
    console.print(f"[bold blue]Batch processing:[/bold blue] {input_dir}")

    # Find files
    files = list(input_dir.glob(pattern))

    if not files:
        console.print(f"[bold red]No files found matching pattern:[/bold red] {pattern}")
        return

    console.print(f"[bold green]Found {len(files)} files[/bold green]")

    # Process
    processor = ReferenceProcessor(
        sample_rate=22050,
        pitch_method=pitch_method,
        denoise=denoise
    )

    output_dir.mkdir(parents=True, exist_ok=True)

    for audio_path in track(files, description="Processing"):
        # Extract metadata from filename
        # Assume format: qari_surah_ayah.ext
        parts = audio_path.stem.split('_')
        metadata = {"file": audio_path.name}

        if len(parts) >= 3:
            metadata.update({
                "qari": parts[0],
                "surah": int(parts[1]) if parts[1].isdigit() else parts[1],
                "ayah": int(parts[2]) if parts[2].isdigit() else parts[2],
            })

        # Process
        ref_data = processor.process_audio_file(audio_path, metadata)

        # Save
        output_path = output_dir / f"{audio_path.stem}.cbor.zst"
        processor.save_cbor(ref_data, output_path, compress=True)

    console.print(f"[bold green]✓ Batch processing complete![/bold green]")
    console.print(f"[bold]Output directory:[/bold] {output_dir}")


@main.command()
@click.argument("cbor_path", type=click.Path(exists=True, path_type=Path))
def inspect(cbor_path):
    """
    Inspect CBOR reference file.

    \b
    Example:
        iqrah-audio inspect output/001001.cbor.zst
    """
    processor = ReferenceProcessor()
    ref_data = processor.load_cbor(cbor_path)

    contour = ref_data["contour"]
    metadata = ref_data.get("metadata", {})
    processing = ref_data.get("processing", {})

    # Display
    table = Table(title=f"Reference File: {cbor_path.name}")
    table.add_column("Property", style="cyan")
    table.add_column("Value", style="magenta")

    # Metadata
    for key, value in metadata.items():
        table.add_row(f"metadata.{key}", str(value))

    # Processing info
    for key, value in processing.items():
        table.add_row(f"processing.{key}", str(value))

    # Contour stats
    f0_hz = contour["f0_hz"]
    voiced = [f for f in f0_hz if f > 0]

    if voiced:
        table.add_row("contour.f0_min", f"{min(voiced):.1f} Hz")
        table.add_row("contour.f0_max", f"{max(voiced):.1f} Hz")
        table.add_row("contour.f0_mean", f"{sum(voiced)/len(voiced):.1f} Hz")

    console.print(table)


if __name__ == "__main__":
    main()
