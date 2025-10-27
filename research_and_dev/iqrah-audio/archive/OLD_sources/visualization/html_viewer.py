"""
Interactive HTML Viewer - Combines all visualizations into a rich web interface.

Creates a comprehensive HTML report with:
- Overall score dashboard
- All component visualizations (rhythm, melody, duration, pronunciation)
- Top-3 issue identification
- Actionable feedback
- Export functionality
"""
from typing import Dict, Optional
from pathlib import Path
import json


HTML_TEMPLATE = """
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Iqrah Recitation Analysis - {surah}:{ayah}</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}

        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            padding: 20px;
            min-height: 100vh;
        }}

        .container {{
            max-width: 1400px;
            margin: 0 auto;
            background: white;
            border-radius: 16px;
            box-shadow: 0 20px 60px rgba(0,0,0,0.3);
            overflow: hidden;
        }}

        .header {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 40px;
            text-align: center;
        }}

        .header h1 {{
            font-size: 2.5em;
            margin-bottom: 10px;
            font-weight: 700;
        }}

        .header .ayah-info {{
            font-size: 1.2em;
            opacity: 0.95;
            margin-top: 10px;
        }}

        .dashboard {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            padding: 40px;
            background: #f8f9fa;
        }}

        .score-card {{
            background: white;
            border-radius: 12px;
            padding: 25px;
            box-shadow: 0 4px 6px rgba(0,0,0,0.1);
            transition: transform 0.2s, box-shadow 0.2s;
        }}

        .score-card:hover {{
            transform: translateY(-5px);
            box-shadow: 0 8px 15px rgba(0,0,0,0.15);
        }}

        .score-card .label {{
            font-size: 0.85em;
            color: #666;
            text-transform: uppercase;
            letter-spacing: 1px;
            margin-bottom: 10px;
        }}

        .score-card .score {{
            font-size: 3em;
            font-weight: 700;
            color: #667eea;
            line-height: 1;
        }}

        .score-card .score.overall {{
            color: #764ba2;
        }}

        .score-card .confidence {{
            font-size: 0.9em;
            color: #999;
            margin-top: 8px;
        }}

        .issues-section {{
            padding: 40px;
            background: white;
        }}

        .issues-section h2 {{
            font-size: 1.8em;
            margin-bottom: 25px;
            color: #333;
            border-bottom: 3px solid #667eea;
            padding-bottom: 10px;
        }}

        .issue-card {{
            background: #fff;
            border-left: 4px solid #667eea;
            padding: 20px;
            margin-bottom: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.08);
        }}

        .issue-card.critical {{
            border-left-color: #e74c3c;
            background: #ffebee;
        }}

        .issue-card.timing {{
            border-left-color: #f39c12;
            background: #fff3e0;
        }}

        .issue-card.style {{
            border-left-color: #3498db;
            background: #e3f2fd;
        }}

        .issue-card .issue-header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 10px;
        }}

        .issue-card .category {{
            font-weight: 700;
            text-transform: uppercase;
            font-size: 0.85em;
            letter-spacing: 1px;
        }}

        .issue-card .impact {{
            font-size: 1.2em;
            font-weight: 700;
            color: #e74c3c;
        }}

        .issue-card .message {{
            color: #555;
            line-height: 1.6;
            margin-top: 10px;
        }}

        .issue-card .feedback {{
            background: rgba(255,255,255,0.8);
            padding: 15px;
            margin-top: 15px;
            border-radius: 6px;
            font-size: 0.95em;
            color: #333;
            border: 1px solid rgba(0,0,0,0.1);
        }}

        .visualizations {{
            padding: 40px;
            background: #fafafa;
        }}

        .visualizations h2 {{
            font-size: 1.8em;
            margin-bottom: 25px;
            color: #333;
            border-bottom: 3px solid #667eea;
            padding-bottom: 10px;
        }}

        .viz-section {{
            background: white;
            border-radius: 12px;
            padding: 30px;
            margin-bottom: 30px;
            box-shadow: 0 4px 6px rgba(0,0,0,0.1);
        }}

        .viz-section h3 {{
            font-size: 1.4em;
            margin-bottom: 20px;
            color: #667eea;
        }}

        .viz-section img {{
            width: 100%;
            height: auto;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }}

        .footer {{
            background: #2c3e50;
            color: white;
            padding: 30px;
            text-align: center;
        }}

        .footer .export-buttons {{
            margin-bottom: 20px;
        }}

        .footer button {{
            background: #667eea;
            color: white;
            border: none;
            padding: 12px 30px;
            margin: 0 10px;
            border-radius: 6px;
            font-size: 1em;
            cursor: pointer;
            transition: background 0.3s;
        }}

        .footer button:hover {{
            background: #764ba2;
        }}

        .tabs {{
            display: flex;
            gap: 10px;
            margin-bottom: 20px;
            border-bottom: 2px solid #e0e0e0;
        }}

        .tab {{
            padding: 12px 24px;
            background: #f5f5f5;
            border: none;
            border-radius: 8px 8px 0 0;
            cursor: pointer;
            font-size: 1em;
            transition: all 0.3s;
        }}

        .tab.active {{
            background: #667eea;
            color: white;
        }}

        .tab-content {{
            display: none;
        }}

        .tab-content.active {{
            display: block;
        }}

        @media print {{
            body {{
                background: white;
                padding: 0;
            }}
            .export-buttons {{
                display: none;
            }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <!-- Header -->
        <div class="header">
            <h1>🎯 Iqrah Recitation Analysis</h1>
            <div class="ayah-info">Surah {surah}, Ayah {ayah}</div>
            <div class="ayah-info">{transliteration}</div>
        </div>

        <!-- Dashboard -->
        <div class="dashboard">
            <div class="score-card">
                <div class="label">Overall Score</div>
                <div class="score overall">{overall_score:.1f}</div>
                <div class="confidence">Confidence: {confidence:.0%}</div>
            </div>
            <div class="score-card">
                <div class="label">Rhythm</div>
                <div class="score">{rhythm_score:.1f}</div>
            </div>
            <div class="score-card">
                <div class="label">Melody</div>
                <div class="score">{melody_score:.1f}</div>
            </div>
            <div class="score-card">
                <div class="label">Duration</div>
                <div class="score">{duration_score:.1f}</div>
            </div>
            <div class="score-card">
                <div class="label">Pronunciation</div>
                <div class="score">{pronunciation_score:.1f}</div>
            </div>
        </div>

        <!-- Top Issues -->
        {issues_html}

        <!-- Visualizations -->
        <div class="visualizations">
            <h2>📊 Detailed Analysis</h2>

            <!-- Tabs -->
            <div class="tabs">
                <button class="tab active" onclick="showTab('rhythm')">Rhythm</button>
                <button class="tab" onclick="showTab('melody')">Melody</button>
                <button class="tab" onclick="showTab('duration')">Duration</button>
                <button class="tab" onclick="showTab('pronunciation')">Pronunciation</button>
            </div>

            <!-- Rhythm Viz -->
            <div id="rhythm-tab" class="tab-content active">
                <div class="viz-section">
                    <h3>⏱️ Rhythm: DTW Alignment Path</h3>
                    {rhythm_viz}
                </div>
            </div>

            <!-- Melody Viz -->
            <div id="melody-tab" class="tab-content">
                <div class="viz-section">
                    <h3>🎵 Melody: Pitch Contour Comparison</h3>
                    {melody_viz}
                </div>
            </div>

            <!-- Duration Viz -->
            <div id="duration-tab" class="tab-content">
                <div class="viz-section">
                    <h3>📏 Duration: Madd Elongations</h3>
                    {duration_viz}
                </div>
            </div>

            <!-- Pronunciation Viz -->
            <div id="pronunciation-tab" class="tab-content">
                <div class="viz-section">
                    <h3>🗣️ Pronunciation: Phoneme Assessment</h3>
                    {pronunciation_viz}
                </div>
            </div>
        </div>

        <!-- Footer -->
        <div class="footer">
            <div class="export-buttons">
                <button onclick="window.print()">📥 Export as PDF</button>
                <button onclick="downloadHTML()">💾 Save HTML</button>
            </div>
            <p>Generated with Iqrah Audio Analysis System | Phase-2 Implementation</p>
        </div>
    </div>

    <script>
        function showTab(tabName) {{
            // Hide all tabs
            const tabs = document.querySelectorAll('.tab-content');
            tabs.forEach(tab => tab.classList.remove('active'));

            // Remove active from all tab buttons
            const tabButtons = document.querySelectorAll('.tab');
            tabButtons.forEach(btn => btn.classList.remove('active'));

            // Show selected tab
            document.getElementById(tabName + '-tab').classList.add('active');

            // Highlight selected button
            event.target.classList.add('active');
        }}

        function downloadHTML() {{
            const html = document.documentElement.outerHTML;
            const blob = new Blob([html], {{ type: 'text/html' }});
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = 'iqrah_analysis_{surah}_{ayah}.html';
            a.click();
            URL.revokeObjectURL(url);
        }}
    </script>
</body>
</html>
"""


