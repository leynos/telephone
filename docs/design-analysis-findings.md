# Telephone Design Document Audit Findings

This audit reviews `docs/telephone-design.md` for the following:

- Sections that exist primarily to say “we won’t be doing this”.
- Sections that read as box‑ticking exercises rather than being material
  to the project.
- Diagrams of poor quality or low value, with a goal of reducing the
  quantity by 40–50% while improving signal.
- Repetition and statements of the obvious that bloat or dilute the
  document.

All file references point to `docs/telephone-design.md` with the first line of
the relevant section for easy navigation.

## Checklist

- [x] Inventory sections explicitly stating “not applicable”/“no UI”.
- [x] Flag generic, enterprise‑template sections for removal/major trim.
- [x] Catalogue low‑value diagrams and propose removals by category.
- [x] Identify repetition/obvious statements for consolidation.
- [x] Recommend concrete next actions per category.
- [x] Apply edits (cut/trim/merge) and re-run markdown checks.
  - `make fmt` now passes; duplicate-heading lint fixed by renaming and
    consolidating sections.
- [x] Re-review for coherence and narrative flow after pruning.

## 1) “We won’t be doing this” sections

These sections exist chiefly to state that an area is out of scope or not
applicable, then go on to add pages of rationale, flows, and diagrams. The
intent can be captured in a single paragraph; the rest is noise for this
project phase.

- Core Services Architecture Is Not Applicable For This System
  - docs/telephone-design.md:3650
  - Issue: Long, detailed justification for not using microservices,
    with multiple diagrams. Adds redundancy with scope/system boundaries
    elsewhere.
  - Action: Replace the entire sub‑section with a short paragraph in the
    scope/boundaries stating “single‑node, monolithic engine” and the
    two key reasons (performance and simplicity). Remove associated
    diagrams.

- Database Design Is Not Applicable To This System
  - docs/telephone-design.md:3843
  - Issue: Extensive content rationalising “no database”, including
    design patterns and flows. Contradicts the lean scope by creating a
    faux database section.
  - Action: Replace with a one‑paragraph note under Implementation
    Boundaries clarifying “memory‑resident, checkpoint/restore only”.
    Remove the section and all diagrams under it.

- No User Interface Required
  - docs/telephone-design.md:6664
  - Issue: Large UI chapter asserting there is no GUI, followed by
    extensive alternatives, examples, and future GUI ideas.
  - Action: Replace with a brief statement in “Scope” that Telephone is
    headless; interaction is via CLI/API only. Remove UI chapter.

- Out‑of‑scope (overly expansive content)
  - docs/telephone-design.md:268
  - Issue: While an out‑of‑scope list is useful, it is overly broad and
    restates exclusions in multiple places later (e.g., UI, distributed,
    database). Some bullets also reappear as full “not applicable”
    chapters.
  - Action: Keep a compact out‑of‑scope list. Remove items that are
    repeated as whole sections elsewhere (or remove the redundant
    sections per the actions above).

## 2) “Box‑ticking” sections

These read as generic enterprise templates and are not specific to a
single‑node, GPU Datalog engine. They dilute focus and will age poorly without
clear ownership.

- 6.3 Integration Architecture (and sub‑sections)
  - docs/telephone-design.md:4086
  - Examples: Protocol specs, rate limiting, API gateway, versioning,
    documentation standards, message queues, external service contracts
    (4090, 4142, 4194, 4243, 4260, 4271, 4502, 4630, 4667).
  - Issue: Reads as a platform/API gateway brief; not grounded in a
    library/engine scope.
  - Action: Collapse to a small “External Interfaces” section scoped to
    CLI and a minimal API surface. Remove flow diagrams.

- 6.4 Security Architecture (AuthN/Z, RBAC, incident response, etc.)
  - docs/telephone-design.md:4823
  - Issue: Detailed JWT choices, mTLS, RBAC modelling, incident
    response playbooks, compliance matrices (4854, 4961, 5033, 5123,
    5178, 5206). Not actionable/specific to the engine; better suited
    to a service product.
  - Action: Replace with a short “Security Posture” that emphasises
    Rust memory safety, GPU memory zeroisation, and interface hardening
    (authn delegated to the embedding application). Remove diagrams and
    token/flow minutiae.

- 6.5 Monitoring And Observability
  - docs/telephone-design.md:5307
  - Issue: Full monitoring platform blueprint (metrics, logs, tracing,
    dashboards) for a library. Much of this belongs in deployment docs
    for a service wrapper, not the core engine.
  - Action: Keep a paragraph on exposed metrics/hooks. Remove dashboard
    and pipeline details and associated diagrams.

- 6.6 Testing Strategy (service/database/UI‑centric content)
  - docs/telephone-design.md:5927
  - Examples: Service integration tests, API testing, database
    integration testing, UI automation, cross‑browser testing (6114,
    6125, 6136, 6201–6263).
  - Issue: Mismatch with a headless engine; portions are irrelevant.
  - Action: Trim to engine‑specific testing guidance (unit, property‑
    based, differential testing, GPU emulation). Remove UI/service
    testing and related diagrams.

