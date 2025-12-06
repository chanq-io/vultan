use crate::state::card::score::Score;
use crate::state::card::Card;
use crate::state::deck::Deck;
use crate::state::hand::Hand;
use crate::state::hand::HandError;
use itertools::*;
//TODO use syntect_tui::into_span;
use anyhow::Result;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect_tui::into_span;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
    Frame, Terminal,
};

fn highlight(s: &str) -> tui::text::Text<'_> {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let syntax = ps.find_syntax_by_extension("md").unwrap();
    let mut theme = ts.themes["base16-eighties.dark"].to_owned();
    theme.settings.background = Some(syntect::highlighting::Color {
        r: 43,
        g: 48,
        b: 59,
        a: 0,
    });
    let mut h = HighlightLines::new(syntax, &theme);
    Text::from(
        s.lines()
            .map(|line| {
                let line_spans: Vec<Span> = h
                    .highlight_line(line, &ps)
                    .unwrap()
                    .into_iter()
                    .filter_map(|segment| into_span(segment).ok())
                    .collect();
                Spans::from(line_spans)
            })
            .collect_vec(),
    )
}

enum RevisingMode {
    Question,
    Answer,
}

struct App {
    revision_mode: RevisingMode,
}

impl Default for App {
    fn default() -> App {
        App {
            revision_mode: RevisingMode::Question,
        }
    }
}

fn read_score_callback<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    deck: &Deck,
    card: &Card,
    n_due: usize,
    n_remaining: usize,
) -> Result<Score> {
    loop {
        terminal.draw(|f| ui(f, app, deck, card, n_due, n_remaining))?;
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('1') => {
                    if matches!(app.revision_mode, RevisingMode::Answer) {
                        app.revision_mode = RevisingMode::Question;
                        return Ok(Score::Fail);
                    }
                }
                KeyCode::Char('2') => {
                    if matches!(app.revision_mode, RevisingMode::Answer) {
                        app.revision_mode = RevisingMode::Question;
                        return Ok(Score::Hard);
                    }
                }
                KeyCode::Char('3') => {
                    if matches!(app.revision_mode, RevisingMode::Answer) {
                        app.revision_mode = RevisingMode::Question;
                        return Ok(Score::Pass);
                    }
                }
                KeyCode::Char('4') => {
                    if matches!(app.revision_mode, RevisingMode::Answer) {
                        app.revision_mode = RevisingMode::Question;
                        return Ok(Score::Easy);
                    }
                }
                KeyCode::Char('A') | KeyCode::Char('a') => {
                    if matches!(app.revision_mode, RevisingMode::Question) {
                        app.revision_mode = RevisingMode::Answer;
                    }
                }
                KeyCode::Char('Q') | KeyCode::Char('q') => {
                    return Err(HandError::ReceivedExitApplicationSignal.into())
                }
                _ => {}
            }
        }
    }
}

pub fn run(deck: &Deck, hand: Hand) -> Result<Vec<Card>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let n_due = hand.number_of_due_cards();
    let mut app = App::default();
    let revised_cards = hand.revise_until_none_fail(|card, n_remaining| {
        read_score_callback(&mut terminal, &mut app, deck, card, n_due, n_remaining)
    });

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    revised_cards
}