def create_interactive_viewer(
    comparison_result: Dict,
    surah: int,
    ayah: int,
    transliteration: str = "",
    output_path: Optional[str] = None,
    rhythm_viz_base64: Optional[str] = None,
    melody_viz_base64: Optional[str] = None,
    duration_viz_base64: Optional[str] = None,
    pronunciation_viz_base64: Optional[str] = None
) -> str:
    """
    Create interactive HTML viewer with all visualizations.

    Args:
        comparison_result: Result from compare_recitations()
        surah: Surah number
        ayah: Ayah number
        transliteration: Romanized text
        output_path: Optional path to save HTML file
        rhythm_viz_base64: Base64-encoded rhythm visualization
        melody_viz_base64: Base64-encoded melody visualization
        duration_viz_base64: Base64-encoded duration visualization
        pronunciation_viz_base64: Base64-encoded pronunciation visualization

    Returns:
        HTML string (also saves to file if output_path provided)
    """
    # Extract scores
    overall_score = comparison_result.get('overall', 0)
    confidence = comparison_result.get('confidence', 0)

    rhythm_score = comparison_result.get('rhythm', {}).get('score', 0)
    melody_score = comparison_result.get('melody', {}).get('score', 0)
    duration_score = comparison_result.get('durations', {}).get('overall', 0)
    pronunciation_score = comparison_result.get('pronunciation', {}).get('score', 0)

    # Generate issues HTML
    issues = comparison_result.get('top_issues', [])
    issues_html = generate_issues_html(issues)

    # Generate visualization HTML
    rhythm_viz = f'<img src="data:image/png;base64,{rhythm_viz_base64}" alt="Rhythm Analysis">' if rhythm_viz_base64 else '<p>No rhythm visualization available</p>'
    melody_viz = f'<img src="data:image/png;base64,{melody_viz_base64}" alt="Melody Analysis">' if melody_viz_base64 else '<p>No melody visualization available</p>'
    duration_viz = f'<img src="data:image/png;base64,{duration_viz_base64}" alt="Duration Analysis">' if duration_viz_base64 else '<p>No duration visualization available</p>'
    pronunciation_viz = f'<img src="data:image/png;base64,{pronunciation_viz_base64}" alt="Pronunciation Analysis">' if pronunciation_viz_base64 else '<p>No pronunciation visualization available</p>'

    # Fill template
    html = HTML_TEMPLATE.format(
        surah=surah,
        ayah=ayah,
        transliteration=transliteration,
        overall_score=overall_score,
        confidence=confidence,
        rhythm_score=rhythm_score,
        melody_score=melody_score,
        duration_score=duration_score,
        pronunciation_score=pronunciation_score,
        issues_html=issues_html,
        rhythm_viz=rhythm_viz,
        melody_viz=melody_viz,
        duration_viz=duration_viz,
        pronunciation_viz=pronunciation_viz
    )

    # Save if path provided
    if output_path:
        output_file = Path(output_path)
        output_file.parent.mkdir(parents=True, exist_ok=True)
        output_file.write_text(html, encoding='utf-8')
        print(f"✅ Interactive viewer saved to: {output_path}")

    return html


