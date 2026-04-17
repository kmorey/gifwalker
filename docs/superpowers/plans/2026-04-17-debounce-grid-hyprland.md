# Debounce Grid Hyprland Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `gifwalker` search while typing, present results in a consistent 4-column grid without the side preview pane, and float/center the app through Hyprland user rules.

**Architecture:** Add a small live-search action helper in the controller layer, wire a cancellable debounce timer in the GTK app, and simplify the window layout to a single grid browser. Keep compositor placement in Hyprland user config by matching the stable app class `com.kevin.gifwalker`.

**Tech Stack:** Rust, GTK4, GLib, Hyprland window rules

---
