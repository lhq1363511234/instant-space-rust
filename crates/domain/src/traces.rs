use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// How a writer proved they were standing at the place.
///
/// This is deliberately not a security boundary — a determined person can lie
/// about their coordinates. It is a badge of honesty: a trace left on site
/// reads differently from one left from a sofa, and both are allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresenceProof {
    /// Arrived through the Space QR code, which only exists at the location.
    Scan,
    /// Read the access code off something physical in the room — the WiFi card,
    /// the hotspot SSID, the sign by the till. Verified server-side against a
    /// hash, so unlike a coordinate this one cannot simply be asserted.
    OnSite,
    /// Browser geolocation put them inside the Space radius.
    Geo,
    /// Vouched for by the Space's Discord community.
    Discord,
    /// Wrote from somewhere else, and says so.
    Remote,
}

impl PresenceProof {
    pub fn as_db(self) -> &'static str {
        match self {
            Self::Scan => "scan",
            Self::OnSite => "onsite",
            Self::Geo => "geo",
            Self::Discord => "discord",
            Self::Remote => "remote",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value {
            "scan" => Self::Scan,
            "onsite" => Self::OnSite,
            "geo" => Self::Geo,
            "discord" => Self::Discord,
            _ => Self::Remote,
        }
    }

    /// Whether this counts as having been there. Used to gate capsule opening
    /// and to mark a trace as on-site.
    pub fn is_on_site(self) -> bool {
        matches!(self, Self::Scan | Self::OnSite | Self::Geo | Self::Discord)
    }

    pub fn label(self, zh: bool) -> &'static str {
        match (self, zh) {
            (Self::Scan, true) => "扫码到场",
            (Self::Scan, false) => "Scanned on site",
            (Self::OnSite, true) => "现场口令",
            (Self::OnSite, false) => "On-site code",
            (Self::Geo, true) => "定位到场",
            (Self::Geo, false) => "Located on site",
            (Self::Discord, true) => "社群确认",
            (Self::Discord, false) => "Community verified",
            (Self::Remote, true) => "远程留下",
            (Self::Remote, false) => "Left remotely",
        }
    }
}

/// One entry in a Space's guest book.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trace {
    pub id: Uuid,
    pub space_id: Uuid,
    pub author_id: Option<Uuid>,
    pub author_name: String,
    pub body: String,
    pub proof: PresenceProof,
    pub proof_distance_m: Option<f64>,
    pub weather: Option<String>,
    pub created_at: OffsetDateTime,
    pub can_delete: bool,
}

/// The standing record of a place: how much has accumulated, and who was first.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpaceChronicle {
    pub trace_count: i64,
    pub on_site_count: i64,
    pub capsule_count: i64,
    pub capsule_opened_count: i64,
    pub first_trace_at: Option<OffsetDateTime>,
    pub first_trace_author: Option<String>,
    pub latest_trace_at: Option<OffsetDateTime>,
}

impl SpaceChronicle {
    /// A Space nobody has written in yet. Every seeded Space starts here, which
    /// means the "first arrival" is still unclaimed.
    pub fn is_untouched(&self) -> bool {
        self.trace_count == 0 && self.capsule_count == 0
    }
}

/// What a stranger can see about a sealed capsule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapsuleSummary {
    pub id: Uuid,
    pub space_id: Uuid,
    pub author_name: String,
    pub recipient_hint: String,
    pub radius_m: i32,
    pub opens_at: Option<OffsetDateTime>,
    pub opened_at: Option<OffsetDateTime>,
    pub opened_by_name: Option<String>,
    pub created_at: OffsetDateTime,
    pub is_mine: bool,
}

impl CapsuleSummary {
    pub fn is_sealed(&self) -> bool {
        self.opened_at.is_none()
    }
}

/// The result of trying to open a capsule. Each failure says exactly one thing,
/// so a stranger guessing passphrases learns nothing about the location and a
/// person standing in the right spot knows whether to keep walking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CapsuleOpenResult {
    Opened {
        body: String,
        author_name: String,
        created_at: String,
    },
    /// Right passphrase, wrong place.
    TooFar { distance_m: f64, radius_m: i32 },
    /// Standing in the right place, wrong words.
    WrongPassphrase,
    /// Sealed until a date.
    NotYet { opens_at: String },
    /// No coordinates offered at all.
    PresenceRequired,
    AlreadyOpened {
        opened_at: String,
        opened_by_name: Option<String>,
    },
    /// Too many wrong guesses; the capsule stops answering.
    Locked,
}

/// Great-circle distance in metres.
///
/// Capsules compare this against their radius, so it has to be honest about
/// the curvature — a naive planar approximation drifts badly at high latitude,
/// and several seeded Spaces sit above 55°N.
pub fn distance_metres(lat_a: f64, lng_a: f64, lat_b: f64, lng_b: f64) -> f64 {
    const EARTH_RADIUS_M: f64 = 6_371_000.0;
    let phi_a = lat_a.to_radians();
    let phi_b = lat_b.to_radians();
    let delta_phi = (lat_b - lat_a).to_radians();
    let delta_lambda = (lng_b - lng_a).to_radians();

    let a = (delta_phi / 2.0).sin().powi(2)
        + phi_a.cos() * phi_b.cos() * (delta_lambda / 2.0).sin().powi(2);
    2.0 * a.sqrt().atan2((1.0 - a).sqrt()) * EARTH_RADIUS_M
}

/// A capsule stops answering after this many wrong guesses.
pub const CAPSULE_MAX_ATTEMPTS: i32 = 8;

pub const TRACE_MAX_CHARS: usize = 600;
pub const CAPSULE_MAX_CHARS: usize = 2000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn on_site_proofs_are_the_verified_kinds() {
        assert!(PresenceProof::Scan.is_on_site());
        assert!(PresenceProof::OnSite.is_on_site());
        assert!(PresenceProof::Geo.is_on_site());
        assert!(PresenceProof::Discord.is_on_site());
        assert!(!PresenceProof::Remote.is_on_site());
    }

    #[test]
    fn proof_round_trips_through_the_database_representation() {
        for proof in [
            PresenceProof::Scan,
            PresenceProof::OnSite,
            PresenceProof::Geo,
            PresenceProof::Discord,
            PresenceProof::Remote,
        ] {
            assert_eq!(PresenceProof::from_db(proof.as_db()), proof);
        }
    }

    #[test]
    fn distance_is_accurate_enough_to_gate_a_300m_radius() {
        // Tiananmen Square to the Forbidden City entrance, roughly 800m apart.
        let d = distance_metres(39.9055, 116.3976, 39.9163, 116.3972);
        assert!((1150.0..1250.0).contains(&d), "unexpected distance {d}");

        // A point 300m due north should land just outside a 300m radius check
        // after rounding, and well inside a 500m one.
        let d = distance_metres(31.2304, 121.4737, 31.2331, 121.4737);
        assert!((295.0..305.0).contains(&d), "unexpected distance {d}");
    }

    #[test]
    fn a_seeded_space_with_nothing_in_it_is_untouched() {
        let chronicle = SpaceChronicle {
            trace_count: 0,
            on_site_count: 0,
            capsule_count: 0,
            capsule_opened_count: 0,
            first_trace_at: None,
            first_trace_author: None,
            latest_trace_at: None,
        };
        assert!(chronicle.is_untouched());
    }
}
