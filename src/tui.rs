use std::collections::HashSet;
use std::fs;
use std::io::{self, Read as _};
use std::path::PathBuf;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;

use crate::ignore::IgnoreFilter;
use crate::render::{icon_for_node, is_executable, ColorMap, IconMap};
use crate::search::levenshtein;
use crate::tree::{load_tree, SortMode, TreeNode};

struct FlatEntry {
    name: String,
    path: PathBuf,
    depth: usize,
    is_dir: bool,
    is_expanded: bool,
    has_children: bool,
    is_last_sibling: bool,
    node_id: usize,
    continuation_depths: HashSet<usize>,
}

#[derive(PartialEq)]
enum InputMode {
    Normal,
    Search,
}

struct App {
    tree: TreeNode,
    root_path: PathBuf,
    entries: Vec<FlatEntry>,
    expanded: HashSet<usize>,
    cursor: usize,
    scroll_offset: usize,
    input_mode: InputMode,
    search_query: String,
    search_matches: HashSet<usize>,
    color_map: ColorMap,
    icon_map: IconMap,
    should_quit: bool,
    filter: IgnoreFilter,
    sort: SortMode,
    max_depth: u32,
    preview_content: Vec<String>,
    preview_error: Option<String>,
}

impl App {
    fn new(
        tree: TreeNode,
        root_path: PathBuf,
        color_map: ColorMap,
        icon_map: IconMap,
        filter: IgnoreFilter,
        sort: SortMode,
        max_depth: u32,
    ) -> Self {
        let mut expanded = HashSet::new();
        expanded.insert(0); // root is always expanded
        let entries = flatten_tree(&tree, &expanded);
        let mut app = App {
            tree,
            root_path,
            entries,
            expanded,
            cursor: 0,
            scroll_offset: 0,
            input_mode: InputMode::Normal,
            search_query: String::new(),
            search_matches: HashSet::new(),
            color_map,
            icon_map,
            should_quit: false,
            filter,
            sort,
            max_depth,
            preview_content: Vec::new(),
            preview_error: None,
        };
        app.load_preview();
        app
    }

    fn rebuild_entries(&mut self) {
        self.entries = flatten_tree(&self.tree, &self.expanded);
        if self.cursor >= self.entries.len() {
            self.cursor = self.entries.len().saturating_sub(1);
        }
    }

