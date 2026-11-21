//! The terminal UI, driven by key events and drawn to a test backend.
//!
//! Each snapshot is the rendered screen at a fixed size over a fixed
//! tracker, so a change to what the user sees shows up as a diff to
//! review. Colour does not appear in a snapshot, so the tests that
//! depend on a style read the buffer's cells directly.

use camino::Utf8PathBuf;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::Modifier;

use super::app::{App, Effect, Message, Mode};
use super::{theme, ui};
use crate::issue::Status;
use crate::repo::Repo;

const WIDTH: u16 = 100;
const HEIGHT: u16 = 24;

fn issue(dir: &std::path::Path, name: &str, frontmatter: &str, body: &str) {
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(
        dir.join(format!("{name}.md")),
        format!("---\n{frontmatter}created: \"2000-01-01T00:00:00Z\"\n---\n\n{body}\n"),
    )
    .unwrap();
}

/// A tracker in two projects: an epic and its children, a blocked
/// issue with notes, a labelled docs task, and one closed issue.
fn tracker(tag: &str) -> Utf8PathBuf {
    let base = std::env::temp_dir()
        .join(format!("tisket-tui-{tag}-{}", std::process::id()))
        .join("tracker");
    let _ = std::fs::remove_dir_all(&base);
    let t = base.join(".tisket");
    std::fs::create_dir_all(t.join("default")).unwrap();
    std::fs::create_dir_all(t.join("docs")).unwrap();
    std::fs::write(base.join("tisket.yml"), "tisket_dir: .tisket\n").unwrap();
    std::fs::write(t.join("default/project.yml"), "name: default\n").unwrap();
    std::fs::write(t.join("docs/project.yml"), "name: docs\n").unwrap();
    issue(
        &t.join("default"),
        "ep01-ship-the-board",
        "title: Ship the board\nstatus: in_progress\nchildren: [ch01-draw-the-list, ch02-read-keys]\nupdated: \"2000-01-05T00:00:00Z\"\n",
        "One screen that lists every open issue.",
    );
    issue(
        &t.join("default"),
        "ch01-draw-the-list",
        "title: Draw the list\nstatus: todo\nupdated: \"2000-01-04T00:00:00Z\"\n",
        "",
    );
    issue(
        &t.join("default"),
        "ch02-read-keys",
        "title: Read keys\nstatus: blocked\nassignee: sam\nlabels: [input]\nupdated: \"2000-01-03T00:00:00Z\"\n",
        "Waits on the key map.\n\n## Scratch Notes\n\nThe terminal reports releases too.",
    );
    issue(
        &t.join("docs"),
        "dc01-write-the-guide",
        "title: Write the guide\nstatus: discovery\npriority: \"2\"\ndue_date: \"2001-02-03\"\nlabels: [docs]\ndepends_on: [ch01-draw-the-list]\nupdated: \"2000-01-02T00:00:00Z\"\n",
        "Explain each key.",
    );
    issue(
        &t.join("default/.closed"),
        "ol01-old-prototype",
        "title: Old prototype\nstatus: done\nupdated: \"2000-01-01T00:00:00Z\"\n",
        "Replaced.",
    );
    Utf8PathBuf::try_from(base).unwrap()
}

fn draw(app: &mut App) -> Terminal<TestBackend> {
    let mut terminal = Terminal::new(TestBackend::new(WIDTH, HEIGHT)).unwrap();
    terminal.draw(|frame| ui::render(app, frame)).unwrap();
    terminal
}

fn press(app: &mut App, code: KeyCode) -> Option<Effect> {
    app.handle_key(KeyEvent::new(code, KeyModifiers::NONE))
}

fn type_text(app: &mut App, text: &str) {
    for c in text.chars() {
        press(app, KeyCode::Char(c));
    }
}

fn selected_title(app: &App) -> String {
    app.selected_issue()
        .map(|i| i.frontmatter.title.clone())
        .unwrap_or_default()
}