def generate_issues_html(issues: list) -> str:
    """Generate HTML for top issues section."""
    if not issues:
        return """
        <div class="issues-section">
            <h2>🎉 Top Issues</h2>
            <div class="issue-card" style="border-left-color: #2ecc71; background: #e8f8f5;">
                <div class="message">Excellent! No significant issues detected.</div>
            </div>
        </div>
        """

    html = '<div class="issues-section">\n<h2>🎯 Top Issues (Most Important First)</h2>\n'

    category_classes = {
        'critical': 'critical',
        'timing': 'timing',
        'style': 'style'
    }

    for i, issue in enumerate(issues[:3], 1):
        category = issue.get('category', 'style')
        component = issue.get('component', 'unknown')
        impact = issue.get('impact', 0)
        message = issue.get('message', 'No details available')
        feedback = issue.get('tajweed_feedback', '')

        css_class = category_classes.get(category, 'style')

        html += f"""
        <div class="issue-card {css_class}">
            <div class="issue-header">
                <div class="category">#{i} - {category.upper()} ({component})</div>
                <div class="impact">-{impact:.1f} pts</div>
            </div>
            <div class="message">{message}</div>
        """

        if feedback:
            html += f'<div class="feedback">💡 <strong>Guidance:</strong> {feedback}</div>'

        html += '</div>\n'

    html += '</div>\n'
    return html