    fn move_cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.load_preview();
        }
    }

    fn move_cursor_down(&mut self) {
        if self.cursor + 1 < self.entries.len() {
            self.cursor += 1;
            self.load_preview();
        }
    }

    fn adjust_scroll(&mut self, viewport_height: usize) {
        if viewport_height == 0 {
            return;
        }
        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        } else if self.cursor >= self.scroll_offset + viewport_height {
            self.scroll_offset = self.cursor - viewport_height + 1;
        }
    }

    fn expand_or_enter(&mut self) {
        if let Some(entry) = self.entries.get(self.cursor)
            && entry.is_dir
            && entry.has_children
        {
            let node_id = entry.node_id;
            self.expanded.insert(node_id);
            self.rebuild_entries();
            self.load_preview();
        }
    }

    fn collapse_or_parent(&mut self) {
        if let Some(entry) = self.entries.get(self.cursor) {
            let node_id = entry.node_id;
            if entry.is_dir && entry.is_expanded && self.expanded.contains(&node_id) {
                self.expanded.remove(&node_id);
                self.rebuild_entries();
                self.load_preview();
                return;
            }
            // Jump to parent: find closest entry with depth - 1
            let target_depth = entry.depth.saturating_sub(1);
            for i in (0..self.cursor).rev() {
                if self.entries[i].depth == target_depth && self.entries[i].is_dir {
                    self.cursor = i;
                    self.load_preview();
                    return;
                }
            }
        }
    }

    fn page_up(&mut self, viewport_height: usize) {
        let half = viewport_height / 2;
        self.cursor = self.cursor.saturating_sub(half);
        self.load_preview();
    }

    fn page_down(&mut self, viewport_height: usize) {
        let half = viewport_height / 2;
        self.cursor = (self.cursor + half).min(self.entries.len().saturating_sub(1));
        self.load_preview();
    }

    fn jump_home(&mut self) {
        self.cursor = 0;
        self.load_preview();
    }

    fn jump_end(&mut self) {
        self.cursor = self.entries.len().saturating_sub(1);
        self.load_preview();
    }

    fn load_preview(&mut self) {
        self.preview_content.clear();
        self.preview_error = None;

        let path = match self.entries.get(self.cursor) {
            Some(entry) => entry.path.clone(),
            None => return,
        };

        if path.is_dir() {
            let count = match fs::read_dir(&path) {
                Ok(entries) => entries.count(),
                Err(e) => {
                    self.preview_error = Some(format!("Cannot read directory: {e}"));
                    return;
                }
            };
            self.preview_content.push(format!("Directory: {}", path.display()));
            self.preview_content.push(format!("{count} entries"));
            return;
        }

        // Check file size
        let metadata = match fs::metadata(&path) {
            Ok(m) => m,
            Err(e) => {
                self.preview_error = Some(format!("Cannot read file: {e}"));
                return;
            }
        };

        if metadata.len() > 1_048_576 {
            self.preview_error = Some("File too large for preview (>1MB)".to_string());
            return;
        }

        let mut file = match fs::File::open(&path) {
            Ok(f) => f,
            Err(e) => {
                self.preview_error = Some(format!("Cannot open file: {e}"));
                return;
            }
        };

        let mut buf = Vec::new();
        if let Err(e) = file.read_to_end(&mut buf) {
            self.preview_error = Some(format!("Cannot read file: {e}"));
            return;
        }

        // Binary detection: check for null bytes in first 512 bytes
        let check_len = buf.len().min(512);
        if buf[..check_len].contains(&0) {
            self.preview_error = Some("Binary file — no preview available".to_string());
            return;
        }

        match String::from_utf8(buf) {
            Ok(text) => {
                for (i, line) in text.lines().enumerate() {
                    if i >= 500 {
                        self.preview_content.push("... (truncated at 500 lines)".to_string());
                        break;
                    }
                    self.preview_content.push(line.to_string());
                }
            }
            Err(_) => {
                self.preview_error = Some("Binary file — no preview available".to_string());
            }
        }
    }

    fn apply_search(&mut self) {
        self.search_matches.clear();
        if self.search_query.is_empty() {
            self.rebuild_entries();
            return;
        }

        let matches = collect_matches(&self.tree, &self.search_query);
        auto_expand_for_matches(&matches, &self.tree, &mut self.expanded);
        self.rebuild_entries();

        // Map matched node_ids to entry indices
        let match_node_ids: HashSet<usize> = matches.into_iter().collect();
        for (i, entry) in self.entries.iter().enumerate() {
            if match_node_ids.contains(&entry.node_id) {
                self.search_matches.insert(i);
            }
        }

        // Jump to first match
        if let Some(&first) = self.search_matches.iter().min() {
            self.cursor = first;
            self.load_preview();
        }
    }

    fn jump_to_next_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        let next = self
            .search_matches
            .iter()
            .filter(|&&i| i > self.cursor)
            .min()
            .copied()
            .or_else(|| self.search_matches.iter().min().copied());
        if let Some(idx) = next {
            self.cursor = idx;
            self.load_preview();
        }
    }

    fn jump_to_prev_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        let prev = self
            .search_matches
            .iter()
            .filter(|&&i| i < self.cursor)
            .max()
            .copied()
            .or_else(|| self.search_matches.iter().max().copied());
        if let Some(idx) = prev {
            self.cursor = idx;
            self.load_preview();
        }
    }

    fn reload_tree(&mut self) {
        self.tree = load_tree(&self.root_path, self.max_depth, 0, &self.filter, self.sort);
        self.expanded.clear();
        self.expanded.insert(0);
        self.search_query.clear();
        self.search_matches.clear();
        self.cursor = 0;
        self.scroll_offset = 0;
        self.rebuild_entries();
        self.load_preview();
    }

    fn handle_normal_key(&mut self, code: KeyCode, viewport_height: usize) {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Up | KeyCode::Char('k') => self.move_cursor_up(),
            KeyCode::Down | KeyCode::Char('j') => self.move_cursor_down(),
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => self.expand_or_enter(),
            KeyCode::Left | KeyCode::Char('h') => self.collapse_or_parent(),
            KeyCode::Char('/') => {
                self.input_mode = InputMode::Search;
                self.search_query.clear();
                self.search_matches.clear();
            }
            KeyCode::Home => self.jump_home(),
            KeyCode::End => self.jump_end(),
            KeyCode::PageUp => self.page_up(viewport_height),
            KeyCode::PageDown => self.page_down(viewport_height),
            KeyCode::Char('r') => self.reload_tree(),
            _ => {}
        }
    }

    fn handle_search_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.search_query.clear();
                self.search_matches.clear();
                self.rebuild_entries();
                self.load_preview();
            }
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.apply_search();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.apply_search();
            }
            KeyCode::Down => self.jump_to_next_match(),
            KeyCode::Up => self.jump_to_prev_match(),
            _ => {}
        }
    }
}