#[test]
fn the_board_lists_open_issues_with_work_under_way_first() {
    let mut app = App::load(&tracker("board")).unwrap();
    insta::assert_snapshot!(draw(&mut app).backend());
}

#[test]
fn an_epic_shows_its_children_and_how_many_are_done() {
    let app = App::load(&tracker("epic")).unwrap();
    assert_eq!(selected_title(&app), "Ship the board");
    let rollup = app.rollup.as_ref().expect("an epic loads its rollup");
    assert_eq!(rollup.rows.len(), 2);
    assert_eq!(rollup.done, 0);
    // The board snapshot shows the rollup drawn.
}

#[test]
fn moving_down_shows_the_next_issue_with_its_notes() {
    let mut app = App::load(&tracker("down")).unwrap();
    press(&mut app, KeyCode::Char('j'));
    assert_eq!(selected_title(&app), "Read keys");
    insta::assert_snapshot!(draw(&mut app).backend());
}

#[test]
fn the_help_overlay_lists_every_key() {
    let mut app = App::load(&tracker("help")).unwrap();
    press(&mut app, KeyCode::Char('?'));
    assert!(matches!(app.mode, Mode::Help));
    insta::assert_snapshot!(draw(&mut app).backend());
    press(&mut app, KeyCode::Esc);
    assert!(matches!(app.mode, Mode::Normal));
}

#[test]
fn the_filter_narrows_the_list_as_you_type() {
    let mut app = App::load(&tracker("filter")).unwrap();
    press(&mut app, KeyCode::Char('/'));
    type_text(&mut app, "docs");
    assert_eq!(app.visible.len(), 1);
    assert_eq!(selected_title(&app), "Write the guide");
    insta::assert_snapshot!(draw(&mut app).backend());

    press(&mut app, KeyCode::Esc);
    assert_eq!(app.visible.len(), 4, "Esc clears the filter");
}

#[test]
fn a_filter_that_matches_nothing_says_how_to_clear_it() {
    let mut app = App::load(&tracker("nomatch")).unwrap();
    press(&mut app, KeyCode::Char('/'));
    type_text(&mut app, "zzz");
    press(&mut app, KeyCode::Enter);
    insta::assert_snapshot!(draw(&mut app).backend());
}

#[test]
fn closed_issues_show_only_when_asked_for() {
    let mut app = App::load(&tracker("closed")).unwrap();
    assert!(app.issues.iter().all(|i| !i.closed));
    press(&mut app, KeyCode::Char('a'));
    assert!(app.issues.iter().any(|i| i.id == "ol01-old-prototype"));
    press(&mut app, KeyCode::Char('a'));
    assert!(app.issues.iter().all(|i| !i.closed));
}

#[test]
fn the_project_key_cycles_through_projects_and_back_to_all() {
    let mut app = App::load(&tracker("projects")).unwrap();
    press(&mut app, KeyCode::Char('p'));
    assert_eq!(app.project.as_deref(), Some("default"));
    assert_eq!(app.visible.len(), 3);
    press(&mut app, KeyCode::Char('p'));
    assert_eq!(app.project.as_deref(), Some("docs"));
    assert_eq!(app.visible.len(), 1);
    press(&mut app, KeyCode::Char('p'));
    assert_eq!(app.project, None);
    assert_eq!(app.visible.len(), 4);
}

#[test]
fn choosing_a_status_writes_it_and_keeps_the_issue_selected() {
    let root = tracker("status");
    let mut app = App::load(&root).unwrap();
    press(&mut app, KeyCode::Char('j'));
    press(&mut app, KeyCode::Char('s'));
    assert!(matches!(app.mode, Mode::Status(_)));
    insta::assert_snapshot!(draw(&mut app).backend());

    // From blocked, one step up is in_progress.
    press(&mut app, KeyCode::Char('k'));
    press(&mut app, KeyCode::Enter);
    let on_disk = Repo::open(&root)
        .unwrap()
        .find_issue("ch02-read-keys")
        .unwrap();
    assert_eq!(on_disk.frontmatter.status, Status::InProgress);
    assert_eq!(selected_title(&app), "Read keys");
    assert_eq!(
        app.message,
        Some(Message::Info("ch02-read-keys is now in_progress".into()))
    );
}

