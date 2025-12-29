# ISS v3.0-Graph: Static Graph Enrichment

**Status**: Planning Phase  
**Priority**: Medium (Enhancement, not critical)  
**Estimated Effort**: 3-4 weeks  
**Prerequisites**: v2.9.3 shipped and stable

---

## Executive Summary

### What is v3.0-Graph?

v3.0-Graph enriches the **static knowledge graph** with smarter edge weighting and connections to improve learning predictions **without adding runtime state**.

### Core Design Principle

```
Move complexity FROM imperative scheduling code TO declarative graph structure.

The graph IS the model. Enhance the graph, not the scheduler.
```

### What Changed from Original v3.0 Plan?

**Original v3.0**: Dynamic sequential link tracking
- Runtime state: `SequentialLink { strength, last_practiced, practice_count }`
- Daily decay logic, session strengthening
- 60-90 hours implementation
- **REJECTED**: Violates graph-centric design philosophy

**v3.0-Graph**: Static graph enhancements
- NO runtime state (energy remains the ONLY dynamic state)
- Smarter edge weights based on content analysis
- 3-4 weeks implementation
- **APPROVED**: Aligns with declarative architecture

---

## Background: Why Static Graph Enhancement?

### Current v2.9 Architecture (Excellent Foundation)

**Static graph encodes learning structure**:
```python
# Sequential dependencies (verse → verse)
verse:1:mem → verse:2:mem (weight: 1.0)
verse:2:mem → verse:3:mem (weight: 1.0)

# Contextual windows (word → word)
word:N-1:mem → word:N:mem (weight: gaussian(distance))
word:N:mem → word:N+1:mem (weight: gaussian(distance))

# Cross-axis support
word:N:translation → word:N:memorization (weight: 0.4)
```

**Dynamic energy propagates through static edges**:
```rust
Energy state (runtime):
  verse:1:mem.energy = 0.85 (practiced)
  verse:2:mem.energy = 0.82 (practiced)
  verse:3:mem.energy = 0.45 (not practiced)

Propagation:
  verse:1 fails → energy drops to 0.70
  verse:2 receives: 0.70 × 1.0 (edge weight) = 0.70 impact
  verse:3 receives: 0.70 × 1.0 = 0.70 impact (same)
```

**This works!** v2.9 achieved 97% better than random with this simple model.

---

### The Gap: All Edges Weighted Equally

**Current limitation**:
```python
# All verses treated the same
verse:1:mem → verse:2:mem (weight: 1.0)  # Easy verse
verse:5:mem → verse:6:mem (weight: 1.0)  # Hard verse (same weight!)

# All words treated the same  
word:1:mem → word:2:mem (weight: 0.5)  # Common word "wa"
word:5:mem → word:6:mem (weight: 0.5)  # Rare word (same weight!)
```

**Problem**: 
- Hard content should have **stronger edges** (needs more support)
- Easy content can be more **independent**
- Thematic relationships **ignored** (verses about same topic not connected)
- Prosodic patterns **ignored** (rhyming verses not connected)

**v3.0-Graph fixes this by making edge weights CONTENT-AWARE.**

---

## v3.0-Graph Goals

### Primary Objectives

1. **Difficulty-Aware Edges**: Hard content gets stronger support edges
2. **Thematic Connections**: Semantically similar content connected
3. **Prosodic Patterns**: Phonetically similar content connected  
4. **Optimized Gaussian Windows**: Tune context parameters per-level
5. **Enhanced Propagation**: Smarter energy flow (still stateless)

### Success Metrics

- Exercise scores improve by 5-10% on same scenarios
- Predictions more accurate (trials closer to cognitive reality)
- NO new runtime state (energy remains only dynamic component)
- Graph enrichment visible in edge count/weight distribution

---

## Phase 1: Difficulty-Weighted Edges

**Duration**: 1 week  
**Goal**: Weight edges based on content difficulty

### Implementation

#### 1.1: Compute Difficulty Scores

**Python implementation** (graph builder):

