# Hardware Validation Log — reference machine (XMG Neo 16 A25)

AMD Ryzen 9 9955HX3D · RTX 5090 Laptop · Windows 11

## 2026-06-15 — GPU power range (nvidia-smi)

- `nvidia-smi -q -d POWER`: Current 175 W, Default 95 W, **Min 5 W, Max 175 W**.
- GPU power limit is software-adjustable downward (5–175 W). 175 W is firmware-capped.
- **Verdict:** GPU power control + telemetry confirmed via NVML/nvidia-smi.

## 2026-06-15 — CPU power write (RyzenAdj v0.19.0)

- `ryzenadj.exe` recognizes `CPU Family: Fire Range`. Monitoring table init **fails**
  (`request_table_ver_and_size is not supported on this family`).
- Setters **succeed**: `Successfully set stapm_limit`, `slow_limit`, `fast_limit`.
- Exe crashes `0xC0000005` on **teardown** (touches the unsupported monitoring table) —
  cosmetic, after the write lands.
- **Verdict:** CPU power *write* works. Plan: call libryzenadj via FFI, only the setters,
  never the monitoring init → avoids the crash. (FFI confirmation = Task 11, pending.)

## 2026-06-15 — CPU telemetry availability (Task 12 spike, part 1)

Open question: can we read CPU power/temp/clocks on Fire Range (RyzenAdj's reader is dead)?

- Captured HWiNFO64 log (`Documents/log.CSV`), ~20 s, light load (agent running).
- **CPU Package Power [W]:** 15.7–30.2, varying with load.
- **CPU (Tctl/Tdie) [°C]:** 57.8–61.0.
- **Core Clocks (avg) [MHz]:** 1523–2779, varying.
- **GPU Rail Powers (avg) [W]:** ~2.3 (GPU idle, expected).
- **Verdict:** All required telemetry **is available** on this silicon. The HWiNFO
  shared-memory fallback is therefore a guaranteed-viable telemetry source. The biggest
  M1 risk is retired.

### Remaining decision (Task 12 part 2 — not blocking)
Telemetry implementation path:
- **A · HWiNFO shared memory** — proven data source, but adds a runtime dependency on
  HWiNFO running with SHM enabled. Against the "lightweight, self-contained" goal.
- **B · Direct ring0 MSR reads** — our own reader of the AMD RAPL MSRs
  (`0xC0010299` unit, `0xC001029B` pkg energy) + Tctl. No external dependency, fits the
  vision; costs a signed-driver setup + decode work.
- **Recommendation:** target B (self-contained), keep A documented as the fallback. B is a
  later HW sub-task; the project is unblocked either way.