#[test]
fn a_done_status_closes_the_issue_and_drops_it_from_the_open_list() {
    let root = tracker("done");
    let mut app = App::load(&root).unwrap();
    press(&mut app, KeyCode::Char('j'));
    press(&mut app, KeyCode::Char('s'));
    // From blocked, two steps down is done.
    for _ in 0..2 {
        press(&mut app, KeyCode::Char('j'));
    }
    press(&mut app, KeyCode::Enter);
    let on_disk = Repo::open(&root)
        .unwrap()
        .find_issue("ch02-read-keys")
        .unwrap();
    assert!(on_disk.closed);
    assert_eq!(on_disk.frontmatter.status, Status::Done);
    assert!(app.issues.iter().all(|i| i.id != "ch02-read-keys"));
}

#[test]
fn a_note_is_appended_to_the_working_notes_on_save() {
    let root = tracker("note");
    let mut app = App::load(&root).unwrap();
    press(&mut app, KeyCode::Char('n'));
    type_text(&mut app, "found it");
    insta::assert_snapshot!(draw(&mut app).backend());
    app.handle_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(matches!(app.mode, Mode::Normal));
    let scratch = Repo::open(&root)
        .unwrap()
        .scratch_read("ep01-ship-the-board")
        .unwrap();
    assert!(scratch.contains("found it"), "{scratch}");
}

#[test]
fn escape_discards_a_note() {
    let root = tracker("discard");
    let mut app = App::load(&root).unwrap();
    press(&mut app, KeyCode::Char('n'));
    type_text(&mut app, "never mind");
    press(&mut app, KeyCode::Esc);
    let scratch = Repo::open(&root)
        .unwrap()
        .scratch_read("ep01-ship-the-board")
        .unwrap();
    assert!(!scratch.contains("never mind"));
}