```python
# File: iqrah_graph_builder/difficulty.py

class DifficultyScorer:
    """Compute difficulty scores for Quranic content."""
    
    def __init__(self, quran: Quran):
        self.quran = quran
        self._word_frequency = self._compute_word_frequencies()
    
    def compute_word_difficulty(self, word: Word) -> float:
        """
        Compute word difficulty (0.0-1.0, higher = harder).
        
        Factors:
        - Length (letters count)
        - Frequency (rare words harder)
        - Tajweed rules (if available)
        """
        # Length factor (normalize by max length ~15)
        length_score = min(word.get_letters_count() / 15.0, 1.0)
        
        # Frequency factor (inverse of log frequency)
        text = word.text_uthmani_simple
        freq = self._word_frequency.get(text, 1)
        frequency_score = 1.0 - (math.log(freq + 1) / math.log(10000))
        
        # Tajweed factor (placeholder - requires tajweed detection)
        tajweed_score = 0.0  # TODO: Implement when tajweed rules available
        
        # Weighted combination
        difficulty = (
            0.3 * length_score +
            0.5 * frequency_score +
            0.2 * tajweed_score
        )
        
        return difficulty.clip(0.0, 1.0)
    
    def compute_verse_difficulty(self, verse: Verse) -> float:
        """
        Compute verse difficulty (0.0-1.0).
        
        Factors:
        - Average word difficulty
        - Verse length
        - Rare word density
        """
        words = [w for w in verse.words if not w.is_end_word()]
        if not words:
            return 0.0
        
        # Average word difficulty
        word_difficulties = [self.compute_word_difficulty(w) for w in words]
        avg_difficulty = sum(word_difficulties) / len(word_difficulties)
        
        # Length factor (long verses harder to memorize)
        length_score = min(len(words) / 50.0, 1.0)  # 50+ words = very long
        
        # Rare word density (proportion of hard words)
        hard_word_count = sum(1 for d in word_difficulties if d > 0.6)
        rare_density = hard_word_count / len(words)
        
        difficulty = (
            0.5 * avg_difficulty +
            0.3 * length_score +
            0.2 * rare_density
        )
        
        return difficulty.clip(0.0, 1.0)
    
    def _compute_word_frequencies(self) -> dict:
        """Count word frequencies across entire Quran."""
        frequencies = Counter()
        
        for chapter in self.quran.chapters:
            for verse in chapter.verses:
                for word in verse.words:
                    if not word.is_end_word():
                        frequencies[word.text_uthmani_simple] += 1
        
        return dict(frequencies)
```

#### 1.2: Apply Difficulty Weighting

**Update graph builder**:

```python
# File: iqrah_graph_builder/builder.py

class KnowledgeGraphBuilder:
    
    def __init__(self, graph: nx.DiGraph, quran: Quran):
        self.G = graph
        self.quran = quran
        self.edge_manager = KnowledgeEdgeManager(graph)
        self.node_manager = NodeManager(graph)
        
        # NEW: Difficulty scorer
        self.difficulty_scorer = DifficultyScorer(quran)
    
    def build_memorization_edges(self) -> int:
        """Build difficulty-aware memorization edges."""
        logger.info("Building difficulty-aware memorization edges...")
        edges_before = self.stats["edges_created"]
        
        for chapter_id in self.get_nodes_by_type("chapter"):
            chapter_key = NIP.get_chapter_key(chapter_id)
            chapter = self.quran[chapter_key]
            
            for verse in chapter.verses:
                verse_id = NIG.for_verse(verse)
                
                # NEW: Compute verse difficulty
                difficulty = self.difficulty_scorer.compute_verse_difficulty(verse)
                
                # NEW: Difficulty-weighted edge
                # Harder verses get STRONGER edges (need more support)
                edge_weight = verse.get_letters_count() * (1.0 + difficulty * 0.5)
                
                # Verse -> Chapter memorization
                self.edge_manager.add_knowledge_edge(
                    f"{verse_id}:memorization",
                    f"{chapter_id}:memorization",
                    Distribution.auto(weight=edge_weight)
                )
                self.stats["edges_created"] += 1
                
                # Word -> Verse (difficulty-aware)
                for word in verse.words:
                    if word.is_end_word():
                        continue
                    
                    word_id = NIG.for_word_instance(word, verse)
                    word_difficulty = self.difficulty_scorer.compute_word_difficulty(word)
                    word_weight = word.get_letters_count() * (1.0 + word_difficulty * 0.5)
                    
                    self.edge_manager.add_knowledge_edge(
                        f"{word_id}:memorization",
                        f"{verse_id}:memorization",
                        Distribution.auto(weight=word_weight)
                    )
                    self.stats["edges_created"] += 1
        
        # ... rest of memorization edges ...
```

**Rust implementation** (update knowledge edge builder):

