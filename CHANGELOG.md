# nestGate — Evolution Record

The changelog is sorted by capability surface, not by date.
Each section tells the arc: what emerged, what it replaced, what it became.
The date-sorted history lives in `CHANGELOG_ARCHIVE.md`.

`H(data|epitope) < H(data|date) < H(data)`

---

## Content-Addressed Storage — BLAKE3-keyed object store

Filesystem paths → content-addressed storage with namespace isolation, streaming retrieval, and cross-platform path safety.

- **CAS foundation** (Session 57): NG-1 through NG-4 — content.put/get/resolve with BLAKE3 integrity; namespace on legacy dispatch; streaming retrieval wired in both server paths.
- **Streaming storage parity** (Session 43l→43n): All 3 storage methods wired in semantic router + idle timeout; cross-spring namespace isolation.
- **content.get size guard** (Session 130): Blobs > 64 MiB return `use_streaming: true` instead of inline base64 — prevents unbounded memory on multi-GB objects.
- **Atomic writes** (Session 128): G4 temp-file-then-rename (`.part` → final) eliminates partial-write visibility to concurrent readers.
- **G4 NTFS safety** (Session 128): `validate_path_segment()` rejects reserved chars, trailing dot/space; strict lowercase BLAKE3 hex validation prevents case-sensitivity mismatches.
- **CAS on ZFS** (Wave 155i): Live composition verified — 3,119 CAS objects on ZFS cold tier (25.4TB, 1.56× compression).

---

## Federation & Replication — cross-gate content distribution

Local-only storage → BTSP-secured federation with streaming replication for multi-GB datasets.

- **HTTP content federation** (Session 93): Federation parity + content serving endpoint; HTTP handler dispatch coverage sprint.
- **Federation streaming** (Session 130): `content.replicate`/`content.replicate.pull` use chunked streaming (>16 MiB threshold, 4 MiB chunks); inline base64 below threshold.
- **Remote decoupling** (Session 130): Hardcoded `"forgejo"` preference → `NESTGATE_PREFERRED_REMOTE` env; falls back to first git remote then `"origin"`.
- **G3 BTSP→CAS wiring** (Session 127→128): Replaced `socat`/raw-TCP with `connect_with_btsp` + `JsonRpcClient::call`; cross-gate replication BTSP-secure; external socat dependency eliminated.
- **S3 shadow run** (Session 66): `content.resolve` path normalization for shadow deployment readiness.

---

## ZFS Integration — tiered storage and snapshot lifecycle

ZFS hard-fail on missing kernel → graceful degrade with live pool metrics, snapshot RPC, and cross-gate mesh registration.

- **ZFS backend** (Session 85→86): ZFS integration tests, content streaming, snapshot RPC; cross-gate federation + mesh registration.
- **Stub → real evolution** (Session 129): `should_transition_to_stage` from `const fn → false` to live age/access-pattern logic; `get_all_pool_metrics` and `get_cache_metrics` from `NotImplemented` to live zpool/ARC collection.
- **Fabricated fallback cleanup** (Session 129): Magic disk I/O numbers → honest zeroes; ARC fallback percentages → 0.0; compression fallback → 1.0 (identity).
- **Procfs consolidation** (Session 122→125): 17 scattered `/proc` readsites → `nestgate_platform::linux_proc` delegates; production reads 58→41.

---

## Transport & BTSP — wire protocol evolution

Raw UDS/TCP → TransportEndpoint abstraction, BTSP ClientHello, and riboCipher signal acceptance.

- **Transport Phase 1→2** (Session 97→98): Ecosystem-standard `TRANSPORT_ENDPOINT` adoption; outbound IPC migrated to `connect_transport()`.
- **BTSP ClientHello** (Session 126): Full 7-step outbound wire handshake (P1 Nest Atomic blocker resolved); `JsonRpcClient::connect_btsp_aware()` central entry point.
- **riboCipher signals** (Session 103): Mito-beacon signal acceptance on UDS accept loop.
- **STARTUP-NG-01** (Session 102): Default HTTP in server mode for discovery parity.
- **Android UDS fatal** (Session 105): NESTGATE-ANDROID-01 — UDS fatal on Android, TCP fallback documented.

---

## Security & Authorization — JWT, MethodGate, encrypt-at-rest

Open JSON-RPC → layered auth with method-level gating and native encryption.

