# Position paper: A pliron + egg layer for MLIR‑style abstraction and DAG‑based computation caching in **Telephone**

## Executive summary

**Telephone**—a GPU‑accelerated, DDlog‑semantics Datalog engine—already has a
clear path to fast joins on WGPU/CUDA, differential updates, and an
event‑centric workload model. What it lacks (by design, so far) is a portable,
*structured* intermediate layer that (1) keeps the logical meaning visible long
enough to do serious rewrites, (2) abstracts over multiple backends, and (3)
gives us stable, canonical forms suitable for **change‑aware compilation
caching**.

We propose a Rust‑native, MLIR‑shaped layer built from:

- **pliron** for a typed, dialect‑extensible IR (ops, regions, attributes,
  verifiers) in safe Rust;
- **egg / egglog** for equality‑saturation and declarative rewrite rules to
  derive canonical, DAG‑friendly plans; and
- a thin adapter to *emit* (not depend on) an MLIR dialect later (via melior)
  once backends or interop require it.

This layer sits between the DDlog parser and the GPU codegen/runtime. It
preserves DDlog‑level structure (relations, rules, deltas, fixpoints,
provenance tags), enables principled rewrites and scheduling, and surfaces
**canonical plan hashes** that key a three‑tier cache: plan → code → results.
It resolves the “should we adopt MLIR now?” contention by being
**MLIR‑compatible** in shape while staying Rust‑native today.

The proposal is consistent with the existing Telephone design (GPU operators,
incremental updates, timely‑friendly future) and its event‑centric target
workloads.

______________________________________________________________________

## Background and goals

**What exists.** The current design specifies: DDlog‑flavoured frontend,
semi‑naïve/differential updates, GPU kernels for
joins/filter/project/aggregate/union, and a host‑orchestrated fixpoint loop. It
targets WGPU for CUDA/Metal portability and leaves the door open to timely
dataflow later.

**Why we need a mid‑level abstraction.** Our workloads are *event‑centric* and
streaming, where rule sets evolve and small fact deltas should trigger
*minimal* recomputation. A structured IR makes it possible to: (i) retain
semantic information (delta/fixpoint/provenance) for correctness‑preserving
rewrites; (ii) abstract over backends (CPU, GPU, distributed); and (iii)
introduce deterministic **plan canonicalisation** so that we can cache compile
artifacts and invalidate them precisely when rules or statistics change. These
needs are amplified in event‑centric knowledge graphs, which are dynamic by
nature.

______________________________________________________________________

## Proposal in one diagram (textual)

```
DDlog source  ──► Parser (per ddlint spec) ──► tel.ir (pliron dialect)
   │                                        ▲          │
   │                                        │          │  egg ruleset:
   ▼                                        │          │    - join assoc/comm
(Desugar: group_by, head refs, etc.)        │          │    - pushdown
   (kept as typed ops + attrs)              │          │    - delta distribution
                                            │          ▼
                                       Canonical plan extraction
                                            │   (cost + verifier)
                                            ▼
                                  PlanHash + LogicalPlan
                                    │                 │
                                    │                 └─► Code cache key (target/ABI salted)
                                    ▼
                          Backend lowerings (GPU/CPU/async)
                                    │
                                    ▼
                              Runtime (deltas/fixpoints)
```

Parser conformance and early desugarings (e.g., `group_by` extraction, by‑ref
heads) are preserved and *typed* in the IR for later passes.

______________________________________________________________________

## IR design (pliron dialect): **`tel`**

**Core ops**

- `tel.relation(name, schema, role=input|output|internal, kind=relation|stream|multiset, attrs={keys,…, semiring,…})`
- `tel.rule(heads: Region<tel.atom>, body: Region<tel.term>)`
- **Relational algebra**:
  `tel.join(lhs, rhs, on)`,`tel.filter(inp, pred)`,`tel.project(inp, cols)`,`tel.aggregate(inp, group_by, agg)`
- **Differential**: `tel.delta(inp)`, `tel.union_set(lhs, rhs)` (dedup
  semantics), `tel.fixpoint(region)` with SCC boundaries
- **Adornments**: `tel.delay(n)`, `tel.diffmark`, `tel.locate(expr)`; by‑ref
  heads keep a typed `ref_new` node per the parser spec.

**Attributes & types**

- `Semiring` attribute (Boolean initially; extensible to probabilistic/weighted
  tags), propagated through ops to preserve provenance semantics end‑to‑end.