```rust
// File: crates/iqrah-knowledge-graph/src/knowledge.rs

// Add difficulty computation module
mod difficulty {
    use std::collections::HashMap;
    
    pub struct DifficultyScorer {
        word_frequencies: HashMap<String, u32>,
    }
    
    impl DifficultyScorer {
        pub fn new(quran: &QuranData) -> Self {
            let mut frequencies = HashMap::new();
            
            // Count word frequencies
            for verse in &quran.verses {
                // Parse words and count
                // (implementation depends on your Quran data structure)
            }
            
            Self { word_frequencies: frequencies }
        }
        
        pub fn compute_word_difficulty(&self, word_text: &str, letter_count: usize) -> f64 {
            // Length score
            let length_score = (letter_count as f64 / 15.0).min(1.0);
            
            // Frequency score
            let freq = self.word_frequencies.get(word_text).unwrap_or(&1);
            let frequency_score = 1.0 - ((*freq as f64 + 1.0).ln() / 10000.0_f64.ln());
            
            // Weighted combination
            let difficulty = 0.3 * length_score + 0.5 * frequency_score;
            difficulty.clamp(0.0, 1.0)
        }
        
        pub fn compute_verse_difficulty(&self, words: &[WordData]) -> f64 {
            if words.is_empty() {
                return 0.0;
            }
            
            let avg_difficulty: f64 = words.iter()
                .map(|w| self.compute_word_difficulty(&w.text, w.letter_count))
                .sum::<f64>() / words.len() as f64;
            
            let length_score = (words.len() as f64 / 50.0).min(1.0);
            
            let difficulty = 0.5 * avg_difficulty + 0.3 * length_score;
            difficulty.clamp(0.0, 1.0)
        }
    }
}

// Update build_knowledge_edges to use difficulty
pub fn build_knowledge_edges(graph: &mut Graph, node_map: &mut HashMap<i64, NodeIndex>) {
    println!("Building difficulty-aware knowledge edges...");
    
    // Create difficulty scorer
    let scorer = difficulty::DifficultyScorer::new(&quran_data);
    
    // ... existing node collection ...
    
    // Apply difficulty weights
    for &(v_idx, v_id) in &verses {
        let difficulty = scorer.compute_verse_difficulty(&verse_words);
        
        // Stronger edge for harder verses
        let edge_weight = 1.0 * (1.0 + difficulty * 0.5);
        
        // Create edge with difficulty-adjusted weight
        graph.add_edge(v_k_idx, ch_k_idx, EdgeData {
            edge_type: EdgeType::Knowledge,
            weight: edge_weight,
        });
    }
}
```

#### 1.3: Validation

**Test difficulty computation**:

```python
# Test: Fatiha vs Baqarah difficulty
fatiha = quran["1"]  # Short, common words
baqarah_50 = quran["2:50"]  # Long, complex

fatiha_diff = scorer.compute_verse_difficulty(fatiha.verses[0])
baqarah_diff = scorer.compute_verse_difficulty(baqarah_50)

assert baqarah_diff > fatiha_diff  # Baqarah should be harder
print(f"Fatiha: {fatiha_diff:.2f}, Baqarah: {baqarah_diff:.2f}")
```

**Expected impact**:
- Exercise scores improve by 3-5% (harder verses get more support)
- Trial predictions more accurate (hard content accounts for hesitation)

---

## Phase 2: Thematic Connections

**Duration**: 1 week  
**Goal**: Connect semantically similar verses across surahs

### Implementation

#### 2.1: Extract Topics from Tafsir

**Python implementation**:

```python
# File: iqrah_graph_builder/thematic.py

class ThematicAnalyzer:
    """Extract and connect verses by themes."""
    
    # Common Quranic themes (can be expanded)
    THEMES = {
        'prayer': ['صلاة', 'الصلاة', 'صلوا'],
        'charity': ['زكاة', 'الزكاة', 'صدقة'],
        'fasting': ['صيام', 'صوم', 'الصيام'],
        'hajj': ['حج', 'الحج', 'بيت'],
        'prophets': ['نوح', 'إبراهيم', 'موسى', 'عيسى', 'محمد'],
        'paradise': ['جنة', 'الجنة', 'جنات'],
        'hellfire': ['نار', 'النار', 'جهنم'],
        'belief': ['آمن', 'يؤمن', 'إيمان'],
        'disbelief': ['كفر', 'الكافرين', 'كفروا'],
    }
    
    def __init__(self, quran: Quran):
        self.quran = quran
    
    def extract_verse_themes(self, verse: Verse) -> Set[str]:
        """Extract themes from verse text."""
        themes = set()
        text = verse.text_uthmani_simple
        
        if not text:
            return themes
        
        for theme, keywords in self.THEMES.items():
            for keyword in keywords:
                if keyword in text:
                    themes.add(theme)
        
        return themes
    
    def find_thematic_pairs(self, min_shared_themes: int = 1) -> List[Tuple[str, str, Set[str]]]:
        """
        Find verse pairs sharing themes.
        
        Returns:
            List of (verse_key_1, verse_key_2, shared_themes)
        """
        # Build theme index
        verse_themes = {}
        for chapter in self.quran.chapters:
            for verse in chapter.verses:
                themes = self.extract_verse_themes(verse)
                if themes:
                    verse_themes[verse.verse_key] = themes
        
        # Find pairs
        pairs = []
        verse_keys = list(verse_themes.keys())
        
        for i in range(len(verse_keys)):
            for j in range(i + 1, len(verse_keys)):
                v1, v2 = verse_keys[i], verse_keys[j]
                
                shared = verse_themes[v1] & verse_themes[v2]
                
                if len(shared) >= min_shared_themes:
                    pairs.append((v1, v2, shared))
        
        return pairs
```