- **JH-0 MethodGate** (Session 59): Pre-dispatch Public/Protected classification; 10-method exempt whitelist.
- **PG-56 BTSP auth** (Session 55): Method-level auth gating security fix.
- **JWT NUCLEUS bypass** (Session 52): Phase 3 transport hardening; JWT gate skip when BTSP is auth layer (upstream gap documented).
- **Native encrypt-at-rest** (Session 48): JWT bypass evolution + native encryption; emoji purge from production strings.
- **Auth token evolution** (Session 124): HMAC-SHA256 → `blake3::keyed_hash`; certificate fingerprinting SHA-256 → BLAKE3.

---

## Integrity & Crypto — BLAKE3 consolidation and vendor elimination

Scattered hash algorithms → BLAKE3-only internal crypto surface with pure-Rust TLS.

- **BLAKE3 consolidation** (Session 124): Auth MACs, certificate fingerprints, BTSP key derivation (HKDF → `blake3::derive_key`), storage checksums — all unified; `sha2`/`hmac`/`hkdf` removed from production deps.
- **Vendor elimination** (Session 124): Vendored `rustls-webpki` → published `oxitls-rustcrypto-provider 0.2.1`; zero C/asm; RUSTSEC-2026-0104 resolved; 692KB vendor directory removed.
- **Content trust** (Session 90): `/tmp` evolution; content integrity verification on retrieval paths.
- **Content provenance metadata** (Session 62): Provenance fields on stored content for trio pipeline integration.

---

## Health & Observability — honest metrics and dashboard truth

Fabricated dashboard numbers → live data collection with honest zeroes for unmeasured quantities.

- **HEALTH-01 contract** (Session 102): Bare `{"method":"health"}` returns standard liveness payload.
- **Dashboard honesty** (Session 131): Fabricated metrics (45.0 connections, 1M throughput, 150ms response) → live `statvfs`/`/proc` reads or honest zeroes; metrics cache write-path wired.
- **GAP-038 socket liveness** (Session 119): Connect-probe replaces `Path::exists` for stale socket detection.
- **Dimensional scorecard** (Session 120): 1,710 tests, 0 clippy, 0 debt markers, 0 unsafe, 0 files >800L confirmed.
- **Production unwrap audit** (Session 121): 0 unwrap, 10 justified expect across 14 crates — all annotated with reason strings.

---

## Cross-Platform & Storage Discovery — Windows and NTFS readiness

Unix-only assumptions → platform-aware path resolution, storage markers, and cross-arch validation.

- **Cross-platform paths** (Session 127): `USERPROFILE`/`%ProgramData%`/`%LOCALAPPDATA%` fallbacks; `Path::join` consistency; ZFS binary PATH lookup on Windows.
- **Windows storage marker** (Session 128): `StorageCapabilityMarkerGuard` writes JSON marker instead of symlink (elevation-free).
- **Cross-architecture adoption** (Session 109): Windows check PASS; aarch64-musl segfault fix validated (Session 76).
- **CI-DIV-03** (Session 104): Linker convergence for cross-arch builds.

---

## Neural API & Lifecycle — self-registration and graceful shutdown

Manual startup → `primal.announce`, stale socket cleanup, and content.* transport parity.

- **primal.announce** (Session 70): Neural API self-registration with capabilities and bound socket paths.
- **Stale socket cleanup** (Session 65): `unlink()` before bind + cleanup on shutdown (Wave 22 stadial gate readiness).
- **content.* transport parity** (Session 60): lifecycle.status; namespace isolation on legacy dispatch.
- **Binary UDS compliance** (Session 95): Native `--socket` on all entry paths.
- **Graceful shutdown** (Session 65): SIGTERM/SIGINT handling with socket cleanup.

---

## Architecture & Engineering — codebase evolution (cross-cutting)

Audit-heavy scattered codebase → modular 14-crate workspace with enforced quality gates.

- **Deep debt sweeps** (Session 87→118): Module splits, fake success elimination, visibility tightening, String::from sweeps, production mock evolution.
- **File size discipline** (ongoing): Test extraction keeps production files ≤800L; `content_federation_handlers.rs` 980→797L via test module split.
- **Primal decoupling** (Session 126→128): Hardcoded primal names → capability-based language across 12+ production files; `BiomeOSDirectory` → `EcosystemDirectory`.
- **Dependency hygiene** (Session 120→131): 99 patch bumps; `tokio-util` pruned from nestgate-rpc; `serde/rc` and `tower-http/fs` dead features removed.
- **Coverage sprint** (Session 95b→101): Tier 2 ZFS helpers, route handlers, StorageConfig, content_ops facade — systematic test expansion to 13K+ tests.

---

*Full date-sorted history: `CHANGELOG_ARCHIVE.md`*
*Last compressed: Wave 172*
