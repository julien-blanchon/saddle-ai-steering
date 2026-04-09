use saddle_bevy_e2e::action::Action;

pub(super) fn wait_and_capture(screenshot: &'static str, frames: u32) -> Vec<Action> {
    vec![Action::WaitFrames(frames), Action::Screenshot(screenshot.into())]
}