#### 2.2: Add Thematic Edges

**Update graph builder**:

```python
# File: iqrah_graph_builder/builder.py

class KnowledgeGraphBuilder:
    
    def build_thematic_edges(self) -> int:
        """
        Build thematic connection edges.
        
        Connects verses with shared themes (prayer, charity, etc.)
        to aid understanding transfer.
        
        Returns:
            Number of edges created
        """
        logger.info("Building thematic edges...")
        edges_before = self.stats["edges_created"]
        
        analyzer = ThematicAnalyzer(self.quran)
        thematic_pairs = analyzer.find_thematic_pairs(min_shared_themes=1)
        
        for v1_key, v2_key, shared_themes in thematic_pairs:
            v1_id = NIG.for_verse(v1_key)
            v2_id = NIG.for_verse(v2_key)
            
            # Weight by number of shared themes
            theme_weight = len(shared_themes) * 0.3  # 0.3 per shared theme
            
            # Bidirectional: understanding one helps understand the other
            self.edge_manager.add_bidirectional_knowledge_edge(
                f"{v1_id}:translation",
                f"{v2_id}:translation",
                Distribution.normal(mean=theme_weight, std=0.1)
            )
            self.stats["edges_created"] += 2
        
        logger.info(f"Connected {len(thematic_pairs)} thematic pairs")
        
        edges_created = self.stats["edges_created"] - edges_before
        logger.info(f"Created {edges_created} thematic edges")
        return edges_created
```

**Rust implementation**:

```rust
// File: crates/iqrah-knowledge-graph/src/thematic.rs

pub struct ThematicAnalyzer {
    themes: HashMap<&'static str, Vec<&'static str>>,
}

impl ThematicAnalyzer {
    pub fn new() -> Self {
        let mut themes = HashMap::new();
        themes.insert("prayer", vec!["صلاة", "الصلاة", "صلوا"]);
        themes.insert("charity", vec!["زكاة", "الزكاة", "صدقة"]);
        // ... more themes
        
        Self { themes }
    }
    
    pub fn extract_verse_themes(&self, verse_text: &str) -> HashSet<String> {
        let mut verse_themes = HashSet::new();
        
        for (theme, keywords) in &self.themes {
            for keyword in keywords {
                if verse_text.contains(keyword) {
                    verse_themes.insert(theme.to_string());
                }
            }
        }
        
        verse_themes
    }
    
    pub fn find_thematic_pairs(&self, verses: &[VerseData]) -> Vec<(i64, i64, usize)> {
        // Build theme index
        let mut verse_themes = HashMap::new();
        for verse in verses {
            let themes = self.extract_verse_themes(&verse.text);
            if !themes.is_empty() {
                verse_themes.insert(verse.id, themes);
            }
        }
        
        // Find pairs with shared themes
        let mut pairs = Vec::new();
        let verse_ids: Vec<i64> = verse_themes.keys().copied().collect();
        
        for i in 0..verse_ids.len() {
            for j in (i + 1)..verse_ids.len() {
                let v1 = verse_ids[i];
                let v2 = verse_ids[j];
                
                let themes1 = &verse_themes[&v1];
                let themes2 = &verse_themes[&v2];
                
                let shared_count = themes1.intersection(themes2).count();
                
                if shared_count >= 1 {
                    pairs.push((v1, v2, shared_count));
                }
            }
        }
        
        pairs
    }
}
```

#### 2.3: Validation

**Expected thematic connections**:
```
All verses about salah (prayer) connected:
  2:43, 2:110, 2:238, 4:103, etc. → thematic cluster

All verses about prophets connected:
  Stories of Ibrahim, Musa, Nuh → cross-surah learning
  
Understanding one verse helps understand thematically similar verses
```

**Impact**:
- Translation axis scores improve (understanding transfers)
- Cross-surah learning emerges naturally from graph

---

## Phase 3: Prosodic Pattern Edges

**Duration**: 1 week  
**Goal**: Connect verses with similar phonetic/rhyme patterns

### Implementation

#### 3.1: Detect Rhyme Patterns

**Python implementation**:

