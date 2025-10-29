# Telephone development roadmap

This roadmap assumes the ddlint-aligned parser in
`docs/telephone-design.md:1603-1667` is complete and that all downstream work
remains outstanding. The plan converts the design specification into a
sequenced set of phases, workstreams, and measurable tasks. Each task lists the
relevant design reference so progress can be audited directly against the
source document. Timeframes are omitted; sequencing follows dependency order.

## Phase 1: Canonical planning pipeline

### Step: Define the `tel` intermediate representation

- [ ] Model relations, rules, delta views, fixpoint regions, and parser
  adornments as typed ops with verifiers in a pliron dialect
  (`docs/telephone-design.md:1743-1759`).
- [ ] Encode semiring attributes, stratification checks, and delay/diff metadata
  so invalid programs fail IR validation
  (`docs/telephone-design.md:1749-1759`, `docs/telephone-design.md:2690-2699`).
- [ ] Provide round-trip tests that lower representative DDlog programs to IR
  and assert structural stability via content-addressable snapshots
  (`docs/telephone-design.md:1743-1768`). Validation: Use deterministic
  fixtures from Appendix A7 and smoke checks from Appendix A9 to seed snapshot
  baselines
  (`docs/telephone-design.md:3855-3872`, `docs/telephone-design.md:3901-3915`).

### Step: Introduce equality-saturation planning

- [ ] Integrate egg/egglog with bounded rewrites for join associativity,
  predicate pushdown, and delta distribution
  (`docs/telephone-design.md:1754-1759`).
- [ ] Implement deterministic plan extraction with a cost heuristic and publish
  salted plan hashes for downstream caches
  (`docs/telephone-design.md:1749-1768`).
- [ ] Add regression fixtures to guarantee canonical plans remain stable across
  identical inputs and configuration tweaks
  (`docs/telephone-design.md:1754-1779`). Validation: Rebuild the EventKG,
  GDELT, and ICEWS corpora via Appendix A3–A5 and re-run `make validate-data`
  to confirm fixture parity
  (`docs/telephone-design.md:3735-3841`, `docs/telephone-design.md:3901-3915`).

### Step: Wire plan, kernel, and result caches

- [ ] Stand up a three-tier cache storing logical plans, compiled kernels, and
  optional result shards keyed by `PlanHash`
  (`docs/telephone-design.md:1770-1778`).
- [ ] Salt cache keys with target ABI fingerprints and statistics digests to
  prevent cross-environment reuse issues (`docs/telephone-design.md:1770-1778`).
- [ ] Exercise invalidation on rule edits, statistics drift, and backend changes
  via automated tests (`docs/telephone-design.md:1770-1781`). Validation: Prime
  caches with the Appendix A datasets after completing the manifest and smoke
  checks to observe invalidation triggers
  (`docs/telephone-design.md:3693-3915`).

### Step: Extend the GPU IR compiler for canonical plans

- [ ] Adapt the GPU IR compiler to consume canonical plans, retaining memory
  layout optimisation passes (`docs/telephone-design.md:1620-1667`).
- [ ] Emit CUDA and SPIR-V kernel bundles that align with cache identities and
  expose instrumentation hooks for later profiling
  (`docs/telephone-design.md:358-374`, `docs/telephone-design.md:2812-2833`).
- [ ] Validate plan-to-kernel lowering across at least two GPU vendors with
  deterministic outputs
  (`docs/telephone-design.md:358-374`, `docs/telephone-design.md:2812-2833`).
  Validation: Replay the Appendix A synthetic and curated datasets through the
  GPU pipeline to confirm deterministic kernels
  (`docs/telephone-design.md:3735-3872`, `docs/telephone-design.md:3901-3915`).

## Phase 2: Incremental execution core

### Step: Implement epoch-based ingestion and scheduling

- [ ] Expose host APIs for `begin_epoch`, `ingest`, `seal_epoch`, and
  `await_fixpoint`, enforcing monotone `epoch: u64` ordering
  (`docs/telephone-design.md:2581-2588`).
- [ ] Instrument telemetry for late-data rejection, epoch throughput, and
  seal-to-commit latency (`docs/telephone-design.md:2585-2606`).
- [ ] Provide ingestion integration tests comparing incremental execution
  against from-scratch recompute across insert and retract cases
  (`docs/telephone-design.md:2585-2599`). Validation: Run the ingestion suite
  on the Appendix A EventKG, GDELT, and ICEWS releases after
  `make validate-data`
  (`docs/telephone-design.md:3735-3841`, `docs/telephone-design.md:3901-3915`).

### Step: Deliver delta iteration and cancellation semantics

- [ ] Implement semi-naïve delta products per stratum and terminate when no new
  deltas remain
  (`docs/telephone-design.md:2402-2458`, `docs/telephone-design.md:2589-2591`).
- [ ] Fold `delta_out` buffers into compacted base storage, summing integer
  weights and deleting cancelled tuples
  (`docs/telephone-design.md:2410-2421`, `docs/telephone-design.md:2595-2599`).
