#[path = "boundary_runtime/deferred.rs"]
mod deferred;
#[path = "boundary_runtime/enter.rs"]
mod enter;
#[path = "boundary_runtime/hard.rs"]
mod hard;
#[path = "boundary_runtime/space.rs"]
mod space;

pub(super) use deferred::{try_handle_deferred_typing_assist, DeferredTypingAssistContext};
pub(super) use enter::{try_handle_enter_autocorrect, EnterAutocorrectContext};
pub(super) use hard::{
    handle_hard_boundary_if_needed, note_learning_backspace_if_needed, HardBoundaryContext,
};
pub(super) use space::{
    handle_space_press, try_handle_space_release, SpacePressContext, SpaceReleaseContext,
};