```python
# File: iqrah_graph_builder/prosody.py

class ProsodicAnalyzer:
    """Detect and connect prosodic patterns (rhymes, rhythms)."""
    
    def __init__(self, quran: Quran):
        self.quran = quran
    
    def extract_rhyme_pattern(self, verse: Verse) -> Optional[str]:
        """
        Extract rhyme pattern from verse ending.
        
        Returns:
            Last 2-3 characters (simplified rhyme detection)
        """
        words = [w for w in verse.words if not w.is_end_word()]
        if not words:
            return None
        
        last_word = words[-1].text_uthmani_simple
        if not last_word or len(last_word) < 3:
            return None
        
        # Simple rhyme: last 2 characters
        rhyme = last_word[-2:]
        return rhyme
    
    def find_rhyming_pairs(self) -> List[Tuple[str, str, str]]:
        """
        Find verse pairs with matching rhyme patterns.
        
        Returns:
            List of (verse_key_1, verse_key_2, rhyme_pattern)
        """
        # Build rhyme index
        rhyme_index = {}
        
        for chapter in self.quran.chapters:
            for verse in chapter.verses:
                rhyme = self.extract_rhyme_pattern(verse)
                if rhyme:
                    if rhyme not in rhyme_index:
                        rhyme_index[rhyme] = []
                    rhyme_index[rhyme].append(verse.verse_key)
        
        # Find pairs
        pairs = []
        
        for rhyme, verse_keys in rhyme_index.items():
            if len(verse_keys) < 2:
                continue
            
            # Connect all verses with same rhyme
            for i in range(len(verse_keys)):
                for j in range(i + 1, len(verse_keys)):
                    pairs.append((verse_keys[i], verse_keys[j], rhyme))
        
        return pairs
```

#### 3.2: Add Prosodic Edges

**Update graph builder**:

```python
# File: iqrah_graph_builder/builder.py

class KnowledgeGraphBuilder:
    
    def build_prosodic_edges(self) -> int:
        """
        Build prosodic pattern edges.
        
        Connects verses with similar rhyme schemes to aid
        memorization through phonetic patterns.
        
        Returns:
            Number of edges created
        """
        logger.info("Building prosodic edges...")
        edges_before = self.stats["edges_created"]
        
        analyzer = ProsodicAnalyzer(self.quran)
        rhyming_pairs = analyzer.find_rhyming_pairs()
        
        for v1_key, v2_key, rhyme in rhyming_pairs:
            v1_id = NIG.for_verse(v1_key)
            v2_id = NIG.for_verse(v2_key)
            
            # Rhyme pattern aids memorization (not translation)
            self.edge_manager.add_bidirectional_knowledge_edge(
                f"{v1_id}:memorization",
                f"{v2_id}:memorization",
                Distribution.normal(mean=0.4, std=0.1)
            )
            self.stats["edges_created"] += 2
        
        logger.info(f"Connected {len(rhyming_pairs)} rhyming pairs")
        
        edges_created = self.stats["edges_created"] - edges_before
        logger.info(f"Created {edges_created} prosodic edges")
        return edges_created
```

#### 3.3: Validation

**Expected prosodic connections**:
```
Surah Al-Rahman (55): All verses end with "-aan"
→ All verses connected (strong rhyme cluster)

Surah Al-Mursalat (77): Repetitive pattern "Woe to deniers"
→ Pattern verses connected

Learning one rhyming verse helps recall others
```

**Impact**:
- Memorization aided by phonetic similarity
- Prosodic patterns emerge from graph structure

---

## Phase 4: Gaussian Window Optimization

**Duration**: 3-5 days  
**Goal**: Tune context window parameters per node type

### Implementation

#### 4.1: Current Gaussian Windows

**Current implementation**:

```python
# Words: window_size=3, base_weight=0.5, std_scale=0.15
# Verses: window_size=1, base_weight=0.7, std_scale=0.1

# Gaussian falloff:
# distance=0 (self): weight=base_weight
# distance=1: weight=base_weight * exp(-(1^2) / (2 * std_scale^2))
# distance=2: weight=base_weight * exp(-(2^2) / (2 * std_scale^2))
```

**Questions to optimize**:
1. Is window_size=3 optimal for words?
2. Is base_weight=0.5 correct?
3. Should std_scale vary by content difficulty?

#### 4.2: Parameter Grid Search

**Python implementation**:

```python
# File: iqrah_graph_builder/optimizer.py

class GaussianWindowOptimizer:
    """Optimize Gaussian window parameters via grid search."""
    
    PARAM_GRID = {
        'word': {
            'window_size': [2, 3, 4, 5],
            'base_weight': [0.4, 0.5, 0.6, 0.7],
            'std_scale': [0.10, 0.12, 0.15, 0.18],
        },
        'verse': {
            'window_size': [1, 2, 3],
            'base_weight': [0.6, 0.7, 0.8],
            'std_scale': [0.08, 0.10, 0.12],
        },
    }
    
    def __init__(self, quran: Quran, exercise_framework):
        self.quran = quran
        self.exercise_framework = exercise_framework
    
    def optimize(self, node_type: str, metric: str = 'exercise_score') -> dict:
        """
        Grid search over parameter space.
        
        Args:
            node_type: 'word' or 'verse'
            metric: 'exercise_score' or 'trial_accuracy'
        
        Returns:
            Best parameters dict
        """
        param_grid = self.PARAM_GRID[node_type]
        
        best_score = 0.0
        best_params = None
        
        for window_size in param_grid['window_size']:
            for base_weight in param_grid['base_weight']:
                for std_scale in param_grid['std_scale']:
                    
                    # Build graph with these parameters
                    graph = self._build_graph(
                        node_type=node_type,
                        window_size=window_size,
                        base_weight=base_weight,
                        std_scale=std_scale,
                    )
                    
                    # Run exercise evaluation
                    score = self._evaluate(graph, metric)
                    
                    if score > best_score:
                        best_score = score
                        best_params = {
                            'window_size': window_size,
                            'base_weight': base_weight,
                            'std_scale': std_scale,
                        }
        
        logger.info(f"Best {node_type} params: {best_params} (score: {best_score:.3f})")
        return best_params
```

