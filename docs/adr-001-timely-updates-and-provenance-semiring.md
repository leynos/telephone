# ADR: Timely Updates & Provenance Semantics for Telephone

**Status**: Accepted (Working Spec v0.1) **Date**: 2025‑10‑25 **Decision
Owner**: Telephone architecture group **Related**: DDlog parser/syntax
(ddlint), Telephone compiler & runtime, GPU storage layout, Event‑centric KG
integration

______________________________________________________________________

## Context

Telephone is a GPU‑accelerated Datalog engine that adopts Differential Datalog
(DDlog)‑style *incremental* computation. Our parser is aligned with the
ddlint/DDlog syntax and desugarings (e.g., `-<N>` delay, `'` diff mark,
multi‑head rules), which already appear in Telephone programs and must have
clear runtime meaning during streaming updates.

The engine’s batch core is in place (semi‑naïve evaluation on GPU; WGPU back
end), and we now need a **normative, implementable spec** for:

1. How updates (inserts/retracts) are grouped in *time* and scheduled to
   fixpoint (“timely updates”).
2. How *provenance/weights* propagate and cancel under joins/unions
   (“provenance semiring” behavior) so that retractions and corrections are
   first‑class.

We also operate over an event‑centric knowledge graph (ECKG), where facts are
naturally timestamped events; the update/time model must compose with that
domain.

Finally, GPU memory is finite; the timely/update model must pair with a memory
policy that keeps the hot working set resident and spills/caches cold state
without breaking correctness.

______________________________________________________________________

## Decision

We adopt **Z‑set style differential updates with epoch time**:

- **Logical time** is represented as *epochs* (monotone `u64`). Updates are
  ingested as *transactions* against a target epoch `t`.
- Every physical tuple is represented internally as a triple
  `(tuple, time=t, diff=Δ)` where `Δ ∈ ℤ` (typically `+1` for insert, `-1` for
  delete). We require the additive structure to support cancellation; joins
  multiply, unions add (standard semiring lineage; see §Provenance).
- The **first implementation** supports *micro‑batch* updates with strictly
  increasing epochs per input stream. A *watermark* (per relation) closes an
  epoch and triggers scheduling; late data (arriving with `time < watermark`)
  is rejected in v0.1 (future work adds corrections).
- The parser’s **`Delay -<N>`** on an atom *shifts its logical time to `t+N`*
  during derivation; **diff mark `'`** denotes the *difference* stream of a
  relation at the current epoch (syntactic sugar over weights). Location
  `@expr` is orthogonal to time and preserved as a payload column.
- Scheduling is **incremental semi‑naïve** over *delta relations*: for a rule
  body `A,B,… → R`, in epoch `t` we evaluate `ΔA_t × B_{≤t} ∪ A_{≤t} × ΔB_t …`,
  iterate to fixpoint, then materialize `ΔR_t`.
- **GPU residence**: each relation maintains *three* device buffers: `base`
  (all prior epochs compacted), `delta_in` (incoming epoch `t`), `delta_out`
  (derived for `t`). After convergence, `delta_out` is folded into `base` with
  dedup/cancellation. Cold epochs may be evicted to host and reread on demand
  (see §Memory).

This decision gives us deterministic, DDlog‑like behavior with clear, testable
semantics that map directly to Telephone’s parser and GPU kernel model.

______________________________________________________________________

## Details

### 1) Time & Transactions

**Time unit**: `epoch: u64`. **Transaction**: a batch of `(tuple, Δ)` arriving
for a single `epoch=t`.

**Ingress rules**

- On `begin_epoch(t)`, `delta_in := updates(t)`.
- When all producers for each input relation signal `watermark(t)`, the epoch
  is *sealed*, and scheduling for `t` begins.

**Rule evaluation in epoch `t`**

- For each stratum (per dependency order), evaluate rules using *delta
  products*: `ΔR_t := ⋃ rules r [ r(ΔX_t, Y_{≤t}, …) ∪ r(X_{≤t}, ΔY_t, …) … ]`
  until no new `Δ` occurs.