#[test]
fn an_empty_note_adds_nothing_and_says_so() {
    let root = tracker("emptynote");
    let path = root.join(".tisket/default/ep01-ship-the-board.md");
    let before = std::fs::read_to_string(&path).unwrap();
    let mut app = App::load(&root).unwrap();
    press(&mut app, KeyCode::Char('n'));
    type_text(&mut app, "   ");
    app.handle_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert_eq!(
        app.message,
        Some(Message::Info("the note is empty; nothing added".into()))
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
}

#[test]
fn a_plain_s_in_a_note_is_text_not_save() {
    let root = tracker("plain-s");
    let mut app = App::load(&root).unwrap();
    press(&mut app, KeyCode::Char('n'));
    type_text(&mut app, "tests pass");
    let Mode::Note(area) = &app.mode else {
        panic!("typing an s must not leave the note box");
    };
    assert_eq!(area.lines(), ["tests pass"]);
}

#[test]
fn edit_asks_the_loop_for_an_editor_on_the_selected_issue() {
    let mut app = App::load(&tracker("edit")).unwrap();
    assert_eq!(
        press(&mut app, KeyCode::Char('e')),
        Some(Effect::EditBody("ep01-ship-the-board".into()))
    );
}

#[test]
fn edit_refuses_a_closed_issue() {
    let mut app = App::load(&tracker("editclosed")).unwrap();
    press(&mut app, KeyCode::Char('a'));
    press(&mut app, KeyCode::Char('G'));
    assert_eq!(selected_title(&app), "Old prototype");
    assert_eq!(press(&mut app, KeyCode::Char('e')), None);
    assert_eq!(
        app.message,
        Some(Message::Error(
            "a closed issue's body cannot change; press s to reopen it first".into()
        ))
    );
    insta::assert_snapshot!(draw(&mut app).backend());
}

#[test]
fn an_unreadable_issue_is_reported_in_the_footer() {
    let root = tracker("broken");
    std::fs::write(
        root.join(".tisket/default/zz99-broken.md"),
        "no frontmatter\n",
    )
    .unwrap();
    let mut app = App::load(&root).unwrap();
    assert_eq!(app.visible.len(), 4);
    match &app.message {
        Some(Message::Error(text)) => assert!(text.contains("zz99-broken"), "{text}"),
        other => panic!("expected a warning, got {other:?}"),
    }
    insta::assert_snapshot!(draw(&mut app).backend());
}

#[test]
fn an_empty_tracker_says_how_to_see_closed_issues() {
    let root = tracker("empty");
    for name in [
        "ep01-ship-the-board",
        "ch01-draw-the-list",
        "ch02-read-keys",
    ] {
        std::fs::remove_file(root.join(format!(".tisket/default/{name}.md"))).unwrap();
    }
    std::fs::remove_file(root.join(".tisket/docs/dc01-write-the-guide.md")).unwrap();
    let mut app = App::load(&root).unwrap();
    insta::assert_snapshot!(draw(&mut app).backend());
}

#[test]
fn the_selected_row_is_drawn_in_reverse_video() {
    let mut app = App::load(&tracker("reverse")).unwrap();
    let terminal = draw(&mut app);
    let buffer = terminal.backend().buffer();
    // Row 2 is the first list row, inside the header and the border.
    let cell = &buffer[(2, 2)];
    assert!(cell.modifier.contains(Modifier::REVERSED));
    let below = &buffer[(2, 3)];
    assert!(!below.modifier.contains(Modifier::REVERSED));
}

#[test]
fn the_selected_row_carries_a_mark_that_needs_no_style() {
    // Under NO_COLOR crossterm's empty colour sequence resets reverse
    // video, so the mark is what shows which issue a key acts on.
    let mut app = App::load(&tracker("mark")).unwrap();
    press(&mut app, KeyCode::Char('j'));
    let terminal = draw(&mut app);
    let buffer = terminal.backend().buffer();
    let row = |y: u16| (1..4).map(|x| buffer[(x, y)].symbol()).collect::<String>();
    assert_eq!(row(3), "> b", "the selected row starts with the mark");
    assert_eq!(row(2), "  i", "an unselected row starts with spaces");
}

#[test]
fn a_status_is_drawn_in_its_colour() {
    let mut app = App::load(&tracker("colour")).unwrap();
    press(&mut app, KeyCode::Char('j'));
    let terminal = draw(&mut app);
    let buffer = terminal.backend().buffer();
    // The second list row is the blocked issue; its status column
    // starts after the border and the selection mark.
    let cell = &buffer[(3, 3)];
    assert_eq!(cell.symbol(), "b");
    assert_eq!(cell.fg, theme::status(Status::Blocked).fg.unwrap());
}

#[test]
fn page_down_moves_by_the_rows_the_pane_shows() {
    let mut app = App::load(&tracker("page")).unwrap();
    draw(&mut app);
    assert_eq!(app.page, usize::from(HEIGHT) - 4);
    press(&mut app, KeyCode::PageDown);
    assert_eq!(
        app.selected,
        app.visible.len() - 1,
        "clamped to the last row"
    );
    press(&mut app, KeyCode::Char('g'));
    assert_eq!(app.selected, 0);
}

#[test]
fn tab_moves_the_keys_to_the_issue_pane_where_they_scroll() {
    let root = tracker("tab");
    long_body(&root);
    let mut app = App::load(&root).unwrap();
    draw(&mut app);
    press(&mut app, KeyCode::Tab);
    press(&mut app, KeyCode::Char('j'));
    press(&mut app, KeyCode::Char('j'));
    assert_eq!(app.detail_scroll, 2);
    assert_eq!(app.selected, 0, "the list cursor stays put");
    press(&mut app, KeyCode::Char('k'));
    press(&mut app, KeyCode::Char('k'));
    press(&mut app, KeyCode::Char('k'));
    assert_eq!(app.detail_scroll, 0, "scroll stops at the top");
}

/// Give the epic a body far longer than the pane.
fn long_body(root: &Utf8PathBuf) {
    let body: Vec<String> = (1..=60).map(|n| format!("line {n}")).collect();
    issue(
        root.join(".tisket/default").as_std_path(),
        "ep01-ship-the-board",
        "title: Ship the board\nstatus: in_progress\nupdated: \"2000-01-05T00:00:00Z\"\n",
        &body.join("\n"),
    );
}

#[test]
fn bottom_in_the_issue_pane_stops_at_the_last_line() {
    let root = tracker("bottom");
    long_body(&root);
    let mut app = App::load(&root).unwrap();
    draw(&mut app);
    press(&mut app, KeyCode::Tab);
    press(&mut app, KeyCode::Char('G'));
    assert!(app.detail_max > 0);
    assert_eq!(app.detail_scroll, app.detail_max);
    // The last line is on screen, and one step up moves the text.
    let terminal = draw(&mut app);
    let screen = format!("{}", terminal.backend());
    assert!(screen.contains("line 60"), "{screen}");
    press(&mut app, KeyCode::Char('k'));
    assert_eq!(app.detail_scroll, app.detail_max - 1);
    press(&mut app, KeyCode::Char('j'));
    press(&mut app, KeyCode::Char('j'));
    assert_eq!(
        app.detail_scroll, app.detail_max,
        "scroll stops at the bottom"
    );
}

#[test]
fn the_filter_matches_an_id_and_a_title_too() {
    let mut app = App::load(&tracker("filter-id")).unwrap();
    press(&mut app, KeyCode::Char('/'));
    type_text(&mut app, "ch02");
    assert_eq!(app.visible.len(), 1);
    assert_eq!(selected_title(&app), "Read keys");
    press(&mut app, KeyCode::Esc);
    press(&mut app, KeyCode::Char('/'));
    type_text(&mut app, "DRAW THE");
    assert_eq!(app.visible.len(), 1, "matching ignores case");
    assert_eq!(selected_title(&app), "Draw the list");
}

#[test]
fn a_closed_issue_shows_its_closed_marker_and_every_field() {
    let mut app = App::load(&tracker("closed-detail")).unwrap();
    press(&mut app, KeyCode::Char('a'));
    press(&mut app, KeyCode::Char('G'));
    press(&mut app, KeyCode::Char('k'));
    assert_eq!(selected_title(&app), "Write the guide");
    insta::assert_snapshot!("docs_task_fields", draw(&mut app).backend());
    press(&mut app, KeyCode::Char('j'));
    insta::assert_snapshot!("closed_issue", draw(&mut app).backend());
}

#[test]
fn an_empty_tracker_with_closed_issues_shown_says_none_are_here() {
    let root = tracker("empty-closed");
    std::fs::remove_dir_all(root.join(".tisket/default")).unwrap();
    std::fs::remove_dir_all(root.join(".tisket/docs")).unwrap();
    std::fs::create_dir_all(root.join(".tisket/default")).unwrap();
    std::fs::write(root.join(".tisket/default/project.yml"), "name: default\n").unwrap();
    let mut app = App::load(&root).unwrap();
    press(&mut app, KeyCode::Char('a'));
    let screen = format!("{}", draw(&mut app).backend());
    assert!(screen.contains("No issues here."), "{screen}");
}

#[test]
fn the_status_already_shown_writes_nothing() {
    // The picker opens on the current status, so Enter straight away
    // must not rewrite the file or bump its time.
    let root = tracker("same-status");
    let path = root.join(".tisket/default/ep01-ship-the-board.md");
    let before = std::fs::read_to_string(&path).unwrap();
    let mut app = App::load(&root).unwrap();
    press(&mut app, KeyCode::Char('s'));
    press(&mut app, KeyCode::Enter);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
    assert_eq!(
        app.message,
        Some(Message::Info(
            "ep01-ship-the-board is already in_progress".into()
        ))
    );
}

#[test]
fn a_status_change_made_elsewhere_is_not_undone() {
    // An agent closes the issue while the screen still shows it open.
    let root = tracker("stale");
    let mut app = App::load(&root).unwrap();
    press(&mut app, KeyCode::Char('j'));
    Repo::open(&root)
        .unwrap()
        .close_issue("ch02-read-keys", None)
        .unwrap();
    press(&mut app, KeyCode::Char('s'));
    press(&mut app, KeyCode::Char('k'));
    press(&mut app, KeyCode::Enter);
    let on_disk = Repo::open(&root)
        .unwrap()
        .find_issue("ch02-read-keys")
        .unwrap();
    assert!(on_disk.closed, "the agent's close stands");
    assert_eq!(on_disk.frontmatter.status, Status::Done);
    assert_eq!(
        app.message,
        Some(Message::Error(
            "ch02-read-keys changed on disk to done; nothing written".into()
        ))
    );
}

#[test]
fn reopening_from_the_picker_says_reopened() {
    let root = tracker("reopen-msg");
    let mut app = App::load(&root).unwrap();
    press(&mut app, KeyCode::Char('a'));
    press(&mut app, KeyCode::Char('G'));
    assert_eq!(selected_title(&app), "Old prototype");
    press(&mut app, KeyCode::Char('s'));
    // From done, four steps up is todo.
    for _ in 0..4 {
        press(&mut app, KeyCode::Char('k'));
    }
    press(&mut app, KeyCode::Enter);
    assert_eq!(
        app.message,
        Some(Message::Info("reopened ol01-old-prototype as todo".into()))
    );
}

#[test]
fn a_reload_warning_outranks_a_confirmation() {
    let root = tracker("warn-outranks");
    std::fs::write(
        root.join(".tisket/default/zz99-broken.md"),
        "no frontmatter\n",
    )
    .unwrap();
    let mut app = App::load(&root).unwrap();
    press(&mut app, KeyCode::Char('r'));
    assert!(matches!(&app.message, Some(Message::Error(t)) if t.contains("zz99-broken")));
    press(&mut app, KeyCode::Char('n'));
    type_text(&mut app, "kept");
    app.handle_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(matches!(&app.message, Some(Message::Error(t)) if t.contains("zz99-broken")));
}

#[test]
fn refresh_without_warnings_says_reloaded() {
    let mut app = App::load(&tracker("reloaded")).unwrap();
    press(&mut app, KeyCode::Char('r'));
    assert_eq!(app.message, Some(Message::Info("reloaded".into())));
}

#[test]
fn a_project_that_disappears_falls_back_to_all_projects() {
    let root = tracker("gone");
    let mut app = App::load(&root).unwrap();
    press(&mut app, KeyCode::Char('p'));
    press(&mut app, KeyCode::Char('p'));
    assert_eq!(app.project.as_deref(), Some("docs"));
    std::fs::remove_dir_all(root.join(".tisket/docs")).unwrap();
    press(&mut app, KeyCode::Char('r'));
    assert_eq!(app.project, None);
    assert_eq!(app.visible.len(), 3);
}

#[test]
fn question_mark_and_q_also_close_the_help() {
    let mut app = App::load(&tracker("help-close")).unwrap();
    for close in ['?', 'q'] {
        press(&mut app, KeyCode::Char('?'));
        press(&mut app, KeyCode::Char(close));
        assert!(matches!(app.mode, Mode::Normal), "{close} closes the help");
        assert!(!app.quit, "q in the help closes it and does not quit");
    }
}

#[test]
fn a_key_release_does_nothing() {
    use ratatui::crossterm::event::{Event, KeyEventKind, KeyEventState};
    let mut app = App::load(&tracker("release")).unwrap();
    let release = KeyEvent {
        code: KeyCode::Char('j'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Release,
        state: KeyEventState::NONE,
    };
    super::handle_event(&mut app, Event::Key(release));
    assert_eq!(app.selected, 0);
    super::handle_event(
        &mut app,
        Event::Key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE)),
    );
    assert_eq!(app.selected, 1);
}

