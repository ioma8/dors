# dors Production Readiness Plan

## Goal

Move `dors` from a strong native macOS prototype into a polished, reliable, supportable product.

## Current State

The app already has the hard prototype pieces in place:

- native AppKit dock shell
- native hover popovers and context menus
- native launcher/runtime app activation paths
- AX-driven custom work-area window management
- config/import/icon/running-app adapters

The biggest remaining gap is no longer basic functionality. It is product reliability and operational quality across real macOS edge cases.

## Main Product Decision

Before polishing further, decide what `dors` is supposed to be:

- a Dock replacement only
- a Dock companion with optional window-management features
- a broader power-user window manager with a Dock UI

That decision affects onboarding, permissions, release expectations, and how much complexity is acceptable in the final product.

## Workstreams

### 1. Product Scope And Support Policy

Define the exact supported scope and unsupported cases.

Questions to answer:

- Is the system Dock hidden, preserved, or treated as fallback only?
- Are multi-monitor setups supported?
- How should Spaces and true fullscreen apps behave?
- What happens if `dors` is not running at login?
- Is window clamping a core feature or optional power-user mode?

Deliverables:

- product definition
- supported/unsupported matrix
- user-facing expectations for startup, relaunch, and failure modes

### 2. Reliability And State Recovery

Stabilize runtime behavior so the app remains correct after long sessions and system changes.

Priority areas:

- finish removing the remaining script-based `window_clamper` query/apply/fallback paths
- harden startup, relaunch, sleep/wake, Dock relaunch, and crashed target-app scenarios
- detect and recover from stale or lost AX observers
- ensure startup window-state initialization stays correct across all “already maximized” cases
- add explicit degraded-mode behavior when permissions disappear

Deliverables:

- native-only happy path for runtime-critical flows
- recovery behavior for stale runtime state
- lower-risk behavior under macOS edge cases

### 3. Permissions, Installation, And Release Path

Turn the app into something users can actually install and trust.

Needed work:

- proper Accessibility permission onboarding UI
- permission re-check and recovery prompts
- signed app bundle
- notarization
- install/update strategy
- login item / auto-start decision

Deliverables:

- first-run onboarding flow
- distributable signed app
- documented release/install process

### 4. UX And Interface Polish

Make the product feel intentional and consistent rather than “functional but custom.”

Focus areas:

- dock animations and interaction timing
- hover window popover refinement
- right-click context menu polish
- error and loading surfaces
- settings/preferences UI
- sizing, spacing, icon behavior, and visual consistency

Likely settings:

- dock size
- spacing
- startup behavior
- window-clamping enable/disable
- animation speed
- logging/debug mode

Deliverables:

- polished visual interaction system
- preferences UI with persisted user settings

### 5. Testing, Diagnostics, And Regression Prevention

Raise confidence enough that future changes do not break core behavior.

Needed coverage:

- managed zoom state transitions
- startup state initialization
- AX event reduction/coalescing
- launcher behavior
- hover popup and context menu logic
- config/import flows

Observability improvements:

- replace broad debug logging with structured logging levels
- keep a narrow diagnostics mode for hard macOS issues
- maintain manual regression checklists for macOS-specific scenarios

Deliverables:

- broader deterministic tests
- cleaner diagnostic system
- repeatable manual QA checklist

### 6. Architecture Cleanup

Make the codebase easier to maintain before product growth makes cleanup expensive.

Recommended cleanup:

- separate Dock UI concerns from window-management concerns more clearly
- reduce global mutable state where practical
- tighten macOS adapter boundaries
- isolate experiments from production paths
- reduce duplication between AX helpers and clamp/runtime paths

Deliverables:

- cleaner module boundaries
- easier future feature work
- lower maintenance cost

## Recommended Priority Order

1. Eliminate remaining script-based `window_clamper` runtime paths
2. Add proper Accessibility permission onboarding and failure recovery
3. Harden reliability around startup/relaunch/sleep-wake/multi-app runtime state
4. Add preferences and user-configurable behavior
5. Package, sign, and notarize the app
6. Run a full UX polish pass
7. Reduce debug logging and move to structured diagnostics
8. Do architecture cleanup before larger feature expansion

## Definition Of “Production Ready”

`dors` should only be called production-ready when all of these are true:

- core runtime behavior works without script fallbacks in critical paths
- startup and relaunch behavior is predictable
- Accessibility permission issues are handled gracefully
- install/update/signing/notarization are solved
- major macOS edge cases are documented and tested
- visual/UI behavior is consistent and intentional
- diagnostics exist for hard field failures without relying on ad hoc debug prints

## Short Version

The remaining work is mostly not about inventing features. It is about:

- reliability
- onboarding
- packaging
- recoverability
- polish

The prototype already proves the product idea. The next stage is making it dependable enough to survive real macOS usage.