- Key/FD metadata and index hints to make rewrites cost‑aware and safe.
- Verified stratification and SCC formation to scope recursion and delta rounds.

This mirrors Telephone’s current logical model while retaining enough structure
to guide correct lowerings to GPU/CPU backends.

______________________________________________________________________

## Rewrite & canonicalisation (egg / egglog)

**Rule families (guarded by metadata):**

- Join **commutativity/associativity** under key/FD constraints (avoid
  exploding cross joins).
- **Selection / projection pushdown** with nullability and semiring‑safety
  checks.
- **Aggregation** normalisation (e.g., SUM/COUNT forms), regrouping when keys
  justify it.
- **Delta distribution** for semi‑naïve: push `delta` through algebra;
  eliminate redundant deltas.
- **Predicate normalisation** (CNF/DNF as needed for index matching).

**Extraction & hashing**

- An egg `Analysis` computes (per e‑class) a **type/semiring summary**, an
  estimated **cost**, and a **digest**.
- Deterministically extract the min‑cost representative; serialise to a
  **normalised textual form**; compute
  `PlanHash = BLAKE3(normalised_text ∥ engine_abi ∥ cost_model_version ∥ target_triple ∥ index_stats_fingerprint)`.
- Salted hashes avoid “cache hits” that are ABI‑incompatible or
  cost‑model‑inappropriate.

This delivers **stable, DAG‑friendly** representations you can hash and
cache—an approach already proven useful in other DAG pipelines (idempotent
identifiers, blob indirection, and graph‑shaped orchestration).

**Guard‑rails**

- Rewrite budgets to cap e‑graph growth; SCC‑scoped saturation (no cross‑SCC
  rule motion) to preserve monotonicity and fixpoint safety.
- Verifiers ensure semiring correctness and stratification post‑rewrite.

______________________________________________________________________

## Caching and invalidation

**Three tiers (keys derived from `PlanHash`):**

1. **Plan cache**: `PlanHash → LogicalPlan` (post‑extraction `tel` plan).
2. **Code cache**: `PlanHash ⊕ Target → CompiledFragment` (GPU/CPU kernels,
   fused loops).
3. **Result cache (optional, sharded)**:
   `(PlanHash, InputSnapshots, Epoch) → MaterialisedChunk`, where
   `InputSnapshots` are content hashes over base relations’ current
   shards/journals.

**Invalidation**

- *Code edit*: re‑parse → rewrite → extract → compare `PlanHash`. If unchanged,
  reuse plan+code; if changed, invalidate dependants via reverse‑topo walk of
  the rule DAG.
- *Data change*: compute delta snapshots; invalidate only result shards whose
  `InputSnapshots` intersect; **plan+code caches survive**—critical for
  streaming/event‑centric graphs.

______________________________________________________________________

## Backend abstraction and lowering

The pliron dialect does not replace Telephone’s GPU IR; it **feeds** it.

- **GPU lowering**: map `tel.join`/`filter`/`project`/`aggregate` to existing
  kernels and fusion passes; batch or stream deltas per Telephone’s runtime.
- **Alternative targets**: enable a CPU JIT path (LLVM), a future
  async/streaming runtime, or a formal analysis/export path—without changing
  the front‑end or rewrites.
- **MLIR interop (future)**: provide an *exporter* to a `telephone` MLIR
  dialect via melior, unlocking the MLIR ecosystem when needed (bufferisation,
  GPU/NVVM, CIRCT, etc.), while keeping core compilation Rust‑native until
  semantics stabilise.

______________________________________________________________________

## How this resolves current contentions

**“Why not adopt MLIR outright?”** MLIR brings powerful infrastructure but at
the cost of a C++ stack and early lock‑in of evolving semantics. The pliron+egg
layer is **MLIR‑shaped** yet Rust‑native, giving us rapid iteration, tight
integration with the existing runtime, and an **easy exit** to MLIR later
through an exporter.

**“Will an extra IR hurt performance?”** No. The layer is compile‑time only. It
reduces total work by increasing reuse (plan/code caches), limiting
recompilations to changed sub‑DAGs, and guiding better fusions. Runtime hot
loops remain the same GPU kernels already described.

**“Does it complicate DDlog semantics?”** We encode DDlog adornments (diff
marks, delays, multi‑head rules, ref heads) and the parser’s desugarings as
*typed* IR nodes with verifiers. This makes semantic intent explicit and
checkable, not brittle.

