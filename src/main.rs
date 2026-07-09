mod engine;
mod games;
mod ui;
mod profile;
mod achievements;
mod leaderboard;
mod settings;
mod save;

use engine::Engine;
use engine::scene::{Scene, SceneType};
use engine::input::{self, InputEvent};
use engine::renderer;
use games::pong::PongGame;
use games::snake::SnakeGame;
use games::chess::ChessGame;
use games::tetris::TetrisGame;
use games::breakout::BreakoutGame;
use games::minesweeper::MinesweeperGame;
use games::sudoku::SudokuGame;
use games::game2048::Game2048;
use ui::menu::SplashScene;
use ui::{MainMenuScene, SettingsScene};
use ratatui::{Terminal, backend::CrosstermBackend, layout::Rect};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode};
use crossterm::execute;
use std::io;

fn main() -> io::Result<()> {
    color_eyre::install().ok();
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let mut engine = Engine::new();
    engine.terminal_size = { let s = terminal.size()?; (s.width, s.height) };

    let mut splash = SplashScene::new();
    let mut menu = MainMenuScene::new();
    let mut settings = SettingsScene::new();
    let mut in_splash = true;
    let mut in_settings = false;

    fn make_game(id: &str) -> Box<dyn Scene> {
        match id {
            "pong" => Box::new(PongGame::new()),
            "snake" => Box::new(SnakeGame::new()),
            "chess" => Box::new(ChessGame::new()),
            "tetris" => Box::new(TetrisGame::new()),
            "breakout" => Box::new(BreakoutGame::new()),
            "minesweeper" => Box::new(MinesweeperGame::new()),
            "sudoku" => Box::new(SudokuGame::new()),
            "2048" => Box::new(Game2048::new()),
            _ => Box::new(PongGame::new()),
        }
    }

    let game_ids: Vec<&str> = vec!["pong","snake","chess","tetris","breakout","minesweeper","sudoku","2048"];
    let mut scenes: Vec<Box<dyn Scene>> = Vec::new();

    let mut last_time = std::time::Instant::now();
    let mut total_time = 0.0f32;

    while engine.running {
        let frame_start = std::time::Instant::now();
        let dt = (frame_start - last_time).as_secs_f32().min(0.05);
        last_time = frame_start;
        total_time += dt;

        let target_frame_time = 1.0 / 60.0;
        let poll_timeout = (target_frame_time * 1000.0) as u128;

        match input::poll_input(poll_timeout) {
            InputEvent::Resize(w, h) => {
                engine.resize(w, h);
                if let Err(e) = terminal.resize(ratatui::layout::Rect::new(0, 0, w, h)) {
                    let _ = e;
                }
            }
            InputEvent::Key(key, ch) => {
                if in_splash {
                    in_splash = false;
                    continue;
                }

                if in_settings {
                    if key == 27 {
                        in_settings = false;
                        engine.settings.set("theme", &settings.selected_theme_id);
                        engine.theme.set_by_id(&settings.selected_theme_id);
                    } else {
                        settings.handle_key(key, ch);
                        engine.settings.set("theme", &settings.selected_theme_id);
                        engine.theme.set_by_id(&settings.selected_theme_id);
                    }
                    continue;
                }

                match key {
                    27 => {
                        if scenes.is_empty() {
                            if menu.in_submenu {
                                menu.in_submenu = false;
                            } else {
                                engine.stop();
                            }
                        } else {
                            scenes.pop();
                        }
                    }
                    10 => {
                        if let Some(scene) = scenes.last() {
                            if scene.scene_type() == SceneType::MainMenu && menu.in_submenu {
                                match menu.sub_selected {
                                    0 => { in_settings = true; settings.refresh(&engine); }
                                    1 => {}
                                    2 => {}
                                    3 => engine.stop(),
                                    _ => {}
                                }
                            }
                        } else {
                            if menu.in_submenu {
                                match menu.sub_selected {
                                    0 => { in_settings = true; settings.refresh(&engine); }
                                    1 => {}
                                    2 => {}
                                    3 => engine.stop(),
                                    _ => {}
                                }
                            } else if menu.selected < game_ids.len() {
                                let mut g = make_game(game_ids[menu.selected]);
                                let (w, h) = engine.terminal_size;
                                g.set_terminal_size(w, h);
                                g.init();
                                g.enter();
                                scenes.push(g);
                            }
                        }
                    }
                    _ => {
                        menu.handle_key(key, ch);
                        if let Some(scene) = scenes.last_mut() {
                            scene.handle_key(key, ch);
                        }
                    }
                }
            }
            InputEvent::None => {}
        }

        if !in_splash && !in_settings {
            if let Some(scene) = scenes.last_mut() {
                scene.update(dt);
            }
        }

        if in_settings {
            settings.update(dt);
        }

        let colors = engine.theme.colors();
        let _ = terminal.draw(|frame| {
            let full = frame.area();
            let buf = frame.buffer_mut();

            renderer::draw_starfield(buf, full, total_time);

            let cw = (full.width * 75 / 100).max(50);
            let ch = (full.height * 90 / 100).max(20);
            let cx = full.x + (full.width - cw) / 2;
            let cy = full.y + (full.height - ch) / 2;
            let cabinet = Rect::new(cx, cy, cw, ch);
            let interior = Rect::new(cabinet.x + 1, cabinet.y + 1, cabinet.width - 2, cabinet.height - 2);

            renderer::fill_rect(buf, interior, colors.bg);
            renderer::draw_arcade_cabinet(buf, cabinet, colors.fg, colors.accent, colors.bg);

            if in_splash {
                splash.render(frame, &engine, interior);
            } else if in_settings {
                settings.render(frame, &engine, interior);
            } else if let Some(scene) = scenes.last_mut() {
                scene.render(frame, &engine, interior);
            } else {
                menu.render(frame, &engine, interior);
            }
        });

        let frame_elapsed = frame_start.elapsed().as_secs_f32();
        if frame_elapsed < target_frame_time {
            let remaining = target_frame_time - frame_elapsed;
            if remaining > 0.001 {
                std::thread::sleep(std::time::Duration::from_secs_f32(remaining));
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
