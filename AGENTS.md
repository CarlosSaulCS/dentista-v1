# Codex Instructions

This repository is the local-first Windows installable dental system for Code Solution Studio.

## Product context

- Keep the product local-first and installable.
- The web SaaS version lives in `CarlosSaulCS/css-aion-new`.
- This project must feel like a professional premium desktop product, not a demo.
- User-facing copy should remain in Spanish.

## Main goals

1. Stabilize the dental desktop app.
2. Improve the UI/UX so it feels modern and premium.
3. Keep local data safe and reliable.
4. Preserve the SQLite data model unless a migration is clearly needed.
5. Prepare the Windows release workflow.
6. Prepare optional future cloud sync without making the app depend on internet.

## Stack

- Tauri 2
- Rust
- React
- TypeScript
- Vite
- Tailwind CSS
- SQLite + SQLx
- TanStack Query/Table
- Zustand
- React Hook Form
- Zod
- Recharts

## Areas to inspect before editing

- `src-tauri/src/lib.rs`
- `src-tauri/src/commands/`
- `src-tauri/src/services/`
- `src-tauri/src/repositories/`
- `src-tauri/src/database/`
- `src-tauri/migrations/`
- `src/app/`
- `src/routes/`
- `src/layouts/`
- `src/features/`
- `src/store/`
- `src/lib/`
- `docs/`

## Required checks

Run these when possible:

```bash
npm run typecheck
npm run lint
npm run build
cd src-tauri && cargo test
```

If a command cannot run in the environment, explain why in the final response or PR summary.

## Implementation rules

- Do not remove existing modules without a clear reason.
- Do not break existing SQLite installations.
- Add migrations instead of destructive schema changes.
- Keep changes small enough to review.
- Prefer typed service boundaries.
- Validate inputs at both frontend and backend command boundaries.
- Keep license, backup, restore, patient data, payment, and appointment flows conservative and reliable.
- Do not expose the local desktop app directly to the public internet.

## Remote/mobile access strategy

For remote access, prefer this architecture:

1. Desktop app remains the system of record locally.
2. Desktop app optionally syncs selected data with a secure cloud backend.
3. Mobile/PWA app connects to the cloud backend, not directly to the local machine.
4. Sync must be opt-in, auditable, and resilient to offline usage.
5. Conflict handling must be explicit and safe.

## PR summary expectations

Every PR should include:

- What changed.
- Why it changed.
- Commands/tests run.
- Known risks.
- Manual QA steps.
- Follow-up tasks.