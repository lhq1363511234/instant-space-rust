use leptos::prelude::*;

/// What we currently believe about the visitor's relationship to this place.
///
/// This is shared by the guest book and the capsules: both need to know
/// whether the person is standing here, and both let them proceed either way —
/// the guest book by labelling the entry, the capsules by refusing to open.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PresenceState {
    /// Arrived via the Space QR code, which only exists at the location.
    pub scanned: RwSignal<bool>,
    pub lat: RwSignal<Option<f64>>,
    pub lng: RwSignal<Option<f64>>,
    /// Locating, or a message explaining why we are not.
    pub status: RwSignal<PresenceStatus>,
    /// The visitor says they are vouched for by the Space's Discord community.
    pub discord_member: RwSignal<bool>,
    /// The access code read off something physical at the place — the WiFi
    /// card, the hotspot SSID, the sign by the till. Unlike everything else
    /// here this is checked server-side, so it is the one claim that holds up.
    pub onsite_code: RwSignal<String>,
    /// What the server said last time it checked the code.
    pub code_state: RwSignal<CodeState>,
}

/// Whether the code the visitor typed has been confirmed by the server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeState {
    Untried,
    Checking,
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PresenceStatus {
    Idle,
    Locating,
    Located,
    Denied,
    Unavailable,
}

impl Default for PresenceState {
    fn default() -> Self {
        Self::new()
    }
}

impl PresenceState {
    pub fn new() -> Self {
        Self {
            scanned: RwSignal::new(false),
            lat: RwSignal::new(None),
            lng: RwSignal::new(None),
            status: RwSignal::new(PresenceStatus::Idle),
            discord_member: RwSignal::new(false),
            onsite_code: RwSignal::new(String::new()),
            code_state: RwSignal::new(CodeState::Untried),
        }
    }

    /// The code as it should travel to the server: `None` unless the visitor
    /// actually typed something.
    pub fn code_claim(&self) -> Option<String> {
        let value = self.onsite_code.get().trim().to_string();
        (!value.is_empty()).then_some(value)
    }

    /// Whether presence is settled well enough to open a capsule without the
    /// server having to fall back on coordinates.
    pub fn is_confirmed(&self) -> bool {
        self.scanned.get() || self.code_state.get() == CodeState::Accepted
    }

    pub fn has_fix(&self) -> bool {
        self.lat.get().is_some() && self.lng.get().is_some()
    }
}

/// Reads `?via=qr` off the current URL. The QR code printed for a Space
/// carries this flag, so scanning it is self-evidencing in a way a shared link
/// is not.
pub fn detect_scan() -> bool {
    #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
    {
        let Some(window) = web_sys::window() else {
            return false;
        };
        let Ok(search) = window.location().search() else {
            return false;
        };
        return search.contains("via=qr");
    }

    #[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
    {
        false
    }
}

/// Asks the browser where we are. Results land in the signals; the caller
/// never blocks on it.
pub fn request_location(state: PresenceState) {
    #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
    {
        use wasm_bindgen::{closure::Closure, JsCast};

        let Some(geolocation) = web_sys::window()
            .and_then(|window| window.navigator().geolocation().ok())
        else {
            state.status.set(PresenceStatus::Unavailable);
            return;
        };

        state.status.set(PresenceStatus::Locating);

        let ok = Closure::once_into_js(move |position: web_sys::Position| {
            let coords = position.coords();
            state.lat.set(Some(coords.latitude()));
            state.lng.set(Some(coords.longitude()));
            state.status.set(PresenceStatus::Located);
        });

        let fail = Closure::once_into_js(move |_error: web_sys::PositionError| {
            state.status.set(PresenceStatus::Denied);
        });

        let options = web_sys::PositionOptions::new();
        options.set_enable_high_accuracy(true);
        options.set_timeout(12_000);
        options.set_maximum_age(60_000);

        let _ = geolocation.get_current_position_with_error_callback_and_options(
            ok.unchecked_ref(),
            Some(fail.unchecked_ref()),
            &options,
        );
    }

    #[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
    {
        state.status.set(PresenceStatus::Unavailable);
    }
}
