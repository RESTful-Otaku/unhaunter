use bevy::prelude::*;

#[derive(Clone, Debug, Default, Event)]
pub struct BoardDataToRebuild {
    pub lighting: bool,
    pub collision: bool,
    /// True when the whole board was (re)loaded, i.e. the room layout may have
    /// changed even if its dimensions did not. Systems that cache per-map data
    /// keyed on dimensions should rebuild when this is set.
    pub initialize: bool,
}