mod editor {
    use super::super::{choose_editor, edit_text};

    fn scratch(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("tisket-editor-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn visual_wins_and_a_blank_value_is_skipped() {
        assert_eq!(choose_editor(Some("vis"), Some("ed")).unwrap(), "vis");
        assert_eq!(choose_editor(Some("  "), Some("ed")).unwrap(), "ed");
        assert_eq!(choose_editor(None, Some("ed")).unwrap(), "ed");
        let err = choose_editor(None, Some("")).unwrap_err();
        assert_eq!(err.to_string(), "set $VISUAL or $EDITOR to edit a body");
    }

    fn files_in(dir: &std::path::Path) -> usize {
        std::fs::read_dir(dir).unwrap().count()
    }

    #[test]
    fn an_editor_that_changes_nothing_returns_none() {
        let dir = scratch("same");
        let edited = edit_text("The body.", "true", &dir, "copy-").unwrap();
        assert_eq!(edited, None);
        assert_eq!(files_in(&dir), 0, "the copy is removed");
    }

    #[test]
    fn an_edited_copy_comes_back() {
        // `cp <new> <copy>` stands in for an editor that rewrites the file.
        let dir = scratch("changed");
        let new = dir.join("new.md");
        std::fs::write(&new, "A new body.\n").unwrap();
        let editor = format!("cp {}", new.display());
        let edited = edit_text("Old body.", &editor, &dir, "copy-").unwrap();
        assert_eq!(edited.as_deref(), Some("A new body."));
        assert_eq!(files_in(&dir), 1, "only new.md is left");
    }

    #[test]
    fn a_failed_editor_changes_nothing_and_names_itself() {
        let dir = scratch("failed");
        let err = edit_text("Old body.", "false", &dir, "copy-").unwrap_err();
        assert_eq!(
            err.to_string(),
            "the editor 'false' exited with code 1; the body is unchanged"
        );
    }

    #[test]
    fn a_missing_editor_is_named() {
        let dir = scratch("missing");
        let err = edit_text("Old body.", "no-such-editor-here", &dir, "copy-").unwrap_err();
        assert!(
            err.to_string()
                .starts_with("could not run the editor 'no-such-editor-here'"),
            "{err}"
        );
    }

    #[test]
    fn the_copy_is_private_and_never_overwrites_a_file() {
        let dir = scratch("private");
        let theirs = dir.join("copy-theirs.md");
        std::fs::write(&theirs, "someone else's file").unwrap();
        // `true` stands in for an editor; the copy it is given is a new
        // file under a random name, so the existing one is untouched.
        assert_eq!(edit_text("x", "true", &dir, "copy-").unwrap(), None);
        assert_eq!(
            std::fs::read_to_string(&theirs).unwrap(),
            "someone else's file"
        );
        let kept = super::super::keep_private("tisket-test-", "kept").unwrap();
        assert_eq!(std::fs::read_to_string(&kept).unwrap(), "kept");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            let mode = std::fs::metadata(&kept).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }
        std::fs::remove_file(&kept).unwrap();
    }
}

