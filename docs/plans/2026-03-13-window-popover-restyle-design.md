# Window Popover Restyle Design

## Goal

Restyle the hover window popover into a compact floating palette that feels intentional and native, without changing its behavior.

## Chosen Direction

Use a compact floating palette:

- dark translucent container
- rounded corners and tighter padding
- custom row styling instead of default AppKit button chrome
- left-aligned titles with clearer spacing
- hover highlight and pressed state

## Scope

In scope:

- visual restyle of the existing `NSPopover` content
- custom row/button appearance
- layout constant updates

Out of scope:

- new data fields
- changing hover timing
- changing popup behavior or activation logic

## Implementation Notes

- Keep `NSPopover`
- Replace stock `NSButton` look with transparent buttons layered over custom row backgrounds
- Add a dedicated styled container view for the palette
- Keep tests focused on layout/style constants