fn flatten_tree(tree: &TreeNode, expanded: &HashSet<usize>) -> Vec<FlatEntry> {
    let mut entries = Vec::new();
    let mut counter = 0usize;
    flatten_recursive(tree, expanded, 0, true, &HashSet::new(), &mut counter, &mut entries);
    entries
}

fn flatten_recursive(
    node: &TreeNode,
    expanded: &HashSet<usize>,
    depth: usize,
    is_last: bool,
    parent_continuations: &HashSet<usize>,
    counter: &mut usize,
    entries: &mut Vec<FlatEntry>,
) {
    let node_id = *counter;
    *counter += 1;

    let is_dir = node.path.is_dir();
    let has_children = !node.children.is_empty();
    let is_expanded = expanded.contains(&node_id);

    let mut continuation_depths = parent_continuations.clone();
    if !is_last && depth > 0 {
        continuation_depths.insert(depth);
    }

    entries.push(FlatEntry {
        name: node.name.clone(),
        path: node.path.clone(),
        depth,
        is_dir,
        is_expanded,
        has_children,
        is_last_sibling: is_last,
        node_id,
        continuation_depths: continuation_depths.clone(),
    });

    if is_expanded && is_dir {
        let child_count = node.children.len();
        // For child continuations, if this node is NOT the last sibling,
        // we need to continue drawing the vertical line at this depth
        let mut child_continuations = parent_continuations.clone();
        if !is_last && depth > 0 {
            child_continuations.insert(depth);
        }
        for (i, child) in node.children.iter().enumerate() {
            let child_is_last = i == child_count - 1;
            flatten_recursive(
                child,
                expanded,
                depth + 1,
                child_is_last,
                &child_continuations,
                counter,
                entries,
            );
        }
    } else if !is_expanded && is_dir && has_children {
        // Skip children but still count them for stable node_ids
        count_nodes(&node.children, counter);
    }
}

fn count_nodes(children: &[TreeNode], counter: &mut usize) {
    for child in children {
        *counter += 1;
        count_nodes(&child.children, counter);
    }
}

fn collect_matches(tree: &TreeNode, query: &str) -> Vec<usize> {
    let mut matches = Vec::new();
    let mut counter = 0usize;
    collect_matches_recursive(tree, query, &mut counter, &mut matches);
    matches
}

fn collect_matches_recursive(
    node: &TreeNode,
    query: &str,
    counter: &mut usize,
    matches: &mut Vec<usize>,
) {
    let node_id = *counter;
    *counter += 1;

    let name_lower = node.name.to_lowercase();
    let query_lower = query.to_lowercase();

    let is_match = name_lower.contains(&query_lower) || {
        let dist = levenshtein(&name_lower, &query_lower);
        let threshold = query_lower.len() / 2;
        dist <= threshold
    };

    if is_match {
        matches.push(node_id);
    }

    for child in &node.children {
        collect_matches_recursive(child, query, counter, matches);
    }
}