- [ ] Extend automated tests to assert fixpoint iteration counts and weight
  cancellation invariants (`docs/telephone-design.md:2402-2458`). Validation:
  Exercise the tests with the curated incremental fixtures and synthetic
  generators from Appendix A7 after smoke checks
  (`docs/telephone-design.md:3735-3872`, `docs/telephone-design.md:3901-3915`).

### Step: Support parser adornments at runtime

- [ ] Honour `Delay -<N>` by scheduling derived tuples into future epochs and
  queueing deltas without duplicate materialisation
  (`docs/telephone-design.md:2592-2594`).
- [ ] Surface diff-mark views by exposing read-only handles over `delta_in` and
  `delta_out` storage (`docs/telephone-design.md:2592-2599`).
- [ ] Verify multi-head rule expansion and `@location` payload retention in
  parser-to-runtime contract tests (`docs/telephone-design.md:2592-2599`).
  Validation: Use Appendix A synthetic datasets to cover multi-head paths and
  confirm payload retention following `make validate-data`
  (`docs/telephone-design.md:3855-3872`, `docs/telephone-design.md:3901-3915`).

### Step: Build GPU memory residency management

- [ ] Provide per-relation device buffers for `base`, `delta_in`, and
  `delta_out`, including allocation, reuse, and teardown hooks
  (`docs/telephone-design.md:2581-2603`, `docs/telephone-design.md:2320-2336`).
- [ ] Implement spill and rehydrate flows for cold epochs using the eviction
  policies in `docs/telephone-design.md:2338-2398`.
- [ ] Emit operational metrics for spill counts, rehydrate latency, and
      protected
  shard usage
  (`docs/telephone-design.md:2338-2398`, `docs/telephone-design.md:3081-3085`).

### Step: Finalise reference counting and deletion handling

- [ ] Maintain per-tuple reference counts to gate deletions and generate
      negative
  deltas safely (`docs/telephone-design.md:2453-2460`).
- [ ] Optimise storage strategy by selecting dense, hash, bitmap, or RLE
  layouts according to relation characteristics
  (`docs/telephone-design.md:2463-2474`).
- [ ] Add negative-delta regression tests covering shared derivations and staged
  deletions (`docs/telephone-design.md:2453-2474`). Validation: Reproduce the
  regression scenarios with Appendix A synthetic and incremental fixtures after
  smoke checks to confirm parity
  (`docs/telephone-design.md:3735-3872`, `docs/telephone-design.md:3901-3915`).

## Phase 3: Streaming ingestion and query capabilities

### Step: Implement high-throughput event ingestion

- [ ] Deliver protocol adapters for Kafka, Pulsar, REST/gRPC, file, and CDC
  sources with the throughput targets in `docs/telephone-design.md:2957-2969`.
- [ ] Build the ingestion pipeline with schema validation, deduplication, batch
  coordination, compression, and dead-letter queues
  (`docs/telephone-design.md:2971-3004`).
- [ ] Provide configurable retry, rate limiting, and circuit breaker policies
  for error handling
  (`docs/telephone-design.md:2971-3004`, `docs/telephone-design.md:3003-3004`).

### Step: Manage schemas and flow control

- [ ] Integrate a schema registry client supporting evolution scenarios per
  `docs/telephone-design.md:3006-3037`.
- [ ] Implement multi-level backpressure and flow control thresholds with
  monitoring hooks (`docs/telephone-design.md:3039-3084`).
- [ ] Deliver out-of-order handling using reorder buffers and watermark timers
      as
  shown in `docs/telephone-design.md:3087-3118`.

### Step: Materialise event-centric knowledge graphs

- [ ] Introduce temporal ordering guarantees (none, per-key, per-source, global)
  selectable per workload (`docs/telephone-design.md:3089-3099`).
- [ ] Implement windowing helpers over epoch arithmetic for sliding and tumbling
  analyses
  (`docs/telephone-design.md:376-385`, `docs/telephone-design.md:2581-2594`).
- [ ] Add automated tests that stitch ingestion, ordering, and incremental
  updates for event-centric rules (`docs/telephone-design.md:2953-3120`).

### Step: Deliver query execution and caching

- [ ] Build the query processing engine covering point, range, aggregation, and
  join workloads with GPU-backed operators
  (`docs/telephone-design.md:2611-2624`).
- [ ] Maintain GPU-resident indices (hash, sorted, bitmap, spatial) and their
  incremental rebuild strategies (`docs/telephone-design.md:2657-2687`).
- [ ] Implement semantic result caching, invalidation triggers, and freshness
  checks per `docs/telephone-design.md:2731-2745`.

## Phase 4: Multi-backend runtime and optimisation

### Step: Harden the GPU abstraction layer

- [ ] Implement trait-based abstractions for device queries, context creation,
  memory allocation, and kernel launch across CUDA, SPIR-V, Metal, and OpenCL
  (`docs/telephone-design.md:2743-2793`, `docs/telephone-design.md:2812-2833`).
