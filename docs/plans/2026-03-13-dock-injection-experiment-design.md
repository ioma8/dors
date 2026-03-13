# Dock Injection Experiment Design

**Goal:** Build a separate, minimal Dock injection experiment that loads code into `Dock.app` and attempts to hide the Dock visually while keeping the Dock process alive and preserving work-area reservation.

## Scope

This experiment is explicitly separate from the main `dors` application. It is not intended to be a production-ready feature. It is an investigative path to determine whether control from inside `Dock.app` can achieve what external Accessibility and private WindowServer calls from our own process could not.

The experiment should:

- use a minimal loader/payload architecture
- load code into `Dock.app`
- perform one behavior only: visually suppress the Dock
- be reversible by unloading / restarting Dock
- leave the main `dors` runtime untouched

The experiment does not need to:

- preserve every Dock behavior
- be stable across macOS versions
- avoid SIP caveats
- integrate with the existing custom dock app

## Approaches Considered

### 1. Recommended: minimal `yabai`-style scripting-addition experiment

Create a small payload that runs inside `Dock.app` and exposes a single “hide Dock visuals” behavior. A companion binary is responsible for deploying/loading the experiment and restoring by restarting Dock.

Pros:

- closest to the class of tools that can actually influence Dock-owned WindowServer behavior
- narrow enough to stay experimental
- more realistic than trying to externally manipulate Dock windows forever

Cons:

- invasive
- likely SIP-sensitive
- version-fragile

### 2. Direct runtime dylib injection into running `Dock.app`

Inject a dylib directly into the running process using a process injection mechanism.

Pros:

- no osax-style packaging

Cons:

- even more brittle
- harder to load/unload cleanly
- more likely to fail due to process protections

### 3. Continue external private WindowServer control

Rejected based on evidence already gathered: the relevant alpha/order calls are ignored or rejected for Dock-related windows from our own process.

## Architecture

Add an isolated experiment tree under `src/dock_injection_experiment/` plus a dedicated binary such as `src/bin/dock_inject_experiment.rs`.

The experiment will have these pieces:

- `payload`: minimal code intended to run inside `Dock.app`
- `loader`: deploy/load helper
- `runtime probe`: code that verifies whether the payload is present and performing the visual suppression
- `restore path`: on exit, restart Dock to return it to normal state

The payload should be deliberately small:

- identify Dock-related rendered elements from inside the Dock process side
- apply a single suppression strategy
- log enough to determine whether the payload is alive and what it changed

## Success Criteria

The experiment is successful if:

- the payload can be loaded into `Dock.app`
- the Dock remains running
- the Dock work area remains reserved
- the Dock becomes visually absent or materially near-invisible
- restoring by restarting Dock returns the system to baseline

## Testing

Testing is mostly manual:

- pure tests for packaging / plan-building helpers where practical
- `cargo check` after every change
- manual execution of the experiment binary
- manual logging review from the injected side and the launcher side

## Risks

- SIP may block the experiment entirely
- the injection/load path may be macOS-version-specific
- unloading code from a live system process may not be realistic; restart-based restore is acceptable
- the payload may need private symbols or behavior that vary across releases

## Implementation Principle

Keep the experiment as small and disposable as possible. If this path works, it proves the architectural point. If it fails, we stop with clear evidence rather than quietly turning `dors` into a full `yabai` clone.
