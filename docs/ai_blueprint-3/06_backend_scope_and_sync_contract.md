# 06 - Backend Scope And Sync Contract

## 1) Backend Role (Current)

Repository audited: `/home/shared/ws/iqrah/iqrah-backend`

Backend is a service layer for:
- auth (Google token exchange + JWT)
- content pack catalog/distribution
- device sync push/pull
- admin operations

It is not currently the scheduling brain.
That remains in local Rust core on device.

## 2) Confirmed API Surface (OpenAPI)

Observed path groups in `openapi.json` include:
- `/v1/auth/google`
- `/v1/users/me`
- `/v1/packs/*`
- `/v1/sync/push`
- `/v1/sync/pull`
- `/v1/admin/*`

This aligns with your original objective of local-first learning plus eventual multi-device sync.

## 3) Why This Architecture Is Still Correct

For your goals, local-first is the right default:
- low latency reviews
- offline reliability
- user progress privacy/control

Backend should focus on:
- identity
- durable sync/merge/conflict handling
- content/package distribution
- optional cloud analytics

Not on replacing device-side scheduling at this stage.

## 4) Current Mobile Sync Integration

Flutter sync stack:
- providers: `lib/providers/sync_provider.dart`
- backend calls: `lib/services/sync_service.dart`
- local-to-remote mapping: `lib/services/sync_mapper.dart`

Local sync payload currently includes:
- memory states
- sessions
- session items
- app settings timestamps

## 5) Maturity Notes

- Backend repo is explicitly WIP, but direction is coherent.
- Mobile has periodic sync loop and auth-aware trigger.
- The largest current product gap is not backend throughput; it is the local session quality and onboarding scheduling policy.

## 6) Strategic Guidance

Do not move scheduling server-side yet.
First make local scheduling/exercise policy pedagogically correct and measurable.
Then let backend remain transport and state convergence layer.
