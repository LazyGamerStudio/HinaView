// src/view/navigation_types.rs
// Common types for navigation system

/// Direction of navigation movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationDirection {
    Previous,
    Next,
}

/// FSM state for high-speed navigation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavState {
    Idle,
    FastNavigating,
}