fn auto_expand_for_matches(
    match_ids: &[usize],
    tree: &TreeNode,
    expanded: &mut HashSet<usize>,
) {
    let match_set: HashSet<usize> = match_ids.iter().copied().collect();
    auto_expand_recursive(tree, &match_set, expanded, &mut 0);
}

fn auto_expand_recursive(
    node: &TreeNode,
    match_set: &HashSet<usize>,
    expanded: &mut HashSet<usize>,
    counter: &mut usize,
) -> bool {
    let node_id = *counter;
    *counter += 1;

    let mut has_match_below = match_set.contains(&node_id);

    for child in &node.children {
        if auto_expand_recursive(child, match_set, expanded, counter) {
            has_match_below = true;
        }
    }

    if has_match_below && !node.children.is_empty() {
        expanded.insert(node_id);
    }

    has_match_below
}

fn rgb_to_color(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}

fn style_for_entry(entry: &FlatEntry, color_map: &ColorMap) -> Style {
    if entry.is_dir {
        Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)
    } else if is_executable(&entry.path) {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        let ext = entry
            .path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        match color_map.get(&ext.to_lowercase()) {
            Some(&(r, g, b)) => Style::default().fg(rgb_to_color(r, g, b)),
            None => Style::default().fg(Color::White),
        }
    }
}

fn tree_prefix(entry: &FlatEntry) -> String {
    let mut prefix = String::new();
    for d in 1..entry.depth {
        if entry.continuation_depths.contains(&d) {
            prefix.push_str("│   ");
        } else {
            prefix.push_str("    ");
        }
    }
    if entry.depth > 0 {
        if entry.is_last_sibling {
            prefix.push_str("└── ");
        } else {
            prefix.push_str("├── ");
        }
    }
    prefix
}

fn dir_indicator(entry: &FlatEntry) -> &'static str {
    if entry.is_dir && !entry.is_expanded && entry.has_children {
        "▶ "
    } else {
        ""
    }
}

fn render_breadcrumb(app: &App) -> Paragraph<'static> {
    let path_str = match app.entries.get(app.cursor) {
        Some(entry) => entry.path.display().to_string(),
        None => String::new(),
    };

    let line = Line::from(vec![
        Span::styled("Kree", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("   Path: "),
        Span::styled(path_str, Style::default().fg(Color::Yellow)),
    ]);

    Paragraph::new(line).block(Block::default().borders(Borders::BOTTOM))
}

fn render_tree_panel(app: &App, area: Rect) -> Paragraph<'static> {
    let viewport_height = area.height.saturating_sub(2) as usize; // borders
    let start = app.scroll_offset;
    let end = (start + viewport_height).min(app.entries.len());

    let mut lines: Vec<Line<'static>> = Vec::new();

    for i in start..end {
        let entry = &app.entries[i];
        let prefix = tree_prefix(entry);
        let indicator = dir_indicator(entry).to_string();
        let icon = icon_for_node(&entry.path, &app.icon_map);
        let icon_str = if icon.is_empty() {
            String::new()
        } else {
            format!("{icon} ")
        };

        let name_style = style_for_entry(entry, &app.color_map);
        let is_cursor = i == app.cursor;
        let is_match = app.search_matches.contains(&i);

        let mut spans = vec![
            Span::styled(prefix, Style::default().fg(Color::DarkGray)),
            Span::styled(indicator, Style::default().fg(Color::Yellow)),
            Span::styled(icon_str, name_style),
        ];

        let mut final_style = name_style;
        if is_cursor {
            final_style = final_style
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD);
        }
        if is_match {
            final_style = final_style.add_modifier(Modifier::UNDERLINED);
        }

        spans.push(Span::styled(entry.name.clone(), final_style));
        lines.push(Line::from(spans));
    }

    Paragraph::new(lines).block(
        Block::default()
            .title(" Tree ")
            .borders(Borders::ALL),
    )
}

