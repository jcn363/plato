# Toolbar Accessibility Icon

The **Accessibility** icon has been added to the Reader toolbar.

- **Icon name:** `accessibility`
- **Location:** Right side of the toolbar, left of the existing Search and Table of Contents icons.
- **Action:** Triggers the Accessibility Settings menu (`ViewId::AccessibilityMenu`).
- **Event:** `Event::Show(ViewId::AccessibilityMenu)` which toggles the menu via `toggle_accessibility_menu`.

This provides a quick entry point for users to adjust:
- Bionic Reading
- Auto‑Pace
- Dyslexic fonts
- High Contrast, Focus Mode, color‑blind filters, and spacing settings.

The icon is displayed on all reader layouts (reflowable and fixed) and respects the current UI theme.

---

*Implemented on*: May 2026
*Author*: OpenCode AI Assistant