#[test]
fn an_edited_body_is_saved_when_the_file_is_as_opened() {
    let root = tracker("save-body");
    let mut app = App::load(&root).unwrap();
    app.save_body(
        "One screen that lists every open issue.",
        "A new body.".into(),
    );
    let issue = Repo::open(&root)
        .unwrap()
        .find_issue("ep01-ship-the-board")
        .unwrap();
    assert_eq!(issue.body.trim(), "A new body.");
    assert_eq!(
        app.message,
        Some(Message::Info(
            "saved the body of ep01-ship-the-board".into()
        ))
    );
}

#[test]
fn a_body_changed_on_disk_during_the_edit_is_not_overwritten() {
    // An agent appends to the body while the editor is open.
    let root = tracker("stale-body");
    let mut app = App::load(&root).unwrap();
    let opened = "One screen that lists every open issue.";
    Repo::open(&root)
        .unwrap()
        .edit_issue(
            "ep01-ship-the-board",
            crate::repo::EditIssueOptions {
                append: Some("Added by an agent."),
                ..Default::default()
            },
        )
        .unwrap();
    app.save_body(opened, "My edit.".into());
    let issue = Repo::open(&root)
        .unwrap()
        .find_issue("ep01-ship-the-board")
        .unwrap();
    assert!(issue.body.contains("Added by an agent."), "{}", issue.body);
    assert!(!issue.body.contains("My edit."));
    let Some(Message::Error(text)) = &app.message else {
        panic!("expected a refusal, got {:?}", app.message);
    };
    assert!(text.contains("changed on disk while you edited"), "{text}");
    let kept = text.rsplit("your text is in ").next().unwrap();
    assert_eq!(std::fs::read_to_string(kept).unwrap(), "My edit.");
}