#### 4.3: Difficulty-Adaptive Windows

**Smarter approach**: Adjust window size based on difficulty

```python
def add_difficulty_adaptive_windows(
    nodes: List[str],
    difficulties: List[float],
    base_window_size: int = 3,
) -> int:
    """
    Create Gaussian windows with difficulty-adaptive sizing.
    
    Hard items get WIDER windows (more context support).
    Easy items get NARROWER windows (more independence).
    """
    edges_created = 0
    
    for i, node in enumerate(nodes):
        difficulty = difficulties[i]
        
        # Adaptive window size
        window_size = int(base_window_size * (1.0 + difficulty * 0.5))
        
        # Create window for this node
        for offset in range(-window_size, window_size + 1):
            j = i + offset
            if j < 0 or j >= len(nodes) or j == i:
                continue
            
            target_node = nodes[j]
            distance = abs(offset)
            
            # Gaussian weight (with difficulty boost)
            weight = 0.5 * math.exp(-(distance**2) / (2 * 0.15**2))
            weight *= (1.0 + difficulty * 0.3)  # Hard items get stronger edges
            
            self.add_edge(node, target_node, weight=weight)
            edges_created += 1
    
    return edges_created
```

#### 4.4: Validation

**Test parameters on Fatiha**:

```python
# Run with different parameters
params_old = {'window_size': 3, 'base_weight': 0.5, 'std_scale': 0.15}
params_new = {'window_size': 5, 'base_weight': 0.6, 'std_scale': 0.12}

score_old = run_exercise(graph_old)  # 0.75
score_new = run_exercise(graph_new)  # 0.82 (+9%)

# Optimal: wider windows, stronger base, sharper falloff
```

**Impact**:
- Exercise scores improve by 5-8%
- Context support tuned to optimal range

---

## Phase 5: Enhanced Energy Propagation

**Duration**: 3-5 days  
**Goal**: Smarter propagation formula (still stateless)

### Implementation

**Current propagation** (implicit in ISS):
```rust
// Simple weighted propagation
fn propagate_energy(source: f64, edge_weight: f64) -> f64 {
    source * edge_weight
}
```

**v3.0-Graph enhancement**:

```rust
// File: crates/iqrah-iss/src/energy_propagation.rs

/// Enhanced energy propagation with context-awareness
pub fn propagate_energy_v3(
    source_energy: f64,
    target_energy: f64,
    edge_weight: f64,
    practice_gap: u32,  // Days since practiced together
) -> f64 {
    // Base propagation
    let base = source_energy * edge_weight;
    
    // Recency factor: if practiced together recently → stronger propagation
    let recency_factor = (-practice_gap as f64 / 30.0).exp();
    
    // Similarity bonus: if energies similar → stronger connection assumed
    let similarity = 1.0 - (source_energy - target_energy).abs();
    let similarity_bonus = similarity * 0.3;
    
    // Combined: base × (static + dynamic boost)
    base * (0.7 + 0.3 * recency_factor + similarity_bonus)
}
```

**Key point**: This is computed at RUNTIME but doesn't require storing state.

**Alternative (simpler)**: Just enhance the static formula:

```python
# In graph builder - compute edge weight with context
def compute_edge_weight(source, target, base_weight):
    # Position similarity (closer nodes → stronger)
    position_distance = abs(source.position - target.position)
    position_factor = exp(-(position_distance**2) / 10.0)
    
    # Difficulty similarity (similar difficulty → stronger)
    difficulty_diff = abs(source.difficulty - target.difficulty)
    difficulty_factor = 1.0 - difficulty_diff
    
    # Combined static weight
    weight = base_weight * position_factor * difficulty_factor
    return weight
```

**This keeps graph fully static** - no runtime state needed.

---

## Integration Plan

### Update build_all() Method

