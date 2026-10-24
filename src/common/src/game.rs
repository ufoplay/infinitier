//! Identifiers for the Infinity Engine games and their underlying engines.
//!
//! These enums live in `infinitier_common` so any crate (importers, the
//! core game-data builder, BAF compiler / decompiler, …) can speak about
//! "which game we're targeting" without reaching for the heavier
//! `infinitier_core::game_detect` module.

use serde::{Deserialize, Serialize};

/// One concrete Infinity Engine game.
///
/// The variants intentionally distinguish the canonical and Enhanced
/// Edition releases (BG vs BGEE, IWD vs IWDEE, PST vs PSTEE), plus the
/// closely-related variants NearInfinity also recognises (BG1+SoD, EET on
/// top of BG2EE, the BG1 Tutu mod, and the BG2/SoA expansion).
///
/// To pick the engine-level configuration most code needs (object specifier
/// layout, combined-string map, …), group by [`Self::engine`].
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Game {
    /// Original Baldur's Gate (and TotSC).
    Bg,
    /// Baldur's Gate II (SoA / ToB / BGT).
    Bg2,
    /// Baldur's Gate Enhanced Edition.
    Bgee,
    /// Baldur's Gate Enhanced Edition + Siege of Dragonspear.
    BgeeSod,
    /// Baldur's Gate II Enhanced Edition.
    Bg2ee,
    /// Enhanced Edition Trilogy (BGEE+SoD+BG2EE merged install).
    Eet,
    /// BG1 Tutu mod (BG1 content running on the BG2 engine).
    Tutu,
    /// Icewind Dale (and HoW / TotL expansions).
    Iwd { heart_of_winter: bool, totl: bool },
    /// Icewind Dale Enhanced Edition.
    Iwdee,
    /// Icewind Dale 2.
    Iwd2,
    /// Planescape: Torment.
    Pst,
    /// Planescape: Torment Enhanced Edition.
    Pstee,
}

impl std::fmt::Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bg => write!(f, "Baldur's Gate"),
            Self::Bg2 => write!(f, "Baldur's Gate II"),
            Self::Bgee => write!(f, "Baldur's Gate Enhanced Edition"),
            Self::BgeeSod => write!(f, "Baldur's Gate Enhanced Edition + Siege of Dragonspear"),
            Self::Bg2ee => write!(f, "Baldur's Gate II Enhanced Edition"),
            Self::Eet => write!(f, "Enhanced Edition Trilogy"),
            Self::Tutu => write!(f, "Tutu"),
            Self::Iwd {
                heart_of_winter,
                totl,
            } => {
                write!(f, "Icewind Dale")?;
                // TotL builds on HoW, so a TotL install has both flags set.
                if *totl {
                    write!(f, " + Trials of the Luremaster")
                } else if *heart_of_winter {
                    write!(f, " + Heart of Winter")
                } else {
                    Ok(())
                }
            }
            Self::Iwdee => write!(f, "Icewind Dale Enhanced Edition"),
            Self::Iwd2 => write!(f, "Icewind Dale 2"),
            Self::Pst => write!(f, "Planescape: Torment"),
            Self::Pstee => write!(f, "Planescape: Torment Enhanced Edition"),
        }
    }
}

/// The engine family a [`Game`] belongs to.
///
/// Several BAF-related lookups (object specifier order, combined-string map,
/// trigger point / object region presence) only depend on the engine, not on
/// the specific game release.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Engine {
    /// Original BG (pre-EE) engine.
    Bg,
    /// Original BG2 engine (also used by Tutu).
    Bg2,
    /// Shared Enhanced Edition engine (BGEE / BG2EE / IWDEE / PSTEE / EET).
    Ee,
    /// Original Icewind Dale (HoW / TotL).
    Iwd,
    /// Icewind Dale 2.
    Iwd2,
    /// Original Planescape: Torment.
    Pst,
}

impl Game {
    /// Returns the [`Engine`] family the game runs on.
    pub fn engine(self) -> Engine {
        match self {
            Game::Bg => Engine::Bg,
            Game::Bg2 | Game::Tutu => Engine::Bg2,
            Game::Bgee | Game::BgeeSod | Game::Bg2ee | Game::Eet | Game::Iwdee | Game::Pstee => {
                Engine::Ee
            }
            Game::Iwd { .. } => Engine::Iwd,
            Game::Iwd2 => Engine::Iwd2,
            Game::Pst => Engine::Pst,
        }
    }

    /// Returns the game's common abbreviation, e.g. `BG2EE` or `IWD+HoW`,
    /// for places too narrow for the full name its `Debug` prints.
    pub fn short_name(self) -> &'static str {
        match self {
            Game::Bg => "BG",
            Game::Bg2 => "BG2",
            Game::Bgee => "BGEE",
            Game::BgeeSod => "BGEE+SoD",
            Game::Bg2ee => "BG2EE",
            Game::Eet => "EET",
            Game::Tutu => "Tutu",
            // TotL builds on HoW, so a TotL install has both flags set.
            Game::Iwd { totl: true, .. } => "IWD+TotL",
            Game::Iwd {
                heart_of_winter: true,
                totl: false,
            } => "IWD+HoW",
            Game::Iwd {
                heart_of_winter: false,
                totl: false,
            } => "IWD",
            Game::Iwdee => "IWDEE",
            Game::Iwd2 => "IWD2",
            Game::Pst => "PST",
            Game::Pstee => "PSTEE",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_name_tells_the_iwd_expansions_apart() {
        let iwd = |heart_of_winter, totl| Game::Iwd {
            heart_of_winter,
            totl,
        };
        assert_eq!(iwd(false, false).short_name(), "IWD");
        assert_eq!(iwd(true, false).short_name(), "IWD+HoW");
        assert_eq!(iwd(true, true).short_name(), "IWD+TotL");
        assert_eq!(Game::Bg2ee.short_name(), "BG2EE");
    }

    #[test]
    fn engine_per_game_grouping() {
        assert_eq!(Game::Bg.engine(), Engine::Bg);
        assert_eq!(Game::Bg2.engine(), Engine::Bg2);
        assert_eq!(Game::Tutu.engine(), Engine::Bg2);
        assert_eq!(Game::Bgee.engine(), Engine::Ee);
        assert_eq!(Game::BgeeSod.engine(), Engine::Ee);
        assert_eq!(Game::Bg2ee.engine(), Engine::Ee);
        assert_eq!(Game::Eet.engine(), Engine::Ee);
        assert_eq!(Game::Iwdee.engine(), Engine::Ee);
        assert_eq!(Game::Pstee.engine(), Engine::Ee);
        assert_eq!(
            Game::Iwd {
                heart_of_winter: false,
                totl: false
            }
            .engine(),
            Engine::Iwd
        );
        assert_eq!(Game::Iwd2.engine(), Engine::Iwd2);
        assert_eq!(Game::Pst.engine(), Engine::Pst);
    }
}
