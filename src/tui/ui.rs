//! Drawing the terminal UI from its state.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};

use super::app::{App, Focus, Message, Mode, STATUSES};
use super::{keys, theme};
use crate::issue::Issue;

/// Draw the whole screen.
pub fn render(app: &mut App, frame: &mut Frame) {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(3),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    let [list, detail] =
        Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)]).areas(body);

    render_header(app, frame, header);
    render_list(app, frame, list);
    render_detail(app, frame, detail);
    render_footer(app, frame, footer);

    match &app.mode {
        Mode::Help => render_help(frame),
        Mode::Status(cursor) => render_status_picker(*cursor, frame),
        Mode::Note(area) => {
            let popup = centered(frame.area(), 60, 10);
            frame.render_widget(Clear, popup);
            frame.render_widget(
                area.as_ref(),
                popup.inner(ratatui::layout::Margin::new(1, 1)),
            );
            frame.render_widget(
                Block::bordered()
                    .title(" Working note ")
                    .title_bottom(Line::from(vec![
                        Span::styled(" ^s ", theme::key()),
                        Span::raw("save  "),
                        Span::styled("Esc ", theme::key()),
                        Span::raw("cancel "),
                    ]))
                    .border_style(theme::focused_border()),
                popup,
            );
        }
        Mode::Normal | Mode::Filter => {}
    }
}

fn render_header(app: &App, frame: &mut Frame, area: Rect) {
    let scope = app.project.as_deref().unwrap_or("all projects");
    let closed = if app.show_closed {
        "open and closed"
    } else {
        "open"
    };
    let line = Line::from(vec![
        Span::styled("tisket", theme::heading()),
        Span::styled(
            format!("  {}", app.root.file_name().unwrap_or(app.root.as_str())),
            theme::dim(),
        ),
        Span::raw(format!(
            "  {scope} · {closed} · {} shown",
            app.visible.len()
        )),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn pane(title: &str, focused: bool) -> Block<'_> {
    Block::new()
        .borders(Borders::ALL)
        .title(format!(" {title} "))
        .border_style(if focused {
            theme::focused_border()
        } else {
            theme::border()
        })
}

fn render_list(app: &mut App, frame: &mut Frame, area: Rect) {
    let block = pane("Issues", app.focus == Focus::List);
    app.page = usize::from(area.height.saturating_sub(2)).max(1);
    if app.visible.is_empty() {
        let hint = if app.filter_text().is_empty() {
            if app.show_closed {
                "No issues here."
            } else {
                "No open issues. Press a to show closed issues."
            }
        } else if matches!(app.mode, Mode::Filter) {
            "No issue matches the filter. Esc clears it."
        } else {
            "No issue matches the filter. Press / then Esc to clear it."
        };
        frame.render_widget(
            Paragraph::new(Span::styled(hint, theme::dim()))
                .wrap(Wrap { trim: true })
                .block(block),
            area,
        );
        return;
    }
    let items: Vec<ListItem> = app
        .visible
        .iter()
        .map(|&i| list_row(&app.issues[i]))
        .collect();
    let list = List::new(items)
        .block(block)
        .highlight_style(theme::selected());
    let mut state = ListState::default().with_selected(Some(app.selected));
    frame.render_stateful_widget(list, area, &mut state);
}

fn list_row(issue: &Issue) -> ListItem<'_> {
    let status = issue.frontmatter.status;
    let short = issue.id.split('-').next().unwrap_or(&issue.id);
    ListItem::new(Line::from(vec![
        Span::styled(format!("{:<12}", status.to_string()), theme::status(status)),
        Span::styled(format!("{short:<5} "), theme::dim()),
        Span::raw(issue.frontmatter.title.clone()),
    ]))
}

fn field<'a>(name: &'a str, value: String) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("{name:<10}"), theme::dim()),
        Span::raw(value),
    ])
}