```python
# File: iqrah_graph_builder/builder.py

class KnowledgeGraphBuilder:
    
    def build_all(
        self,
        include_memorization: bool = True,
        include_translation: bool = True,
        include_grammar: bool = True,
        
        # NEW v3.0-Graph options
        include_difficulty_weighting: bool = True,
        include_thematic_edges: bool = True,
        include_prosodic_edges: bool = True,
        optimize_gaussian_windows: bool = False,  # Slow, run once
    ) -> None:
        """Build all knowledge edges with v3.0-Graph enhancements."""
        
        if self._is_compiled:
            raise RuntimeError("Already compiled")
        
        logger.info("Building v3.0-Graph knowledge graph...")
        
        # Original v2.9 edges (with difficulty weighting if enabled)
        if include_memorization:
            self.build_memorization_edges()
        
        if include_translation:
            self.build_translation_edges()
        
        if include_grammar:
            self.build_grammar_edges()
        
        # NEW: Thematic connections
        if include_thematic_edges:
            self.build_thematic_edges()
        
        # NEW: Prosodic patterns
        if include_prosodic_edges:
            self.build_prosodic_edges()
        
        # NEW: Gaussian optimization (expensive)
        if optimize_gaussian_windows:
            self._optimize_and_rebuild_windows()
```

---

## Expected Impact

### Quantitative Improvements

**Exercise scores** (predicted):
```
Before v3.0-Graph (v2.9):
  Fatiha: 0.75 (Good)
  Juz 30: 1.0 (Easy)

After v3.0-Graph:
  Fatiha: 0.82 (+9%)
  Juz 30: 1.0 (same, already optimal)

Improvement: 5-10% on small/medium goals
```

**Trial prediction accuracy**:
```
Before: Hard verses predicted as 2.5 trials (actual: 3.2)
After: Hard verses predicted as 3.1 trials (actual: 3.2)

Error reduction: 30-40%
```

### Qualitative Improvements

**Thematic learning**:
```
User learns verse about salah in Surah 2
→ Automatically improves understanding of salah verses in Surah 4, 5
→ Graph encodes semantic relationships
```

**Prosodic patterns**:
```
User memorizes first verse of Al-Rahman (rhyme: "-aan")
→ All other "-aan" verses slightly boosted (phonetic priming)
→ Natural memorization patterns emerge
```

**Difficulty awareness**:
```
Hard verses (rare words, long) get stronger support edges
→ Energy propagation stronger for difficult content
→ Realistic hesitation captured
```

---

## Success Criteria

### Minimum Viable

- [ ] Difficulty scores computed for all verses/words
- [ ] Edge weights adjusted by difficulty (harder → stronger)
- [ ] Thematic connections created (at least 500+ pairs)
- [ ] Prosodic connections created (rhyme detection working)
- [ ] No regressions (v3.0-Graph ≥ v2.9 scores)
- [ ] Still zero runtime state (energy only)

### Stretch Goals

- [ ] Exercise scores improve by 8-10% (not just 5%)
- [ ] Trial prediction error <10% (vs 15% in v2.9)
- [ ] Thematic clusters visible in graph visualization
- [ ] Gaussian parameters optimized per level

---

## Non-Goals (Out of Scope)

**v3.0-Graph does NOT include**:
- ❌ Runtime link state (SequentialLink, decay, practice counts)
- ❌ Dynamic edge weights (all weights static at graph build time)
- ❌ Scheduler changes (only graph changes)
- ❌ UI/visualization (backend only)
- ❌ Tajweed rule detection (placeholder, future work)

---

## Timeline

**Total**: 3-4 weeks

- **Week 1**: Difficulty weighting (Phase 1)
- **Week 2**: Thematic connections (Phase 2)
- **Week 3**: Prosodic patterns (Phase 3)
- **Week 4**: Gaussian tuning + validation (Phases 4-5)

**Milestones**:
- Week 1: Difficulty scores validated on sample surahs
- Week 2: Thematic clusters visible (prayer, charity, etc.)
- Week 3: Rhyme patterns connected
- Week 4: Exercise scores improved, ready to ship

---

## Migration from v2.9

### Zero Breaking Changes

**v2.9 graphs remain valid**:
- Old graphs work unchanged
- New graphs have additional edges (backward compatible)
- No runtime state changes
- No config changes required

### Optional Rebuild

**To get v3.0-Graph benefits**:
```bash
# Rebuild knowledge graph with v3.0-Graph enhancements
python -m iqrah_graph_builder.cli \
  --data-dir /path/to/quran \
  --output knowledge_v3.graphml \
  --enable-difficulty-weighting \
  --enable-thematic-edges \
  --enable-prosodic-edges
```

**Result**: New graph file with enhanced edges.

---

## Validation Plan

### Phase 1 Validation: Difficulty

```bash
# Test difficulty scoring
pytest tests/test_difficulty_scorer.py

# Verify edge weights adjusted
python scripts/validate_difficulty_edges.py

# Expected: Hard verses have 1.3-1.5x stronger edges
```

### Phase 2 Validation: Thematic

