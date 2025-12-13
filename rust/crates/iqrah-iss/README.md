# ISS: Iqrah Student Simulation

**Status**: ✅ Production Ready (v2.9.3)
**Latest Release**: v2.9.3 (December 2024)
**Validation**: n=30 seeds, 180 days, Juz 30

The Iqrah Student Simulation (ISS) is a high-fidelity cognitive modeling engine designed to simulate and validate spaced repetition algorithms for Quranic memorization (Hifz).

## Quick Start

ISS v2.9 provides a quality-first memorization engine validated on
Quranic Hifz tasks. Target pace: ~2 ayahs/day (part-time learner).

### Run a Benchmark
```bash
cargo run --release -p iqrah-iss -- compare \
  --preset juz_amma \
  -V iqrah_default,baseline_random \
  -n 3 --days 180
```

### Expected Results
- Coverage: ~65% (370/564 ayahs)
- Exercise Score: 1.0 (iqrah) vs 0.0 (random)
- Retention: >95% on learned items

## Key Features

- ✅ **Retention-Guaranteed**: <10% failure rate, zero death spirals
- ✅ **Cognitively Validated**: 97% superior to random on sequential recall
- ✅ **Production Hardened**: Multi-seed validation (n=30), robust metrics
- ✅ **Conservative by Design**: Quality > speed (2 ayahs/day pace)
- ✅ **Backlog Protection**: Automatic throttling when falling behind
