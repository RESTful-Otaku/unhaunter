use crate::events::hint::OnScreenHintEvent;
use crate::resources::current_evidence_readings::CurrentEvidenceReadings;
use crate::resources::hint_ui_state::HintUiState;
use crate::resources::mission_select_mode::CurrentMissionSelectMode;
use bevy::prelude::*;

/// The core plugin for the Unhaunter game.
pub struct UnhaunterCorePlugin;

impl Plugin for UnhaunterCorePlugin {
    /// Builds the plugin by adding necessary systems to the app.
    fn build(&self, app: &mut App) {
        crate::metric_recorder::app_setup(app);
        crate::systems::evidence_decay::app_setup(app);
        crate::systems::board::app_setup(app);
        crate::systems::animation::app_setup(app);
        app.init_resource::<CurrentEvidenceReadings>();
        app.init_resource::<CurrentMissionSelectMode>();
        app.init_resource::<HintUiState>();
        app.init_resource::<crate::noise::PerlinNoise>();
        app.init_resource::<crate::resources::player_input::PlayerInput>();
        app.init_resource::<crate::audio_feedback::AudioFeedbackManager>();
        app.init_resource::<crate::object_pool::ObjectPoolManager>();
        app.init_resource::<crate::optimized_noise::OptimizedPerlinNoise>();
        app.init_resource::<crate::performance_optimizer::PerformanceMonitor>();
        app.add_event::<OnScreenHintEvent>();
        app.add_systems(Update, (
            crate::audio_feedback::process_audio_feedback,
            crate::object_pool::manage_pooled_entities,
            crate::object_pool::cleanup_empty_pools,
            crate::object_pool::process_particle_effects,
            crate::optimized_noise::report_noise_cache_stats,
            crate::optimized_noise::cleanup_noise_cache,
            crate::performance_optimizer::monitor_frame_performance,
            crate::performance_optimizer::report_performance_stats,
            crate::performance_optimizer::monitor_memory_usage,
            crate::performance_optimizer::frustum_culling,
        ));
    }
}
