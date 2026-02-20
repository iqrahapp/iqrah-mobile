# 07 - Release Gates And Definition Of Done

## Launch Gates (All Required)

## Gate A - Core Learning Correctness

1. Cold-start sessions never empty.
2. Goal/chunk-aware selection verified.
3. 3-budget session mix validated in telemetry.
4. Core scheduled exercise pool stable and tested.
5. G3/G6/G7 blueprint gaps are closed (`C-011`..`C-013`).
6. Artifact usage inventory and safe git-hygiene cleanup completed (`Q-013`, `Q-014`).

## Gate B - Data Platform Reliability

1. Backend release registry live with validation-gated publishing.
2. Mobile bootstrap + atomic activation + rollback tested.
3. Checksum mismatch activation rate = 0.

## Gate C - Sync/Backup Trust

1. Multi-device sync conflict tests pass.
2. Sync observability is live.
3. Offline learning works after initial artifact install.

## Gate D - Product/UI Quality

1. Reader flow is polished and fast.
2. Word/root exploration is complete and usable.
3. Audio-assisted practice exists in main learning flow.
4. UX quality baseline achieved across loading/error/empty states.

## Gate E - Operational Readiness

1. Release publish/deprecate/rollback runbooks verified.
2. Alerting and dashboards active with solo-operator checklist.
3. Security and rate-limit checks in place.

## Final Definition Of Done

The project is considered production-ready only when:
1. All tickets in this folder are complete or explicitly deferred with product-owner approval.
2. All launch gates A-E pass with evidence.
3. Both source sets are satisfied:
  - `docs/ai_blueprint-3`
  - `docs/data_platform_blueprint`
4. The app demonstrably delivers:
  - reliable daily learning flow,
  - beautiful and motivating interface,
  - safe data updates,
  - dependable backup/sync across devices.