- Apply **delay**: if a head or body atom has `-<N>`, it contributes to
  `time = t+N`. Deltas are queued into the appropriate future `delta_in` bucket.
- Multi‑head rules are sugar for multiple single‑head rules sharing the same
  body. Locations `@e` are carried as columns; they do not affect epoch routing.

**Commit**

- After fixpoint for epoch `t`, fold `Δ*_t` into `base`, cancelling opposite
  weights, then publish `watermark(t)` to downstream consumers.

**Current limits (v0.1)**

- No out‑of‑order within a stream; late data (`time < current_watermark`) is
  rejected or logged as *correction‑needed*.
- No partial orders on time; a single total‐order epoch clock suffices. Future
  work can model timely dataflow “capabilities/watermarks” per source to permit
  out‑of‑order processing.

### 2) Provenance / Semiring Semantics

We use **Z‑set semantics** by default:

- **Weight domain**: `i64` (an Abelian group under `+`).
- **Operators**:

  - **Union**: `(x, w1) ⊎ (x, w2) = (x, w1+w2)`
  - **Join**: `w = w_left * w_right` where `*` is multiplication in the
    *provenance semiring*; in Z‑set/Boolean, `*` is integer product
    (effectively conjunction).
  - **Projection/Aggregation**: project sums weights for identical projected
    keys; aggregates may specify a semiring/monoid (e.g., `count` uses integer
    +).
- **Retraction**: deletion is `Δ=-1` for the tuple; cancellation happens during
  dedup/compaction.
- **Extension point**: programs may declare an alternate *tag* column to carry
  a user‑chosen semiring (e.g., probabilities), with the default Z‑set still
  driving *existence/cancellation*. Tag propagation (⊕/⊗) follows Lobster‑style
  semiring tags, but does not alter fixpoint/convergence rules.

### 3) Mapping from Syntax to Semantics

- **`Delay -<N>`** (parser): shifts logical time by `+N`. If on body atom, its
  contribution is scheduled for `t+N`; if on head, derived tuple is emitted at
  `t+N`. Range checks use the parser’s `u32` constraint; overflow is a parser
  error.
- **`DiffMark '`**: denotes the *difference* stream of a relation; at runtime,
  this is the `Δ` for the active epoch. It is not a separate storage; it views
  `delta_in`/`delta_out`.
- **`@ location`**: preserved as data; no special timing behavior.

### 4) Scheduler & GPU Execution

- **Per‑relation state (device)**:

  - `base`: SoA columns for all compacted tuples with non‑zero weight (up to
    watermark).
  - `delta_in`: epoch‑t incoming updates (triples held with weights).
  - `delta_out`: newly derived updates in epoch `t`.
- **Kernels**: joins (two‑phase count + scatter), projection, union/dedup
  (sort+unique with weight sum), compaction (filter weight≠0).
- **Convergence**: repeat delta‑products per stratum until `delta_out` empty.
  Then fold `delta_out`→`base`.

### 5) Memory & Hot/Cold Layout

- **Hot set**: most recent compacted epochs + current `delta` stay on GPU.
- **Cold spill**: older epochs are snapshot‑compacted and evicted to host
  (pinned) with *Bloom‑style* guards or min/max key ranges to minimize needless
  swaps; rehydrate on demand for joins that need them. The spill policy is
  purely a **performance layer**; logical results remain identical.

### 6) ECKG Alignment

- ECKG events carry natural time; the *event timestamp* maps to Telephone’s
  `epoch` (via a configurable calendar → epoch function). Temporal reasoning
  rules (e.g., window joins) are expressed through `Delay` and standard
  relational operators, not bespoke temporal syntax (v0.1).

______________________________________________________________________

## Alternatives Considered

1. **Timely Dataflow capabilities from day one**
   *Pros*: rich out‑of‑order & partial orders. *Cons*: higher complexity for
   GPU kernels & scheduling. *Why not now*: v0.1 favors a total‑order epoch; we
   can integrate capabilities later without breaking programs.

