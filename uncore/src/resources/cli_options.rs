use bevy::prelude::Resource;

/// Command-line options forwarded from the native binary into the Bevy app.
///
/// These are consumed at app setup time and inserted into the ECS world so that
/// gameplay systems (map loading, QA tooling, etc.) can read them.
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct CliOptions {
    /// Include draft maps that are normally hidden from players (`--draft-maps`).
    pub include_draft_maps: bool,
    /// Requested log verbosity (`--verbose`, repeatable).
    ///
    /// `0` keeps the default `INFO` level, each additional flag raises the
    /// filter (`-v` => `DEBUG`, `-vv` => `TRACE`). See [`CliOptions::log_level`].
    pub verbose: u8,
    /// Silence all audio at startup (`--mute`).
    pub mute: bool,
}

impl CliOptions {
    /// Returns the Bevy [`Level`](bevy::log::Level) that corresponds to the
    /// requested verbosity.
    pub fn log_level(&self) -> bevy::log::Level {
        match self.verbose {
            0 => bevy::log::Level::INFO,
            1 => bevy::log::Level::DEBUG,
            _ => bevy::log::Level::TRACE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_cli_options_are_quiet() {
        let opts = CliOptions::default();
        assert!(!opts.include_draft_maps);
        assert!(!opts.mute);
        assert_eq!(opts.verbose, 0);
        assert_eq!(opts.log_level(), bevy::log::Level::INFO);
    }

    #[test]
    fn verbose_maps_to_higher_log_levels() {
        let debug = CliOptions {
            verbose: 1,
            ..Default::default()
        };
        assert_eq!(debug.log_level(), bevy::log::Level::DEBUG);
        let trace = CliOptions {
            verbose: 2,
            ..Default::default()
        };
        assert_eq!(trace.log_level(), bevy::log::Level::TRACE);
        let clamped = CliOptions {
            verbose: 42,
            ..Default::default()
        };
        assert_eq!(clamped.log_level(), bevy::log::Level::TRACE);
    }
}
