use dors::native_app::app_context_menu::{
    AppContextAction, action_from_tag, context_action_tag, context_action_title,
};

#[test]
fn context_menu_tags_round_trip_to_actions() {
    for action in [
        AppContextAction::Kill,
        AppContextAction::ForceKill,
        AppContextAction::CopyPid,
    ] {
        assert_eq!(action_from_tag(context_action_tag(action)), Some(action));
    }
}

#[test]
fn context_menu_titles_match_expected_labels() {
    assert_eq!(context_action_title(AppContextAction::Kill), "Kill App");
    assert_eq!(
        context_action_title(AppContextAction::ForceKill),
        "Force Kill App"
    );
    assert_eq!(context_action_title(AppContextAction::CopyPid), "Copy PID");
}