#[test]
fn a_body_whose_save_is_refused_is_kept() {
    // The issue is closed while the editor is open, so the write fails.
    let root = tracker("closed-mid-edit");
    let mut app = App::load(&root).unwrap();
    let opened = "One screen that lists every open issue.";
    Repo::open(&root)
        .unwrap()
        .close_issue("ep01-ship-the-board", Some("done"))
        .unwrap();
    app.save_body(opened, "My edit.".into());
    let Some(Message::Error(text)) = &app.message else {
        panic!("expected a refusal, got {:?}", app.message);
    };
    assert!(text.contains("nothing written"), "{text}");
    let kept = text.rsplit("your text is in ").next().unwrap();
    assert_eq!(std::fs::read_to_string(kept).unwrap(), "My edit.");
}

#[test]
fn a_saved_body_with_a_reload_warning_is_not_reported_as_refused() {
    let root = tracker("save-with-warning");
    let mut app = App::load(&root).unwrap();
    std::fs::write(
        root.join(".tisket/default/zz99-broken.md"),
        "no frontmatter\n",
    )
    .unwrap();
    app.save_body(
        "One screen that lists every open issue.",
        "New body.".into(),
    );
    let issue = Repo::open(&root)
        .unwrap()
        .find_issue("ep01-ship-the-board")
        .unwrap();
    assert_eq!(issue.body.trim(), "New body.");
    let text = format!("{:?}", app.message);
    assert!(!text.contains("nothing written"), "{text}");
}

#[test]
fn an_error_wraps_to_show_whole_and_stays_until_esc() {
    let mut app = App::load(&tracker("long-error")).unwrap();
    let path = "/a/long/temporary/directory/holding/tisket-ep01-abcdef.md";
    app.message = Some(Message::Error(format!(
        "ep01's body changed on disk while you edited; nothing written; your text is in {path}"
    )));
    let screen = draw(&mut app)
        .backend()
        .to_string()
        .replace(['"', '\n', ' '], "");
    assert!(screen.contains("tisket-ep01-abcdef.md"), "{screen}");
    assert!(screen.contains("Escdismisses"), "{screen}");
    press(&mut app, KeyCode::Char('j'));
    assert!(
        matches!(app.message, Some(Message::Error(_))),
        "a move keeps it"
    );
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.message, None);
}

#[test]
fn q_quits() {
    let mut app = App::load(&tracker("quit")).unwrap();
    press(&mut app, KeyCode::Char('q'));
    assert!(app.quit);
}