fn styled_title(text: &str) -> Span<'_> {
    Span::styled(
        format!(" {} ", text),
        Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD),
    )
}
fn ui<B: Backend>(
    f: &mut Frame<B>,
    app: &App,
    deck: &Deck,
    card: &Card,
    n_due: usize,
    n_remaining: usize,
) {
    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(10)
        .horizontal_margin(20)
        .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
        .split(f.size());

    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)].as_ref())
        .split(vertical_layout[0]);

    let info_view = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(horizontal_layout[1]);

    let bold_style = Style::default().add_modifier(Modifier::BOLD);
    let deck_name_info_prefix = Span::styled("CURRENTLY REVISING DECK:       ", bold_style);
    let deck_name_info_suffix = Span::from(deck.name.clone());
    let n_cards_in_deck_prefix = Span::styled("TOTAL NUMBER OF CARDS IN DECK: ", bold_style);
    let n_cards_in_deck_suffix = Span::from(format!("{}", deck.card_paths.len()));
    let n_cards_to_revise_prefix = Span::styled("NUMBER OF CARDS TO BE REVISED: ", bold_style);
    let n_cards_to_revise_suffix = Span::from(format!("{}", n_due));

    let deck_view = Paragraph::new(Text::from(vec![
        Spans::from(vec![deck_name_info_prefix, deck_name_info_suffix]),
        Spans::from(vec![n_cards_in_deck_prefix, n_cards_in_deck_suffix]),
        Spans::from(vec![n_cards_to_revise_prefix, n_cards_to_revise_suffix]),
    ]))
    .block(
        Block::default()
            .title(styled_title("DECK INFO"))
            .borders(Borders::ALL),
    )
    .alignment(Alignment::Left)
    .wrap(Wrap { trim: true });

    let fail_command = Span::styled("[1] FAIL: ", Style::default().add_modifier(Modifier::BOLD));
    let fail_instruction =
        Span::from("You failed to answer the question and want to revise it again this session.");
    let hard_command = Span::styled("[2] HARD: ", Style::default().add_modifier(Modifier::BOLD));
    let hard_instruction = Span::from("It took you a long time to answer the question.");
    let pass_command = Span::styled("[3] PASS: ", Style::default().add_modifier(Modifier::BOLD));
    let pass_instruction = Span::from("It took a short amount of time to answer the question.");
    let easy_command = Span::styled("[4] EASY: ", Style::default().add_modifier(Modifier::BOLD));
    let easy_instruction = Span::from("You answered the question almost immediately.");
    let answer_command = Span::styled(
        "[A] ANSWER: ",
        Style::default().add_modifier(Modifier::BOLD),
    );
    let answer_instruction = Span::from("Reveal the answer to the current question.");
    let quit_command = Span::styled("[Q] QUIT: ", Style::default().add_modifier(Modifier::BOLD));
    let quit_instruction = Span::from("Exit the application.");
    let reviseing_question_instructions = vec![
        Spans::from(vec![answer_command, answer_instruction]),
        Spans::from(vec![quit_command.clone(), quit_instruction.clone()]),
    ];
    let reviseing_answer_instructions = vec![
        Spans::from(vec![fail_command, fail_instruction]),
        Spans::from(vec![hard_command, hard_instruction]),
        Spans::from(vec![pass_command, pass_instruction]),
        Spans::from(vec![easy_command, easy_instruction]),
        Spans::from(vec![quit_command, quit_instruction]),
    ];
    let instructions = match app.revision_mode {
        RevisingMode::Question => reviseing_question_instructions,
        RevisingMode::Answer => reviseing_answer_instructions,
    };
    let commands_view = Paragraph::new(Text::from(instructions))
        .block(
            Block::default()
                .title(styled_title("COMMANDS"))
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });

    let (question_answer_title, question_answer_content) = match app.revision_mode {
        RevisingMode::Question => ("QUESTION", card.question.as_str()),
        RevisingMode::Answer => ("ANSWER", card.answer.as_str()),
    };
    let question_answer_view = Paragraph::new(highlight(question_answer_content))
        .block(
            Block::default()
                .title(styled_title(question_answer_title))
                .borders(Borders::ALL),
        )
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    let n_due = n_due as f64;
    let n_remaining = n_remaining as f64;
    let n_revised = n_due - n_remaining;
    let progress = (n_revised / n_due) * 100.0;
    let progress_view = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(styled_title("QUEUE PROGRESS")),
        )
        .gauge_style(
            Style::default()
                .fg(Color::White)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .label(format!(
            "{} card{} revised",
            n_revised,
            if n_revised == 1.0 { "" } else { "s" }
        ))
        .percent(progress as u16);

    f.render_widget(question_answer_view, horizontal_layout[0]);
    f.render_widget(progress_view, vertical_layout[1]);
    f.render_widget(deck_view, info_view[0]);
    f.render_widget(commands_view, info_view[1]);
}