**“Is this compatible with the event‑centric KG focus?”** Yes. Event‑centric
loads depend on fast, incremental maintenance and small code edits over
long‑lived programs. Canonical plans plus snapshot‑keyed caches cut churn
dramatically for these streaming scenarios.

______________________________________________________________________

## Worked micro‑example (conceptual)

Source:

```
CoPresent(p1, p2) :-
  Happened(p1, _, loc, t),
  Happened(p2, _, loc, t),
  p1 != p2.
```

- Parser desugars and types; IR builds two keyed joins on `(loc,t)`, plus
  filter `p1!=p2`.
- egg saturates: pushes filter before second join; applies join commutativity
  to pick an indexed side; yields a canonical `tel` plan; we compute `PlanHash`.
- On new `Happened'` deltas, delta distribution transforms the plan to only
  join `ΔHappened` with base `Happened` (and symmetrically), preserving
  semi‑naïve work bounds. Codegen reuses compiled kernels; only result shards
  with affected snapshots recompute.

______________________________________________________________________

## Engineering plan

**Phase A (2–3 sprints).**

- Define `tel` dialect (types/ops/attrs/verifiers) in pliron.
- Implement minimal egg ruleset (pushdown, join assoc/comm, delta distribution)
  with budgets.
- Deterministic extraction + `PlanHash`.
- Plan+code caches behind feature flags.
- Lowering to existing GPU pipeline for two kernels (join, filter).

**Phase B.**

- Expand rewrites (aggregation, projection pruning).
- SCC‑scoped fixpoint regions; provenance/semiring verifier.
- Result‑cache shards keyed by input snapshots (content hashes over relation
  journals) and epoch.

**Phase C.**

- Optional MLIR exporter (melior) to a `telephone` dialect.
- Cost model informed by index stats and target fingerprints; expose in
  `PlanHash` salt for stable caching under configuration drift.

______________________________________________________________________

## Risks & mitigations

- **E‑graph blow‑up** → enforce node/iteration caps; staged rewrites;
  cost‑guided patterns.
- **Non‑confluent patterns** → deterministic extraction order and tie‑breakers;
  verifier gates.
- **Cache key flapping** (e.g., stats change) → salt `PlanHash` with *versions*
  and *stats fingerprints*; keep reuse high across benign edits.
- **Semantic drift** → keep parser desugarings and IR verifiers aligned; use
  golden tests from the parser spec.

______________________________________________________________________

## Conclusion

A **pliron + egg** mid‑layer gives Telephone the **structured semantics**,
**backend portability**, and **deterministic canonicalisation** it
needs—without derailing the current GPU‑first path. It complements the existing
design (incremental updates, event‑centric focus, WGPU kernels) and provides
the practical machinery for **DAG‑based computation caching** that persists
across small code and data changes. If/when we need full MLIR interop, an
exporter completes the bridge.

This is the smallest change that unlocks the most leverage: faster iteration
now, robust reuse later, and a clean runway to multiple backends and ecosystems.

______________________________________________________________________

### Sources from the project corpus

- **GPU‑Accelerated Datalog with DDlog Semantics in Rust** — architecture, GPU
  kernels, incremental updates, and timely integration path.
- **Event‑Centric Knowledge Graphs: A Deep Dive** — workload characteristics
  motivating incremental maintenance and low‑churn recompilation.
- **Architectural Blueprint for a High‑Performance Email Intelligence Pipeline
  in Rust** — DAG modeling, idempotent hashing, and staged, cache‑aware
  pipelines (patterns adapted here).
- **Differential Datalog Parser & Syntax Specification (Updated)** — concrete
  syntax, desugarings, and head/body adornments reflected as typed ops/attrs in
  `tel` dialect.

______________________________________________________________________

### Appendix: selected `tel` op → backend mapping

| `tel` op                            | Backend strategy                                                                                           |
| ----------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| `tel.join(lhs,rhs,on)`              | sort‑merge/hash join kernels; two‑phase (count→prefix‑sum→emit); fuse with projection when safe.           |
| `tel.filter(inp,pred)`              | per‑tuple kernel; predicate pushdown before join when verified.                                            |
| `tel.project(inp,cols)`             | copy/map kernel; opportunistically fused.                                                                  |
| `tel.aggregate(inp, group_by, agg)` | segmented reductions; atomics or sort+reduce by key depending on cardinality.                              |
| `tel.delta(inp)`                    | marks incremental path; distributes through algebra via rewrites to limit work.                            |
| `tel.fixpoint(region)`              | host‑orchestrated iteration per SCC; GPU kernels per round; differential deltas drive convergence.         |