- [ ] Provide feature detection and capability registries so plan lowering can
  adapt to hardware profiles
  (`docs/telephone-design.md:358-374`, `docs/telephone-design.md:2743-2793`).
- [ ] Deliver multi-GPU orchestration within a node, including scheduling and
  memory partitioning
  (`docs/telephone-design.md:1708-1734`, `docs/telephone-design.md:3500-3533`).

### Step: Optimise kernel performance

- [ ] Implement memory coalescing, occupancy tuning, and architecture-specific
  launch configurations (`docs/telephone-design.md:2793-2833`).
- [ ] Capture GPU metrics (utilisation, memory, temperature) and integrate them
  with monitoring exporters
  (`docs/telephone-design.md:2847-2853`, `docs/telephone-design.md:3544-3561`).
- [ ] Add automated microbenchmarks to regress join, aggregation, and union
  kernels across target hardware
  (`docs/telephone-design.md:1534-1667`, `docs/telephone-design.md:2793-2833`).

### Step: Extend runtime performance monitoring

- [ ] Publish Prometheus-compatible metrics for kernel timings, cache hit rates,
  epoch latency, and spill activity
  (`docs/telephone-design.md:3081-3085`, `docs/telephone-design.md:3544-3561`).
- [ ] Provide Grafana dashboards and alerting thresholds for GPU health, query
  latency, and backlog size (`docs/telephone-design.md:3544-3581`).
- [ ] Document runbooks for triaging performance regressions using the collected
  metrics (`docs/telephone-design.md:3544-3581`).

## Phase 5: Provenance, guardrails, and resilience

### Step: Deliver provenance tracking levels

- [ ] Implement configurable provenance levels (none, rule, tuple, full chain)
  with the storage overheads in `docs/telephone-design.md:2690-2711`.
- [ ] Provide witness-hash computation APIs for lightweight audit queries
  (`docs/telephone-design.md:2713-2727`).
- [ ] Ensure provenance respects spill and rehydrate flows and add regression
  tests covering cancellation
  (`docs/telephone-design.md:2595-2603`, `docs/telephone-design.md:2690-2711`).

### Step: Harden operational guardrails

- [ ] Enforce GPU resource quotas, starvation protection, and provenance
  validation hooks (`docs/telephone-design.md:3074-3079`).
- [ ] Extend observability outputs with guardrail-specific metrics (rate
  limiting, rejection counts, protected shards)
  (`docs/telephone-design.md:3081-3085`, `docs/telephone-design.md:3039-3084`).
- [ ] Publish operational guidance covering guardrail configuration, failure
  modes, and escalation paths
  (`docs/telephone-design.md:3074-3085`, `docs/telephone-design.md:3583-3619`).

### Step: Provide backup and recovery tooling

- [ ] Implement checkpoint, snapshot, and log backup routines with the cadence
      in
  `docs/telephone-design.md:3583-3593`.
- [ ] Automate recovery workflows that reload GPU context, restore checkpoints,
  and validate state before resuming processing
  (`docs/telephone-design.md:3594-3619`).
- [ ] Add disaster-recovery drills to CI/CD environments or staging clusters and
  record execution evidence (`docs/telephone-design.md:3583-3619`).

## Phase 6: Infrastructure, testing, and documentation

### Step: Establish build and release infrastructure

- [ ] Produce container images, binary packages, and cargo releases following
  the distribution matrix in `docs/telephone-design.md:3516-3538`.
- [ ] Provide installation documentation and automation scripts for supported
  environments (`docs/telephone-design.md:3525-3538`).
- [ ] Ensure versioned artefacts integrate with plan and kernel caches so
  upgrades remain deterministic
  (`docs/telephone-design.md:1770-1778`, `docs/telephone-design.md:3516-3538`).

### Step: Implement CI/CD and testing strategy

- [ ] Configure lint, unit, and integration pipelines as described in
  `docs/telephone-design.md:3470-3513`, adding self-hosted GPU runners where
  required.
- [ ] Expand unit, property-based, and GPU execution tests to cover canonical
  planning, incremental updates, and provenance features
  (`docs/telephone-design.md:3089-3124`).
- [ ] Integrate long-running regression suites that exercise ingestion, query,
  and rollback paths end to end
  (`docs/telephone-design.md:2953-3124`, `docs/telephone-design.md:3583-3619`).

### Step: Publish developer and operator documentation

- [ ] Document the `tel` IR, planning pipeline, and caching strategy for
  contributors (`docs/telephone-design.md:1743-1781`).
- [ ] Produce runbooks covering ingestion configuration, guardrail tuning, and
  recovery actions
  (`docs/telephone-design.md:2953-3120`, `docs/telephone-design.md:3074-3085`, `docs/telephone-design.md:3583-3619`).
- [ ] Maintain a changelog that links roadmap tasks to released features so
  stakeholders can trace delivery (`docs/telephone-design.md:1743-3619`).