```bash
# Find thematic clusters
python scripts/find_thematic_clusters.py

# Expected output:
# Prayer cluster: 87 verses
# Charity cluster: 56 verses
# Prophets cluster: 234 verses
```

### Phase 3 Validation: Prosodic

```bash
# Find rhyme patterns
python scripts/find_rhyme_patterns.py

# Expected: Surah 55 (Al-Rahman) = 78 rhyming verses
```

### Phase 4 Validation: Exercise Scores

```bash
# Run benchmark with v2.9 graph
cargo run --release -p iqrah-iss -- compare \
  --graph knowledge_v2.9.graphml \
  --scenario fatiha \
  -n 10 --days 30

# Run benchmark with v3.0 graph
cargo run --release -p iqrah-iss -- compare \
  --graph knowledge_v3.0.graphml \
  --scenario fatiha \
  -n 10 --days 30

# Compare scores: v3.0 should be 5-10% higher
```

---

## Documentation Requirements

### Update CHANGELOG.md

```markdown
## [v3.0.0] - TBD

### Graph Enrichment (v3.0-Graph)

**Static graph enhancements** - no runtime state changes.

**What's New**:
1. **Difficulty-Weighted Edges**: Hard content gets stronger support
2. **Thematic Connections**: Semantically similar verses connected
3. **Prosodic Patterns**: Rhyming verses connected (aids memorization)
4. **Optimized Gaussian Windows**: Context parameters tuned per level

**Impact**:
- Exercise scores improved by 5-10%
- Trial predictions more accurate (hard content properly modeled)
- Thematic learning emerges (understanding transfers across surahs)
- Prosodic patterns aid memorization (phonetic priming)

**Design Philosophy**:
- ZERO runtime state (energy remains only dynamic component)
- Fully declarative (complexity in graph, not scheduler)
- Backward compatible (v2.9 graphs still work)

**Breaking Changes**: None

**Migration**: Optional graph rebuild for enhanced edges
```

### Create v3.0-Graph Guide

**File**: `docs/v3_0_graph_enhancements.md`

```markdown
# v3.0-Graph: Static Knowledge Graph Enrichment

## Overview

v3.0-Graph enriches the static knowledge graph with content-aware
edge weighting and semantic/prosodic connections.

## Design Principle

> Move complexity FROM imperative code TO declarative graph structure.

All enhancements are STATIC (computed at graph build time).
NO runtime state beyond energy.

## Enhancements

### 1. Difficulty-Weighted Edges

Hard content (rare words, long verses) gets stronger support edges.

**Example**:
- Verse with rare words: edge weight × 1.5
- Common verse: edge weight × 1.0

**Benefit**: Realistic difficulty modeling

### 2. Thematic Connections

Verses about same topic connected (prayer, charity, prophets).

**Benefit**: Understanding transfers across surahs

### 3. Prosodic Patterns

Rhyming verses connected (phonetic similarity).

**Benefit**: Memorization aided by sound patterns

## Building a v3.0 Graph

```bash
python -m iqrah_graph_builder.cli \
  --enable-difficulty-weighting \
  --enable-thematic-edges \
  --enable-prosodic-edges
```

## Validation

Exercise scores improve by 5-10% on same scenarios.
```

---

## Open Questions

### Q1: Should Thematic Edges Be Weighted by Theme Strength?

**Current**: All thematic edges = 0.3 per shared theme

**Alternative**: Weight by theme importance
- Prayer (core pillar): 0.5
- Minor theme: 0.2

**Decision**: Start with uniform, refine based on validation.

---

### Q2: Should Gaussian Windows Be Asymmetric?

**Current**: Symmetric (backward/forward same weight)

**Alternative**: Forward-weighted (memorization is directional)
- Forward edge: base_weight × 1.0
- Backward edge: base_weight × 0.7

**Decision**: Try symmetric first, asymmetric in v3.1 if needed.

---

### Q3: How Many Themes to Include?

**Current**: 9 themes (prayer, charity, prophets, etc.)

**Alternative**: Expand to 30+ themes (more granular)

**Decision**: Start with 9 core themes, expand based on coverage analysis.

---

## Conclusion

**v3.0-Graph = Static graph enrichment, NOT dynamic link tracking**

**Aligns with design philosophy**:
- Declarative (complexity in graph)
- Stateless (no runtime state beyond energy)
- Scalable (graph tuning, not scheduler hacks)

**Practical**:
- 3-4 weeks (vs 6-8 for dynamic links)
- No migrations (pure graph changes)
- Backward compatible (v2.9 graphs work)

**Impactful**:
- 5-10% exercise score improvement
- More accurate predictions
- Semantic/prosodic patterns emerge

---

**End of v3.0-Graph Planning Document**

*This document is self-contained. You can execute v3.0-Graph from this document alone without remembering the context of this conversation.*