- 3.4 Third‑party Services
  - docs/telephone-design.md:958
  - Issue: “Services” framing misleads; core engine depends on toolchains
    and libraries, not hosted services.
  - Action: Fold essential items into Dependencies; drop the standalone
    “services” framing.

- 8.6 Infrastructure Cost Estimates
  - docs/telephone-design.md:7277
  - Issue: Price tables will go stale; out of scope for the design
    document of an engine.
  - Action: Remove. If needed, maintain a separate, owned cost note.

- 8.7 Deployment Workflow (embedded install scripts/config samples)
  - docs/telephone-design.md:7311
  - Issue: Operational how‑tos and bash scripts are not design; they
    belong in a separate ops/deploy guide or repository.
  - Action: Remove from the design doc. Keep a short pointer to
    installation docs.

## 3) Low‑value diagrams to remove (target: −40–50%)

There are 106 Mermaid diagrams (`mermaid`). Many are generic flowcharts or
sequence diagrams that restate nearby bullet points. Removing the sets below
should comfortably achieve a 40–50% reduction without losing clarity.

Remove entire diagram groups under these headings (illustrative references
shown):

- 4.1.3 Decision Points And Business Rules
  - Examples: Rule compilation decisions and incremental update flows
    (e.g., docs/telephone-design.md:1291, 1339).
  - Reason: Linearised restatements of text; adds little beyond prose.

- 4.4 Error Handling And Recovery
  - Examples: GPU→CPU fallback, recovery workflows
    (docs/telephone-design.md:1852, 1892).
  - Reason: Basic trees duplicating error‑handling bullets.

- 4.5 Performance And Monitoring
  - Examples: Performance tracking, SLA monitoring (1947, 1990).
  - Reason: Operational flows better suited to a runbook.

- 5.2.5 Component Interaction Diagrams, 5.2.6 State Transition Diagrams
  - Examples: Multiple sequence/flow diagrams (2255–2278, 2342).
  - Reason: Repetitive with 5.1’s high‑level diagram; keep only one
    canonical architecture view.

- 6.1 Core Services Architecture (Not Applicable)
  - Examples: Monolith integration and external API sequences (3698,
    3764).
  - Reason: Section to be removed entirely.

- 6.2 Database Design (Not Applicable)
  - Examples: Memory/state flows (3887, 3929).
  - Reason: Section to be removed entirely.

- 6.3 Integration Flow Diagrams
  - Examples: End‑to‑end integration, event stream flows (4669–4756).
  - Reason: Non‑essential for a headless engine.

- 6.4.7 Security Architecture Diagrams
  - Examples: Zone architecture, authn/z flows (5210, 5245).
  - Reason: Better captured as a short security posture statement.

- 6.5 Monitoring/Observability diagrams
  - Examples: Metrics/logs/tracing flows (5335, 5379, 5413, 5467).
  - Reason: Restate standard observability patterns; remove.

Prefer to keep a minimal, high‑value set, e.g.:

- One top‑level architecture diagram (docs/telephone-design.md:106).
- One incremental update core flow (choose a single, concise diagram).
- One GPU memory state lifecycle (if it captures unique mechanics).

This preserves conceptual scaffolding without overwhelming readers.

## 4) Repetition and stating the obvious

The document repeats the same ideas across multiple chapters. Suggested
consolidations follow.

- Single‑node/monolithic design
  - Repeats in: System Boundaries, 6.1 Core Services Architecture,
    8.1 Infrastructure Overview.
  - Action: State once in “Scope/Boundaries”. Remove the dedicated “not
    applicable” chapter and trim the rest.

- DDlog compatibility and incremental semantics
  - Repeats across Features (F‑001/2/3), Success Criteria, Component
    Details.
  - Action: Centralise under “Language/Semantics” and reference it.

- GPU backends (CUDA/SPIR‑V) and hardware independence
  - Repeated in intro, feature tables, tech stack, component details.
  - Action: Keep a single authoritative “Backends” section; remove the
    rest.

- API/Auth/Versioning boilerplate
  - Appears in Integration Architecture and Security Architecture.
  - Action: Collapse to a brief “Interfaces” section; delegate detailed
    auth flows to whoever owns the hosting surface.

- Operational/monitoring/SLA content
  - Spread across 4.5, 5.4.*, 6.5, 8.5/8.6/8.7.
  - Action: Move operational content out of the design doc into ops docs
    or delete if not owned.

- UI vs headless clarifications
  - Appears in Out‑of‑scope and an entire UI chapter.
  - Action: Keep a single line in Scope and remove the UI chapter.

## Recommended next actions

- Cut “not applicable/no UI” chapters; retain a concise statement in
  Scope/Boundaries (single node, headless, memory‑resident).
- Remove Integration Architecture, Security Architecture, Monitoring,
  and Cost/Install workflow sections unless there is a concrete, owned plan to
  implement them in the engine repository.
- Prune 40–50% of diagrams by removing the groups listed above; keep a
  single, canonical architecture diagram and one diagram each for the
  incremental flow and GPU memory lifecycle.
- Consolidate repeated content into one authoritative location and
  replace duplicates with links/references.
- After pruning, re‑run `make markdownlint` and `make fmt`; fix wrapping
  and headings; ensure the design doc fits the current scope and remains
  maintainable.
