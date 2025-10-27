#!/usr/bin/env python3
"""Debug test to see what's happening"""

import numpy as np
import sys
sys.path.insert(0, ".")
from src.iqrah_audio.streaming.online_dtw_v2 import TrueOnlineDTW

# Simple identical data
ref = np.array([100.0] * 20)
query = np.array([100.0] * 20)

dtw = TrueOnlineDTW(ref)
dtw.seed(query[:5], force_position=0)

print(f"After seed: ref_pos={dtw.state.reference_position}")
print(f"Params: slope={dtw.slope_constraint}, bonus={dtw.diagonal_bonus}\n")

# Process with debug
for i in range(5, 10):
    print(f"\n{'='*60}")
    print(f"Processing frame {i} (query_frame={query[i]})")
    state = dtw.update(query[i], query_confidence=1.0, debug=True)
    print(f"Result: ref_pos={state.reference_position}, expected={i}")
