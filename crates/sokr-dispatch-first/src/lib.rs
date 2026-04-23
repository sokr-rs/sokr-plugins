//! First-capable dispatch policy plugin for SOKR.
//!
//! Iterates registered substrates in registration order.
//! Dispatches to the first one returning Ok on capability.
//! Falls back to NoCapableSubstrate if none capable.
