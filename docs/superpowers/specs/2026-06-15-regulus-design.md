# regulus — Design Specification

- **Status:** Draft (pending review)
- **Date:** 2026-06-15
- **Author:** suwandre
- **Reference machine:** XMG Neo 16 A25 — AMD Ryzen 9 9955HX3D ("Fire Range"), NVIDIA RTX 5090 Laptop GPU, 96 GB RAM, Tongfang chassis, Windows 11

## 1. Vision

A lightweight, performant, minimalist control center for laptop **power, thermals, and performance** — a leaner, feature-richer alternative to vendor tools like XMG Control Center.

Positioning: **universal laptop control center, starting with XMG.** The app runs on any machine and exposes whatever that machine actually supports, degrading gracefully. It is *not* a promise that every feature works everywhere — that is physically impossible (see §3). The first shipped backend targets the reference machine; the architecture makes every additional backend additive, never a rewrite.

## 2. Goals / Non-Goals

### Goals
- Set CPU power limits (STAPM / sPPT / fPPT) and GPU power limit (TGP).
- A software-enforced **combined CPU+GPU budget** (Steam Deck-style), with per-domain control as the alternative.
- Live telemetry (CPU + GPU watts / temps / clocks).
- Named profiles (Quiet / Balanced / Beast / custom), persisted.
- Re-apply the active profile automatically on boot and on wake (limits reset otherwise).
- Extremely low idle footprint (target: well under vendor tooling).
- Hardware Abstraction Layer enabling future platforms/vendors/chassis as drop-in backends.

### Non-Goals (v1)
- Controlling RAM power, SSD power, or arbitrary peripheral power — these are not runtime-controllable knobs (see §3).
- Capping *total wall power* — uncontrollable overhead (RAM, panel, VRM losses) makes this impossible. We cap CPU+GPU only.
- macOS power-limit control — not possible (see §3).
- Backends for non-reference hardware (Linux, Intel, AMD GPU, other OEMs) — specified as roadmap, not built in v1.

## 3. Hardware Reality (Hard Constraints)

There is **no universal power-control API.** Each axis has a distinct interface, and some have none:

| Axis | Interface | v1? |
|------|-----------|-----|
| Windows + AMD CPU | RyzenAdj/SMU via ring0 driver | ✅ |
| Windows + Intel CPU | MSR RAPL (`0x610`), XTU-style | roadmap |
| Linux | `/sys` powercap RAPL, ryzenadj-linux, NVML, `acpi_call` (root + sysfs, no driver) | roadmap |
| macOS Intel | mostly locked | telemetry only |
| macOS Apple Silicon | **locked by Apple — no power-limit API** | telemetry only |
| NVIDIA GPU | NVAPI (Windows) / NVML / nvidia-smi | ✅ |
| AMD GPU | ADL / amdgpu sysfs | roadmap |
| Intel Arc GPU | distinct | roadmap |
| Chassis (OASIS, MUX, fans) | per-OEM EC, reverse-engineered | XMG only (v1); each new OEM = its own RE effort |

**Verified on the reference machine (2026-06-15):**
- CPU: RyzenAdj v0.19.0 recognizes "Fire Range" and **successfully sets** `stapm_limit`, `slow_limit`, `fast_limit` (SMU accepted all three). The exe crashes (`0xC0000005`) on *teardown* while touching the unsupported monitoring table — cosmetic, post-write. Mitigation: call `libryzenadj` via FFI and avoid the monitoring-init path entirely.
- CPU telemetry: RyzenAdj monitoring table is **unsupported on Fire Range** — no readout via that path. Telemetry must come from direct MSR reads (see §5) or a fallback source.
- GPU: `nvidia-smi` reports power limit adjustable **5–175 W** (default 95, max 175). Downward control works directly. The 175 W ceiling is firmware-capped; software cannot exceed it (a 250 W mod requires a physical shunt + modified vBIOS — out of scope).

## 4. Architecture

A **single elevated process**, launched at login via Windows Task Scheduler (one UAC consent, ever; always running → covers reapply + telemetry). Internally modular so milestones plug in without rewrites.

```
┌─────────────────────────────────────────────┐
│ regulus (single elevated process)            │
│                                              │
│  UI layer (Slint)                            │
│   ├─ quick-panel (tray)  — budget-first (A)  │
│   └─ window             — independent (B)    │
│        renders ONLY supported capabilities   │
│              │ (in-process channel)          │
│  Engine                                      │
│   ├─ Capability model (runtime detection)    │
│   ├─ Backend registry                        │
│   ├─ Modules: power, telemetry [v1];         │
│   │           oc, display, stats, cooler […] │
│   └─ HAL traits ↓                            │
│        PowerControl / Telemetry / Cooler /   │
│        Display / Overclock                   │
│              │                               │
│  Backends (implement HAL traits)             │
│   └─ v1: Windows·x86·AMD·NVIDIA·XMG-Tongfang │
│        ├─ CPU power  → libryzenadj (FFI)      │
│        ├─ CPU telem  → ring0 MSR reads        │
│        ├─ GPU        → NVAPI / nvidia-smi     │
│        └─ chassis    → XMG/Tongfang EC + OASIS│
└─────────────────────────────────────────────┘
```

### Hardware Abstraction Layer (HAL)
- **Capability traits:** `PowerControl`, `Telemetry`, `Cooler`, `Display`, `Overclock`.
- **Runtime detection** builds a **capability model** of the current machine (OS × CPU vendor × GPU vendor × chassis).
- **Backend registry:** detection selects and loads matching backends; the engine composes them.
- **Capability-driven UI:** Slint renders only features the loaded backends support. Everything else is hidden.

### Modules (v1: `power`, `telemetry`)
Each module is a Rust module behind a trait, depending only on HAL trait objects — mockable for tests, swappable per backend.