fn render_detail(app: &mut App, frame: &mut Frame, area: Rect) {
    let block = pane("Issue", app.focus == Focus::Detail);
    let Some(issue) = app.selected_issue() else {
        frame.render_widget(block, area);
        return;
    };
    let fm = &issue.frontmatter;
    let mut lines: Vec<Line> = vec![
        Line::styled(fm.title.clone(), theme::heading()),
        Line::from(vec![
            Span::styled(fm.status.to_string(), theme::status(fm.status)),
            Span::styled(
                format!(
                    "  {} · {}{}",
                    issue.id,
                    issue.project,
                    if issue.closed { " · closed" } else { "" }
                ),
                theme::dim(),
            ),
        ]),
        Line::raw(""),
    ];
    if let Some(p) = &fm.priority {
        lines.push(field("Priority", p.clone()));
    }
    if let Some(a) = &fm.assignee {
        lines.push(field("Assignee", a.clone()));
    }
    if let Some(d) = &fm.due_date {
        lines.push(field("Due", d.clone()));
    }
    if !fm.labels.is_empty() {
        lines.push(field("Labels", fm.labels.join(", ")));
    }
    if !fm.depends_on.is_empty() {
        lines.push(field("Depends", fm.depends_on.join(", ")));
    }
    if issue.diverges {
        lines.push(field("Branches", "differs on another branch".into()));
    }
    if !fm.children.is_empty() {
        match &app.rollup {
            Some(rollup) => {
                lines.push(field(
                    "Children",
                    format!("{} of {} done", rollup.done, rollup.rows.len()),
                ));
                for row in &rollup.rows {
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(format!("{:<12}", row.status), theme::dim()),
                        Span::raw(format!("{} {}", row.id, row.title)),
                    ]));
                }
            }
            None => lines.push(field("Children", fm.children.join(", "))),
        }
    }
    section(&mut lines, "Body", &issue.body);
    section(&mut lines, "Working notes", &issue.scratch);

    let paragraph = Paragraph::new(Text::from(lines)).wrap(Wrap { trim: false });
    // The furthest scroll that still shows the last line at the bottom
    // of the pane. The count is of wrapped lines, so it needs the
    // width the text will wrap to.
    let inner = block.inner(area);
    let rows = paragraph.line_count(inner.width);
    let max = u16::try_from(rows.saturating_sub(usize::from(inner.height))).unwrap_or(u16::MAX);
    let scroll = app.detail_scroll.min(max);
    frame.render_widget(paragraph.scroll((scroll, 0)).block(block), area);
    app.detail_max = max;
    app.detail_scroll = scroll;
}

fn section<'a>(lines: &mut Vec<Line<'a>>, title: &'a str, text: &'a str) {
    let text = text.trim();
    if text.is_empty() {
        return;
    }
    // One blank line before a section, never two.
    if lines.last().is_some_and(|l| l.width() > 0) {
        lines.push(Line::raw(""));
    }
    lines.push(Line::styled(title, theme::heading()));
    lines.extend(text.lines().map(Line::raw));
}

fn render_footer(app: &App, frame: &mut Frame, area: Rect) {
    if matches!(app.mode, Mode::Filter) {
        let [label, input] =
            Layout::horizontal([Constraint::Length(8), Constraint::Min(1)]).areas(area);
        frame.render_widget(Span::styled(" filter ", theme::key()), label);
        frame.render_widget(&app.filter, input);
        return;
    }
    let line = match &app.message {
        Some(Message::Info(text)) => Line::raw(format!(" {text}")),
        Some(Message::Error(text)) => Line::styled(format!(" {text}"), theme::error()),
        None => {
            let filter = app.filter_text();
            let mut spans = Vec::new();
            if !filter.is_empty() {
                spans.push(Span::styled(" filter ", theme::key()));
                spans.push(Span::raw(format!("{filter}  ")));
            }
            for (key, what) in [
                ("s", "status"),
                ("n", "note"),
                ("e", "edit"),
                ("/", "filter"),
                ("?", "help"),
                ("q", "quit"),
            ] {
                spans.push(Span::styled(format!(" {key} "), theme::key()));
                spans.push(Span::styled(what, theme::dim()));
            }
            Line::from(spans)
        }
    };
    frame.render_widget(Paragraph::new(line), area);
}

fn render_help(frame: &mut Frame) {
    let rows: Vec<Line> = keys::help_rows()
        .map(|b| {
            Line::from(vec![
                Span::styled(format!(" {:<11}", b.label), theme::key()),
                Span::raw(b.description),
            ])
        })
        .collect();
    let height = u16::try_from(rows.len())
        .unwrap_or(u16::MAX)
        .saturating_add(2);
    let popup = centered(frame.area(), 56, height);
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(rows).block(
            Block::bordered()
                .title(" Keys ")
                .title_bottom(" Esc, ? or q to close ")
                .border_style(theme::focused_border()),
        ),
        popup,
    );
}

fn render_status_picker(cursor: usize, frame: &mut Frame) {
    let items: Vec<ListItem> = STATUSES
        .iter()
        .map(|s| ListItem::new(Span::styled(s.to_string(), theme::status(*s))))
        .collect();
    let popup = centered(
        frame.area(),
        24,
        u16::try_from(STATUSES.len()).unwrap_or(7) + 2,
    );
    frame.render_widget(Clear, popup);
    let list = List::new(items)
        .block(
            Block::bordered()
                .title(" Status ")
                .title_bottom(Line::from(vec![
                    Span::styled(" Enter ", theme::key()),
                    Span::raw("set  "),
                    Span::styled("Esc ", theme::key()),
                    Span::raw("cancel "),
                ]))
                .border_style(theme::focused_border()),
        )
        .highlight_style(theme::selected());
    let mut state = ListState::default().with_selected(Some(cursor));
    frame.render_stateful_widget(list, popup, &mut state);
}

/// A popup of the given size, centred in the area.
fn centered(area: Rect, width: u16, height: u16) -> Rect {
    area.centered(Constraint::Length(width), Constraint::Length(height))
}
