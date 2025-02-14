//! Various data used in RMS scripts. Includes:
//!
//! - Syntax
//! - Pre-defined Labels
//! - Terrain Constants
//! - Object Constants
//! - Effect Constants
//! - Effect Type Constants
//! - Attribute Constants
//! - Resource Constants
//! - Technology Constants
//! - ModifyTech Constants
//! - Class Constants
//! - Map Type Constants
//! - Cliff Type Constants
//! - Season Type Constants
//! - Assign Type Constants
//! - Player Data Constants
//! - Civilization Constants

use std::fmt::Display;

use crate::lexer;

/// The type of label, indicating how it's intended to be used in a map script.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum LabelType {
    /// The game mode selected in the lobby dropdown menu.
    GameMode,
    /// The size of the map, including the original sizes and HD' Ludicrous.
    MapSizeLegacy,
    /// The size of the map, using the new sizes introduced in DE.
    MapSizeModern,
    /// The amount of resources with which players begin a game.
    StartingResources,
    /// The age and technology level at which players beign a game.
    StartingAge,
    /// Lobby checkboxes for settings in addition to the dropdown menus.
    AdditionalLobbySettings,
    /// The number of players in the game.
    PlayerCount,
    /// The number of teams in the game, where a team consists of at least 2 players.
    TeamCount,
    /// The number of players on each team. Note the team number refers to the lobby order,
    /// not the number selected in the lobby.
    TeamSize,
    /// Indicates whether a player is on a given team. Note the player and team numbers
    /// refer to lobby order, not the color or team numbers selected in the lobby.
    PlayerInTeam,
    /// The version of the game for which the map is generated.
    GameVersions,
}

impl Display for LabelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use LabelType::*;
        write!(
            f,
            "{}",
            match self {
                GameMode => "Game Mode",
                MapSizeLegacy => "Map Size Legacy",
                MapSizeModern => "Map Size Modern",
                StartingResources => "Starting Resources",
                StartingAge => "Starting Age",
                AdditionalLobbySettings => "Additional Lobby Settings",
                PlayerCount => "Player Count",
                TeamCount => "Team Count",
                TeamSize => "Team Size",
                PlayerInTeam => "Player-in-team",
                GameVersions => "Game Versions",
            }
        )
    }
}

/// A label for if statements.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct Label {
    /// The name of the label. Consists of only non-whitespace tokens and must be nonempty.
    name: String,
    /// The description of the label, if the label is built-in.
    description: Option<String>,
    /// The type of label, if the label is built-in.
    label_type: Option<LabelType>,
}

impl Label {
    /// Constructs a new label using `name` with the given `description`, and `label_type`.
    /// The `name` must consist of only non-whitespace tokens and must be nonempty.
    /// If the label is built-in, then it
    pub fn new(name: &str, description: Option<&str>, label_type: Option<LabelType>) -> Self {
        debug_assert!(!name.is_empty() && !name.chars().any(lexer::is_whitespace));
        Self {
            name: String::from(name),
            description: description.map(String::from),
            label_type,
        }
    }
}