### Combined budget policy
- Software policy over two independent domains: enforce `cpu_cap + gpu_cap ≤ B`.
- These are **caps, not live draw.** Actual draw ≤ sum of caps.
- v1: **static split** driven by a master watt value + CPU↔GPU bias.
- **Dynamic rebalancing** (shift idle CPU budget to GPU via a feedback loop) is a later milestone — requires solid telemetry first.

### Profiles & persistence
- Profiles: Quiet / Balanced / Beast + custom. Stored as TOML in `%APPDATA%\regulus`.
- **Reapply** the active profile on boot (login task) and on wake (`WM_POWERBROADCAST` / power-event hook).

## 5. CPU Telemetry (Key Risk)

RyzenAdj cannot read telemetry on Fire Range. Plan: read AMD RAPL-style MSRs directly through the same ring0 driver used for hardware access:
- Energy: `MSR_RAPL_PWR_UNIT 0xC0010299`, `MSR_PKG_ENERGY_STAT 0xC001029B`, `MSR_CORE_ENERGY_STAT 0xC001029A` — power = energy delta / interval.
- Temp: SMN / Tctl read. Clocks: `APERF`/`MPERF` or P-state status MSRs.
- **Fallback** if direct MSR reads prove unreliable on this silicon: read HWiNFO64 shared memory, or bundle LibreHardwareMonitor's sensor layer.
- GPU telemetry comes free from NVAPI / nvidia-smi.

This is the **first spike** in M1 — validate before building UI on top of it.

## 6. Performance Requirements

Performance comes from architecture, not framework choice:
1. **No render when hidden** — when the panel/window is closed, the render loop stops entirely. Only the telemetry poll runs.
2. **Reactive repaint** — Slint repaints only on data change (chosen over immediate-mode for idle efficiency).
3. **Decoupled telemetry** — engine polls hardware at 1–2 Hz; UI reads cached samples. Never poll the driver at frame rate.
4. **Software renderer first** — default to Slint's CPU renderer (no GPU context); switch to GPU (femtovg/skia) only if graph animation feels rough.
5. **Release tuning** — `lto = true`, `opt-level = 3` (or `z`), `panic = "abort"`, `strip = true`.
6. **Lazy engine** — sleeps between polls; event-driven on power events.

Target: ~15–40 MB idle RAM, ~0% CPU with panel closed.

## 7. Security / Driver Considerations

- Ring0 hardware access is required (MSR reads, SMU mailbox). The common `WinRing0` driver is **flagged by Windows Defender** (CVE-2020-14979, blocklisted unsigned driver).
- For any public release, select a **signed ring0 driver** (signed WinRing0 fork, or libryzenadj's bundled driver for writes plus a signed MSR reader). Resolve before distribution. For personal use, the bundled driver is acceptable.
- The app requires elevation; the login Task Scheduler entry holds it so users consent once.

## 8. UI Surfaces

- **Quick-panel (tray click):** budget-first layout (A) — master watt + bias slider, live telemetry, profile buttons. Closes on click-away. The daily driver.
- **Full window:** independent layout (B) — separate CPU/GPU controls + soft combined cap that warns/clamps, full telemetry with graphs, left-nav listing modules (roadmap items badged). The precision surface.
- Both are **capability-driven** — only supported controls render.
- Minimalist Slint theme: generous spacing, one accent color, clean typography.

## 9. Testing Strategy

- **Pure logic** (budget-split math, profile serialization, capability model, backend selection) → unit-tested, no hardware.
- **Hardware I/O** behind HAL traits → mocked in tests.
- **Manual hardware-validation harness** — the existing probe scripts (`tdp-probe.ps1`, `tdp-probe2.ps1`) are the seed; expand into a checklist run on the reference machine for `[HW]` tasks.

## 10. Milestones

| Milestone | Scope |
|-----------|-------|
| **M1** | Modular core + HAL + capability detection + `power` + `telemetry` + profiles + reapply + both UIs. v1 backend: Windows·x86·AMD·NVIDIA·XMG-Tongfang (incl. OASIS + XMG-specific unlocks). **The runnable app.** |
| **M2** | GPU overclock/undervolt via NVAPI (clock/voltage curve). |
| **M3** | Display module: auto refresh-rate, brightness; MUX / dGPU-only / Optimus switching (EC reverse-engineering gated). |
| **M4** | Stats showcase + ranking against comparable laptops. |
| **M5** | OASIS watercooler (USB protocol RE) + CPU Curve Optimizer / undervolt (RE-gated). Dynamic combined-budget rebalancing. |
| **Roadmap backends** | Linux, Windows·Intel (MSR RAPL), AMD GPU (ADL/sysfs), Intel Arc, other OEM chassis, macOS (telemetry-only). |

## 11. Build Methodology

- Tasks tagged **`[AUTO]`** (testable without hardware — HAL, UI, logic) or **`[HW]`** (requires the reference machine).
- `[AUTO]` executed via an autonomous builder→review→repair loop (Ralph for build/test/repair + a review gate). `[HW]` executed manually, human-in-the-loop, validated on the real machine. Autonomous loops never validate hardware tasks.

## 12. Open Questions

1. Do direct MSR reads return reliable power/temp/clocks on Fire Range, or is the HWiNFO/LHM fallback needed? (M1 spike)
2. Which signed ring0 driver for distribution?
3. Which XMG/Tongfang EC registers expose chassis controls (fans, OASIS, MUX, XMG-specific unlocks), and how are they read/written? (RE effort)
4. Can the RTX 5090 Laptop reach its full 175 W via NVAPI on this chassis, or is it arbitration-capped lower?
5. Does `libryzenadj` via FFI set limits cleanly without the teardown crash when we control the call sequence?