2. **Pure Boolean set semantics (no weights)**
   *Pros*: simpler storage. *Cons*: no clean retraction/correction; poor fit
   for differential updates. *Rationale*: Z‑sets are the standard for
   DDlog‑style incremental maintenance.

3. **Processing‑time only (no user time)**
   *Pros*: trivial ingestion. *Cons*: misaligned with ECKG; cannot express
   delayed/event‑time logic. *Rationale*: we standardize on logical event time
   via epochs.

______________________________________________________________________

## Consequences

**Positive**

- Deterministic, cancelable updates with DDlog‑style deltas.
- Clear mapping from Telephone syntax (`-<N>`, `'`, multi‑head) to runtime.
- GPU‑friendly delta iteration with bounded device memory via spill policy.

**Negative / Limits**

- No late data in v0.1; callers must resubmit as a forward correction at a
  future epoch.
- Total‑order epochs only (no partial orders).
- Extra storage for weights & deltas compared to pure‑set materialization.

______________________________________________________________________

## Operational Notes

- **API**:
  `begin_epoch(t)`,`ingest(Rel, rows, t)`,`seal_epoch(t)`,`await_fixpoint(t)`,`commit(t)`.
- **Observability**: per‑epoch counters for in/derived/retracted tuples;
  convergence iterations; GPU spill/rehydrate counts.
- **Testing**: each update batch validated by comparing *incremental result* vs
  *from‑scratch recomputation* on a CPU reference for small fixtures.

______________________________________________________________________

## Open Questions / v0.2 Targets

- Support **late/out‑of‑order** events with per‑source watermarks and bounded
  time skew; adopt capability‑style progress tracking.
- First‑class **probabilistic tags** with a user‑declared semiring, alongside
  Z‑set existence.
- Native **temporal operators** (windows/interval predicates) as syntactic
  sugar over epoch math; consider a minimal temporal library aligned with ECKG
  practices.

______________________________________________________________________

## Appendix A — Worked Example

**Rule with delay and multi‑head (Telephone syntax)**

```ddlog
// Derived at time t+10 due to delay on the head
Alert(u) -<10>, Audit'(u) :-
    Login(u, ip),
    SuspiciousIP(ip).
```

- In epoch `t`, the body contributes `ΔR_{t+10}` (head delay).
- `Alert` emits at `t+10`. `Audit'` is the *diff* stream for `Audit` at `t+10`
  (materialized as a delta).
- If later `SuspiciousIP(ip)` is retracted at `t+12` (`Δ=-1`), the join yields
  a corresponding negative delta that cancels the prior derivation at `t+22`.

______________________________________________________________________

## Appendix B — Parser Alignment Checklist

- `Delay -<N>` → shift epoch by `+N` (u32 range‑checked at parse).
- `'` (diff mark) → exposes per‑epoch delta streams; no separate storage.
- Multi‑head LHS, location `@` → expanded/preserved with no timing effects.
  All conform to ddlint’s grammar and documented early desugarings.

______________________________________________________________________

## Appendix C — Memory Policy Sketch

- Keep `base` for recent epochs on GPU; evict older compacted shards to host.
- During joins, if a shard is absent, fetch & cache with a compact index;
  maintain key‑range summaries to avoid unnecessary reads. This implements
  “impedance matching” between large ECKGs and limited GPU RAM without changing
  logical outcomes.

______________________________________________________________________

**References** — *Differential Datalog Parser & Syntax Specification
(Updated)*, used for `Delay`, diff marks, multi‑head/location semantics. —
*GPU‑Accelerated Datalog with Differential Datalog (DDlog) Semantics in Rust*,
basis for GPU delta evaluation and future timely integration. — *Event‑Centric
Knowledge Graphs: A Deep Dive*, domain alignment for event time and modeling. —
*GPU memory impedance matching*, hot/cold strategy for large graphs vs device
RAM.

______________________________________________________________________

### Acceptance Criteria

- Given a sequence of epochs with inserts and deletes, Telephone’s incremental
  result equals full recomputation.
- `Delay` and diff mark behavior matches this ADR in unit and integration tests.
- Eviction/spill does not affect results; only performance counters change.
