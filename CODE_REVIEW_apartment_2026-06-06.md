# Code Review: apartment

Date: 2026-06-06
Project Path: D:\WebHatchery\RustGames\apartment

## Findings
- [High] Files at or above 800 lines:
  - src\state\gameplay_actions.rs (977 lines)
  - src\ui\apartment_panel.rs (1152 lines)
- [Medium] TODO/FIXME markers found (4).
  - D:\WebHatchery\RustGames\apartment\src\ui\event_modal.rs - 32:    // TODO: Implement proper wrapping. For now, assuming relatively short text or manual breaks.
  - D:\WebHatchery\RustGames\apartment\src\state\mission_system.rs - 140:                    // TODO: Implement actual tax reduction logic storage
  - D:\WebHatchery\RustGames\apartment\src\state\gameplay_actions.rs - 803:                                // TODO: Apply to tenant network
  - D:\WebHatchery\RustGames\apartment\src\state\gameplay_actions.rs - 808:                                // TODO: Apply to tenant relationships
- [Info] Error-handling markers: unwrap(10), expect(2), panic!(1).