fn render_preview_panel(app: &App) -> Paragraph<'static> {
    let title = match app.entries.get(app.cursor) {
        Some(entry) => format!(" Preview: {} ", entry.name),
        None => " Preview ".to_string(),
    };

    let lines: Vec<Line<'static>> = if let Some(err) = &app.preview_error {
        vec![Line::from(Span::styled(
            err.clone(),
            Style::default().fg(Color::Red),
        ))]
    } else {
        app.preview_content
            .iter()
            .enumerate()
            .map(|(i, line)| {
                Line::from(vec![
                    Span::styled(
                        format!("{:>4} │ ", i + 1),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::raw(line.clone()),
                ])
            })
            .collect()
    };

    Paragraph::new(lines).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL),
    )
}

fn render_status_bar(app: &App) -> Paragraph<'static> {
    let line = match app.input_mode {
        InputMode::Normal => Line::from(vec![
            Span::styled(" q ", Style::default().fg(Color::Black).bg(Color::Cyan)),
            Span::raw(" Quit  "),
            Span::styled(" / ", Style::default().fg(Color::Black).bg(Color::Cyan)),
            Span::raw(" Search  "),
            Span::styled(" Enter ", Style::default().fg(Color::Black).bg(Color::Cyan)),
            Span::raw(" Expand  "),
            Span::styled(" h ", Style::default().fg(Color::Black).bg(Color::Cyan)),
            Span::raw(" Collapse  "),
            Span::styled(" j/k ", Style::default().fg(Color::Black).bg(Color::Cyan)),
            Span::raw(" Navigate  "),
            Span::styled(" r ", Style::default().fg(Color::Black).bg(Color::Cyan)),
            Span::raw(" Reload"),
        ]),
        InputMode::Search => Line::from(vec![
            Span::styled(" Search: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(app.search_query.clone(), Style::default().fg(Color::White)),
            Span::styled("█", Style::default().fg(Color::Yellow)),
            Span::raw("   "),
            Span::styled(" Esc ", Style::default().fg(Color::Black).bg(Color::Cyan)),
            Span::raw(" Cancel  "),
            Span::styled(" Enter ", Style::default().fg(Color::Black).bg(Color::Cyan)),
            Span::raw(" Confirm  "),
            Span::styled(" ↑/↓ ", Style::default().fg(Color::Black).bg(Color::Cyan)),
            Span::raw(" Jump match"),
        ]),
    };

    Paragraph::new(line).block(Block::default().borders(Borders::TOP))
}

fn ui(frame: &mut ratatui::Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // breadcrumb
            Constraint::Min(5),   // body
            Constraint::Length(2), // status
        ])
        .split(frame.area());

    // Breadcrumb
    frame.render_widget(render_breadcrumb(app), chunks[0]);

    // Body: tree + preview split
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    let viewport_height = body[0].height.saturating_sub(2) as usize;
    app.adjust_scroll(viewport_height);

    frame.render_widget(render_tree_panel(app, body[0]), body[0]);
    frame.render_widget(render_preview_panel(app), body[1]);

    // Status bar
    frame.render_widget(render_status_bar(app), chunks[2]);
}

pub fn run(
    tree: TreeNode,
    root_path: PathBuf,
    color_map: ColorMap,
    icon_map: IconMap,
    filter: IgnoreFilter,
    sort: SortMode,
    max_depth: u32,
) -> io::Result<()> {
    // Setup terminal
    terminal::enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;

    // Panic hook to restore terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = terminal::disable_raw_mode();
        let _ = io::stdout().execute(LeaveAlternateScreen);
        original_hook(info);
    }));

    let backend = ratatui::backend::CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(tree, root_path, color_map, icon_map, filter, sort, max_depth);

    // Main loop
    loop {
        terminal.draw(|frame| ui(frame, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            // Ctrl+C always quits
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                break;
            }

            // Calculate viewport height for page up/down
            let viewport_height = terminal.size()?.height.saturating_sub(6) as usize;

            match app.input_mode {
                InputMode::Normal => app.handle_normal_key(key.code, viewport_height),
                InputMode::Search => app.handle_search_key(key.code),
            }

            if app.should_quit {
                break;
            }
        }
    }

    // Restore terminal
    terminal::disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}
