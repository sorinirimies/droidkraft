use crate::adb::AdbCommand;
use crate::fastboot::FastbootManager;
use crate::menu::MenuCommand;
use crate::message::{CommandResult, Message};
use crate::model::{AppState, Model};

/// Update function - the heart of Elm architecture
/// Takes the current model and a message, returns updated model
/// This is a pure function that handles all state transitions
pub async fn update(model: &mut Model, message: Message) {
    match message {
        // Navigation messages
        Message::MenuUp => {
            model.menu.previous();
        }

        Message::MenuDown => {
            model.menu.next();
        }

        Message::SectionNext => {
            model.menu.next_section();
        }

        Message::SectionPrev => {
            model.menu.previous_section();
        }

        Message::RefreshDeviceInfo => {
            let status = model.adb_manager.fetch_device_status();
            model.device_status = status;
        }

        // Pushed by the background device-watcher task whenever the set of
        // connected devices changes (connect or disconnect event).
        Message::DeviceStatusUpdate(mut new_status) => {
            // Try to keep the device the user had selected.  The watcher
            // always creates a fresh AdbManager (no selection memory), so we
            // re-apply the previously active serial if it still exists in the
            // new device list.
            if let Some(prev_serial) = model.device_status.active().map(|d| d.serial.clone()) {
                if let Some(idx) = new_status
                    .devices
                    .iter()
                    .position(|d| d.serial == prev_serial)
                {
                    new_status.selected_idx = idx;
                }
            }

            // Keep the AdbManager's internal selection in sync so that
            // subsequent manual commands target the right device.
            if let Some(active) = new_status.active() {
                model.adb_manager.select_device(active.serial.clone());
            }

            model.device_status = new_status;
        }

        Message::NextDevice => {
            model.device_status.cycle_next();
            let status = model.adb_manager.fetch_device_status();
            model.device_status = status;
        }

        Message::EnterChild => {
            // Flat menu has no child mode — trigger visual effects only
            model.effects.start_fade_in();
            model.effects.start_slide_in();
        }

        Message::ExitChild => {
            // No-op: flat menu has no child mode
        }

        // Command execution
        Message::ExecuteCommand(command) => {
            // Special handling for logcat — bypass normal command execution
            if matches!(command, MenuCommand::OpenLogcat) {
                model.state = AppState::Logcat;
                if let Some(dev) = model.device_status.active() {
                    model.logcat.start_streaming(dev.serial.clone());
                } else {
                    model.logcat.status_message =
                        Some("No device connected — connect a device first.".to_string());
                }
                return;
            }

            if matches!(command, MenuCommand::OpenDevMode) {
                model.state = AppState::DevMode;
                return;
            }

            if matches!(command, MenuCommand::OpenRomFlash) {
                model.state = AppState::RomFlash;
                return;
            }

            model.last_command_label = Some(model.menu.get_selected_label().to_string());
            model.state = AppState::Loading;
            model.clear_results();
            model.loading_counter = 0;
            model.effects.reset_slide();

            let result = execute_adb_command(model, command).await;

            // Handle result directly to avoid recursion
            match result {
                CommandResult::Success(output) => {
                    model.set_result(output);
                }
                CommandResult::Error(error) => {
                    model.set_error(error);
                }
            }
            model.state = AppState::ShowResult;
            model.effects.start_slide_in();
        }

        Message::CommandStarted => {
            model.state = AppState::Loading;
            model.loading_counter = 0;
        }

        Message::CommandCompleted(result) => {
            match result {
                CommandResult::Success(output) => {
                    model.set_result(output);
                }
                CommandResult::Error(error) => {
                    model.set_error(error);
                }
            }
            model.state = AppState::ShowResult;
            model.effects.start_slide_in();
        }

        // Scroll messages
        Message::ScrollUp => {
            if model.scroll_position > 0 {
                model.scroll_position -= 1;
            }
        }

        Message::ScrollDown => {
            if model.scroll_position + 1 < model.total_result_lines() {
                model.scroll_position += 1;
            }
        }

        Message::ScrollPageUp => {
            model.scroll_position = model.scroll_position.saturating_sub(10);
        }

        Message::ScrollPageDown => {
            let max_scroll = model.total_result_lines().saturating_sub(1);
            model.scroll_position = (model.scroll_position + 10).min(max_scroll);
        }

        Message::ScrollToTop => {
            model.scroll_position = 0;
        }

        Message::ScrollToBottom => {
            model.scroll_position = model.total_result_lines().saturating_sub(1);
        }

        // Screen streaming messages

        // ── Logcat ────────────────────────────────────────────────────────
        Message::OpenLogcat => {
            model.state = AppState::Logcat;
            if let Some(dev) = model.device_status.active() {
                model.logcat.start_streaming(dev.serial.clone());
            } else {
                model.logcat.status_message =
                    Some("No device connected — connect a device first.".to_string());
            }
        }

        Message::CloseLogcat => {
            model.logcat.stop_streaming();
            model.state = AppState::Menu;
            model.needs_device_refresh = true;
        }

        Message::LogcatScrollUp => {
            model.logcat.scroll_up(1);
        }

        Message::LogcatScrollDown => {
            model.logcat.scroll_down(1);
        }

        Message::LogcatScrollPageUp => {
            model.logcat.scroll_up(20);
        }

        Message::LogcatScrollPageDown => {
            model.logcat.scroll_down(20);
        }

        Message::LogcatScrollToTop => {
            model.logcat.scroll_to_top();
        }

        Message::LogcatScrollToBottom => {
            model.logcat.scroll_to_bottom();
        }

        Message::LogcatTogglePause => {
            model.logcat.toggle_pause();
        }

        Message::LogcatClear => {
            model.logcat.clear();
        }

        Message::LogcatCycleLevel => {
            model.logcat.filter.cycle_level();
            model.logcat.rebuild_filtered();
        }

        Message::LogcatToggleSearch => {
            use crate::logcat::FilterField;
            if model.logcat.filter.active_field == FilterField::Search {
                model.logcat.filter.active_field = FilterField::None;
            } else {
                model.logcat.filter.active_field = FilterField::Search;
            }
        }

        Message::LogcatToggleTagFilter => {
            use crate::logcat::FilterField;
            if model.logcat.filter.active_field == FilterField::Tag {
                model.logcat.filter.active_field = FilterField::None;
            } else {
                model.logcat.filter.active_field = FilterField::Tag;
            }
        }

        Message::LogcatTogglePackageFilter => {
            use crate::logcat::FilterField;
            if model.logcat.filter.active_field == FilterField::Package {
                model.logcat.filter.active_field = FilterField::None;
            } else {
                model.logcat.filter.active_field = FilterField::Package;
            }
        }

        Message::LogcatSearchInput(c) => {
            if model.logcat_save_active {
                let pos = model.logcat_save_cursor;
                let byte_idx = model
                    .logcat_save_path
                    .char_indices()
                    .nth(pos)
                    .map(|(i, _)| i)
                    .unwrap_or(model.logcat_save_path.len());
                model.logcat_save_path.insert(byte_idx, c);
                model.logcat_save_cursor += 1;
            } else {
                model.logcat.filter.insert_char(c);
                model.logcat.rebuild_filtered();
            }
        }

        Message::LogcatSearchBackspace => {
            if model.logcat_save_active {
                if model.logcat_save_cursor > 0 {
                    let pos = model.logcat_save_cursor;
                    let byte_idx = model
                        .logcat_save_path
                        .char_indices()
                        .nth(pos - 1)
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    let next_byte = model
                        .logcat_save_path
                        .char_indices()
                        .nth(pos)
                        .map(|(i, _)| i)
                        .unwrap_or(model.logcat_save_path.len());
                    model.logcat_save_path.drain(byte_idx..next_byte);
                    model.logcat_save_cursor -= 1;
                }
            } else {
                model.logcat.filter.delete_char();
                model.logcat.rebuild_filtered();
            }
        }

        Message::LogcatSearchDelete => {
            if model.logcat_save_active {
                let pos = model.logcat_save_cursor;
                let max = model.logcat_save_path.chars().count();
                if pos < max {
                    let byte_idx = model
                        .logcat_save_path
                        .char_indices()
                        .nth(pos)
                        .map(|(i, _)| i)
                        .unwrap_or(model.logcat_save_path.len());
                    let next_byte = model
                        .logcat_save_path
                        .char_indices()
                        .nth(pos + 1)
                        .map(|(i, _)| i)
                        .unwrap_or(model.logcat_save_path.len());
                    model.logcat_save_path.drain(byte_idx..next_byte);
                }
            } else {
                model.logcat.filter.delete_char_forward();
                model.logcat.rebuild_filtered();
            }
        }

        Message::LogcatCursorLeft => {
            if model.logcat_save_active {
                if model.logcat_save_cursor > 0 {
                    model.logcat_save_cursor -= 1;
                }
            } else {
                model.logcat.filter.move_cursor_left();
            }
        }

        Message::LogcatCursorRight => {
            if model.logcat_save_active {
                let max = model.logcat_save_path.chars().count();
                if model.logcat_save_cursor < max {
                    model.logcat_save_cursor += 1;
                }
            } else {
                model.logcat.filter.move_cursor_right();
            }
        }

        Message::LogcatExitFilter => {
            use crate::logcat::FilterField;
            model.logcat.filter.active_field = FilterField::None;
        }

        Message::LogcatToggleWordWrap => {
            model.logcat.toggle_word_wrap();
        }

        Message::LogcatSave => {
            model.logcat_save_active = true;
            model.logcat_save_filtered_only = false;
            let filename =
                crate::logcat::LogcatState::default_save_filename(model.logcat_save_format);
            model.logcat_save_path = filename;
            model.logcat_save_cursor = model.logcat_save_path.chars().count();
        }

        Message::LogcatSaveFilteredOnly => {
            if model.logcat_save_active {
                // Cycle: TXT all → TXT filtered → JSON all → JSON filtered
                if model.logcat_save_filtered_only {
                    // Was filtered, switch to next format, all
                    model.logcat_save_filtered_only = false;
                    model.logcat_save_format = model.logcat_save_format.cycle();
                } else {
                    // Was all, switch to filtered same format
                    model.logcat_save_filtered_only = true;
                }
                // Update filename extension
                let filename =
                    crate::logcat::LogcatState::default_save_filename(model.logcat_save_format);
                model.logcat_save_path = filename;
                model.logcat_save_cursor = model.logcat_save_path.chars().count();
            } else {
                model.logcat_save_active = true;
                model.logcat_save_filtered_only = true;
                let filename =
                    crate::logcat::LogcatState::default_save_filename(model.logcat_save_format);
                model.logcat_save_path = filename;
                model.logcat_save_cursor = model.logcat_save_path.chars().count();
            }
        }

        Message::LogcatCancelSave => {
            use crate::model::LogcatSaveMode;
            if model.logcat_save_mode == LogcatSaveMode::FileBrowser {
                // Go back to path input mode instead of closing
                model.logcat_save_mode = LogcatSaveMode::PathInput;
            } else {
                model.logcat_save_active = false;
                model.logcat_save_path.clear();
                model.logcat_save_cursor = 0;
                model.logcat_save_mode = LogcatSaveMode::PathInput;
                model.logcat_file_explorer = None;
            }
        }

        Message::LogcatFileSaved(path_str) => {
            model.logcat_save_active = false;
            let path = std::path::PathBuf::from(&path_str);
            let result = match (model.logcat_save_filtered_only, model.logcat_save_format) {
                (true, crate::logcat::SaveFormat::Json) => {
                    model.logcat.save_filtered_to_json_file(&path)
                }
                (false, crate::logcat::SaveFormat::Json) => model.logcat.save_to_json_file(&path),
                (true, crate::logcat::SaveFormat::Text) => {
                    model.logcat.save_filtered_to_file(&path)
                }
                (false, crate::logcat::SaveFormat::Text) => model.logcat.save_to_file(&path),
            };
            match result {
                Ok(count) => {
                    let kind = if model.logcat_save_filtered_only {
                        "filtered"
                    } else {
                        "all"
                    };
                    let fmt = model.logcat_save_format.label();
                    model.logcat.status_message = Some(format!(
                        "\u{2705} Saved {} {} entries as {} to {}",
                        count,
                        kind,
                        fmt,
                        path.display()
                    ));
                }
                Err(e) => {
                    model.logcat.status_message = Some(format!("\u{274c} Save failed: {}", e));
                }
            }
            model.logcat_save_path.clear();
            model.logcat_save_cursor = 0;
        }

        Message::LogcatSaveAs => {
            use crate::model::LogcatSaveMode;
            model.logcat_save_mode = LogcatSaveMode::FileBrowser;
            if model.logcat_file_explorer.is_none() {
                let start_dir = std::env::current_dir().unwrap_or_else(|_| {
                    std::path::PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/".into()))
                });
                let explorer = tui_file_explorer::FileExplorer::new(start_dir, vec![]);
                model.logcat_file_explorer = Some(explorer);
            }
        }

        // -- New logcat feature messages ------------------------------------
        Message::LogcatToggleRegex => {
            model.logcat.filter.toggle_regex();
            model.logcat.rebuild_filtered();
        }
        Message::LogcatToggleExclude => {
            use crate::logcat::FilterField;
            if model.logcat.filter.active_field == FilterField::Exclude {
                model.logcat.filter.active_field = FilterField::None;
            } else {
                model.logcat.filter.active_field = FilterField::Exclude;
            }
        }
        Message::LogcatToggleCompact => {
            model.logcat.toggle_compact();
        }
        Message::LogcatToggleDetail => {
            model.logcat.toggle_detail();
        }
        Message::LogcatBookmarkToggle => {
            model.logcat.toggle_bookmark();
        }
        Message::LogcatBookmarkNext => {
            model.logcat.next_bookmark();
        }
        Message::LogcatBookmarkPrev => {
            model.logcat.prev_bookmark();
        }
        Message::LogcatHScrollLeft => {
            model.logcat.h_scroll_left(4);
        }
        Message::LogcatHScrollRight => {
            model.logcat.h_scroll_right(4);
        }
        Message::LogcatHScrollReset => {
            model.logcat.h_scroll_reset();
        }
        Message::LogcatCopyLine => match model.logcat.copy_selected_to_clipboard() {
            Ok(()) => {
                model.logcat.status_message = Some("Line copied to clipboard.".to_string());
            }
            Err(e) => {
                model.logcat.status_message = Some(format!("Copy failed: {}", e));
            }
        },
        Message::LogcatToggleFold => {
            model.logcat.toggle_fold_at_selected();
        }
        Message::LogcatSelectUp => {
            model.logcat.select_up();
        }
        Message::LogcatSelectDown => {
            model.logcat.select_down();
        }

        Message::LogcatFileExplorerKey(key_event) => {
            use crate::model::LogcatSaveMode;
            use ratatui::crossterm::event::KeyCode;

            if let Some(ref mut explorer) = model.logcat_file_explorer {
                // tui-file-explorer uses crossterm 0.29 while we use 0.28.
                // We can't pass our KeyEvent directly, so we map our
                // KeyCode to the explorer's public API manually.
                let outcome = match key_event.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        explorer.cursor = explorer.cursor.saturating_sub(1);
                        tui_file_explorer::ExplorerOutcome::Pending
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if !explorer.entries.is_empty()
                            && explorer.cursor < explorer.entries.len() - 1
                        {
                            explorer.cursor += 1;
                        }
                        tui_file_explorer::ExplorerOutcome::Pending
                    }
                    KeyCode::PageUp => {
                        explorer.cursor = explorer.cursor.saturating_sub(10);
                        tui_file_explorer::ExplorerOutcome::Pending
                    }
                    KeyCode::PageDown => {
                        if !explorer.entries.is_empty() {
                            explorer.cursor =
                                (explorer.cursor + 10).min(explorer.entries.len() - 1);
                        }
                        tui_file_explorer::ExplorerOutcome::Pending
                    }
                    KeyCode::Home | KeyCode::Char('g') => {
                        explorer.cursor = 0;
                        tui_file_explorer::ExplorerOutcome::Pending
                    }
                    KeyCode::End | KeyCode::Char('G') => {
                        if !explorer.entries.is_empty() {
                            explorer.cursor = explorer.entries.len() - 1;
                        }
                        tui_file_explorer::ExplorerOutcome::Pending
                    }
                    KeyCode::Enter | KeyCode::Char('l') => {
                        if explorer.mkdir_active {
                            // Confirm mkdir
                            let name = explorer.mkdir_input.trim().to_string();
                            explorer.mkdir_active = false;
                            explorer.mkdir_input.clear();
                            if !name.is_empty() {
                                let new_dir = explorer.current_dir.join(&name);
                                if std::fs::create_dir_all(&new_dir).is_ok() {
                                    explorer.reload();
                                    if let Some(idx) =
                                        explorer.entries.iter().position(|e| e.path == new_dir)
                                    {
                                        explorer.cursor = idx;
                                    }
                                }
                            }
                            tui_file_explorer::ExplorerOutcome::Pending
                        } else if let Some(entry) = explorer.entries.get(explorer.cursor) {
                            // Descend into dir or select file
                            if entry.is_dir {
                                let path = entry.path.clone();
                                explorer.navigate_to(path);
                                tui_file_explorer::ExplorerOutcome::Pending
                            } else {
                                tui_file_explorer::ExplorerOutcome::Selected(entry.path.clone())
                            }
                        } else {
                            tui_file_explorer::ExplorerOutcome::Pending
                        }
                    }
                    KeyCode::Right => {
                        // Navigate into dir only
                        if let Some(entry) = explorer.entries.get(explorer.cursor) {
                            if entry.is_dir {
                                let path = entry.path.clone();
                                explorer.navigate_to(path);
                            }
                        }
                        tui_file_explorer::ExplorerOutcome::Pending
                    }
                    KeyCode::Left | KeyCode::Backspace | KeyCode::Char('h') => {
                        if explorer.mkdir_active {
                            explorer.mkdir_input.pop();
                        } else if explorer.search_active {
                            if explorer.search_query.is_empty() {
                                explorer.search_active = false;
                            } else {
                                explorer.search_query.pop();
                                explorer.cursor = 0;
                                explorer.reload();
                            }
                        } else {
                            // Ascend to parent
                            if let Some(parent) =
                                explorer.current_dir.parent().map(|p| p.to_path_buf())
                            {
                                let prev = explorer.current_dir.clone();
                                explorer.navigate_to(parent);
                                // Try to land cursor on the directory we came from
                                if let Some(idx) =
                                    explorer.entries.iter().position(|e| e.path == prev)
                                {
                                    explorer.cursor = idx;
                                }
                            }
                        }
                        tui_file_explorer::ExplorerOutcome::Pending
                    }
                    KeyCode::Char('/') => {
                        explorer.search_active = true;
                        tui_file_explorer::ExplorerOutcome::Pending
                    }
                    KeyCode::Char('.') => {
                        explorer.show_hidden = !explorer.show_hidden;
                        explorer.reload();
                        tui_file_explorer::ExplorerOutcome::Pending
                    }
                    KeyCode::Char('s') => {
                        explorer.sort_mode = explorer.sort_mode.next();
                        explorer.reload();
                        tui_file_explorer::ExplorerOutcome::Pending
                    }
                    KeyCode::Char('n') => {
                        explorer.mkdir_active = true;
                        explorer.mkdir_input.clear();
                        tui_file_explorer::ExplorerOutcome::Pending
                    }
                    KeyCode::Char('S') => {
                        // "Save Here" — save into the current directory
                        let filename = crate::logcat::LogcatState::default_save_filename(
                            crate::logcat::SaveFormat::Text,
                        );
                        let save_path = explorer.current_dir.join(filename);
                        tui_file_explorer::ExplorerOutcome::Selected(save_path)
                    }
                    KeyCode::Char(c) if explorer.search_active => {
                        explorer.search_query.push(c);
                        explorer.cursor = 0;
                        explorer.reload();
                        tui_file_explorer::ExplorerOutcome::Pending
                    }
                    KeyCode::Char(c) if explorer.mkdir_active => {
                        explorer.mkdir_input.push(c);
                        tui_file_explorer::ExplorerOutcome::Pending
                    }
                    KeyCode::Esc => {
                        if explorer.search_active {
                            explorer.search_active = false;
                            explorer.search_query.clear();
                            explorer.reload();
                            tui_file_explorer::ExplorerOutcome::Pending
                        } else if explorer.mkdir_active {
                            explorer.mkdir_active = false;
                            explorer.mkdir_input.clear();
                            tui_file_explorer::ExplorerOutcome::Pending
                        } else {
                            tui_file_explorer::ExplorerOutcome::Dismissed
                        }
                    }
                    KeyCode::Char('q') => tui_file_explorer::ExplorerOutcome::Dismissed,
                    _ => tui_file_explorer::ExplorerOutcome::Pending,
                };

                match outcome {
                    tui_file_explorer::ExplorerOutcome::Selected(path) => {
                        // User selected a file — use its path as the save target
                        let path_str = path.display().to_string();
                        model.logcat_save_path = path_str;
                        model.logcat_save_cursor = model.logcat_save_path.chars().count();
                        model.logcat_save_mode = LogcatSaveMode::PathInput;
                    }
                    tui_file_explorer::ExplorerOutcome::Dismissed => {
                        model.logcat_save_mode = LogcatSaveMode::PathInput;
                    }
                    _ => {}
                }
            }
        }

        // ── Dev Tools ─────────────────────────────────────────────────────
        Message::OpenDevMode => {
            model.state = AppState::DevMode;
        }

        Message::CloseDevMode => {
            model.state = AppState::Menu;
            model.needs_device_refresh = true;
        }

        // ── Custom-ROM flasher ────────────────────────────────────────────
        Message::OpenRomFlash => {
            model.state = AppState::RomFlash;
        }
        Message::CloseRomFlash => {
            model.state = AppState::Menu;
            model.needs_device_refresh = true;
        }
        Message::RomDetect => model.rom_flash.detect(),
        Message::RomSelectUp => model.rom_flash.select_up(),
        Message::RomSelectDown => model.rom_flash.select_down(),
        Message::RomDownload => model.rom_flash.download_selected(),
        Message::RomRunStep => {
            if model.rom_flash.next_step_needs_confirmation() {
                model.rom_flash.status =
                    "⚠ This step is destructive — press Shift+F to CONFIRM.".to_string();
            } else {
                model.rom_flash.run_next_step();
            }
        }
        Message::RomConfirmStep => model.rom_flash.run_next_step(),

        Message::DevBuild => {
            model.devtools.start_build();
        }

        Message::DevBuildAndInstall => {
            model.devtools.build_and_install();
        }

        Message::DevRun => match model.devtools.run_app() {
            Ok(()) => {}
            Err(e) => {
                model.devtools.status_message = Some(format!("❌ {}", e));
            }
        },

        Message::DevCycleFocus => {
            model.devtools.cycle_focus();
        }

        Message::DevToggleEditorPicker => {
            model.devtools.toggle_editor_picker();
        }

        Message::DevEditorUp => {
            model.devtools.editor_picker_up();
        }

        Message::DevEditorDown => {
            model.devtools.editor_picker_down();
        }

        Message::DevEditorConfirm => {
            model.devtools.editor_picker_confirm();
        }

        Message::DevToggleVariantPicker => {
            model.devtools.toggle_variant_picker();
        }

        Message::DevNextVariant => {
            model.devtools.next_variant();
        }

        Message::DevPrevVariant => {
            model.devtools.prev_variant();
        }

        Message::DevFileExplorerKey(key_event) => {
            use ratatui::crossterm::event::KeyCode;
            if let Some(ref mut explorer) = model.devtools.file_explorer {
                // Same crossterm version bridge as logcat save dialog
                match key_event.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        explorer.cursor = explorer.cursor.saturating_sub(1);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if !explorer.entries.is_empty()
                            && explorer.cursor < explorer.entries.len() - 1
                        {
                            explorer.cursor += 1;
                        }
                    }
                    KeyCode::Enter | KeyCode::Char('l') => {
                        if let Some(entry) = explorer.entries.get(explorer.cursor) {
                            if entry.is_dir {
                                let path = entry.path.clone();
                                explorer.navigate_to(path);
                            }
                            // File selection is handled by DevOpenFile
                        }
                    }
                    KeyCode::Right => {
                        if let Some(entry) = explorer.entries.get(explorer.cursor) {
                            if entry.is_dir {
                                let path = entry.path.clone();
                                explorer.navigate_to(path);
                            }
                        }
                    }
                    KeyCode::Left | KeyCode::Backspace | KeyCode::Char('h') => {
                        if let Some(parent) = explorer.current_dir.parent().map(|p| p.to_path_buf())
                        {
                            let prev = explorer.current_dir.clone();
                            explorer.navigate_to(parent);
                            if let Some(idx) = explorer.entries.iter().position(|e| e.path == prev)
                            {
                                explorer.cursor = idx;
                            }
                        }
                    }
                    KeyCode::Char('/') => {
                        explorer.search_active = true;
                    }
                    KeyCode::Char('.') => {
                        explorer.show_hidden = !explorer.show_hidden;
                        explorer.reload();
                    }
                    KeyCode::Char(c) if explorer.search_active => {
                        explorer.search_query.push(c);
                        explorer.cursor = 0;
                        explorer.reload();
                    }
                    KeyCode::Esc if explorer.search_active => {
                        explorer.search_active = false;
                        explorer.search_query.clear();
                        explorer.reload();
                    }
                    _ => {}
                }

                // After any navigation, check if we've entered a different
                // Gradle project and auto-update project_dir + app modules.
                let browser_dir = explorer.current_dir.clone();
                model.devtools.sync_project_from_browser(&browser_dir);
            }
        }

        Message::DevOpenFile => {
            // Open the selected file in the configured editor
            if let Some(_binary) = model.devtools.editor.binary() {
                if let Some(ref explorer) = model.devtools.file_explorer {
                    if let Some(entry) = explorer.entries.get(explorer.cursor) {
                        if !entry.is_dir {
                            let path = entry.path.clone();
                            model.devtools.status_message = Some(format!(
                                "Opening {} in {}…",
                                path.file_name().unwrap_or_default().to_string_lossy(),
                                model.devtools.editor.label()
                            ));
                            // The actual editor launch will be handled in app.rs
                            // since it needs terminal suspend/resume
                        }
                    }
                }
            } else {
                model.devtools.status_message = Some("No editor set — press E to pick one".into());
            }
        }

        // ── Theme ─────────────────────────────────────────────────────────
        Message::ToggleThemeSelector => {
            model.theme_selector.toggle();
        }

        Message::ThemeNext => {
            model.theme_selector.next();
            let theme = model.theme_selector.apply();
            model.theme = theme;
        }

        Message::ThemePrev => {
            model.theme_selector.prev();
            let theme = model.theme_selector.apply();
            model.theme = theme;
        }

        Message::ThemeApply => {
            let theme = model.theme_selector.apply();
            model.theme = theme;
        }

        // Application lifecycle
        Message::Tick => {
            tick(model).await;
        }

        Message::Quit => {
            model.running = false;
        }

        Message::ReturnToMenu => {
            model.state = AppState::Menu;
            model.clear_results();
            // Refresh device stats after every command so the dashboard stays current.
            model.needs_device_refresh = true;
        }

        Message::SkipStartup => {
            model.state = AppState::Menu;
            model.effects.start_slide_in();
            model.needs_device_refresh = true;
        }
    }
}

/// Handle tick updates (animations, timers, etc.)
async fn tick(model: &mut Model) {
    let now = std::time::Instant::now();
    let elapsed = now.duration_since(model.last_tick);
    model.last_tick = now;

    // Update effects
    model.effects.tick(elapsed);

    // Update menu animations
    model.menu.tick();

    // Update loading animation
    if model.state == AppState::Loading {
        model.loading_counter += 1;
    }

    // Update reveal animation for results
    if model.state == AppState::ShowResult {
        model.reveal_counter += 1;
    }

    // Poll for new logcat entries
    if model.state == AppState::Logcat {
        model.logcat.poll_new_entries();
    }

    // Poll for build output in dev mode
    if model.state == AppState::DevMode {
        model.devtools.poll_build_output();
    }

    // Poll the ROM flasher worker (detect/download/flash results)
    if model.state == AppState::RomFlash {
        model.rom_flash.poll();
    }

    // Check if startup is complete
    if model.state == AppState::Startup && model.effects.is_startup_complete() {
        model.state = AppState::Menu;
        model.needs_device_refresh = true;
    }

    // Auto-fetch device status once after transitioning to the Menu
    if model.needs_device_refresh && model.state == AppState::Menu {
        model.needs_device_refresh = false;
        let status = model.adb_manager.fetch_device_status();
        model.device_status = status;
    }
}

/// Execute a command dispatched from the menu (ADB or fastboot).
async fn execute_adb_command(model: &mut Model, command: MenuCommand) -> CommandResult {
    // Dispatch fastboot commands to FastbootManager without ADB
    let adb_command: AdbCommand = match command {
        MenuCommand::Fastboot(cmd) => {
            return match FastbootManager::new().execute(cmd) {
                Ok(output) => CommandResult::Success(output),
                Err(e) => CommandResult::Error(format!("{}", e)),
            };
        }
        MenuCommand::Adb(cmd) => cmd,
        MenuCommand::OpenLogcat => {
            // Should never reach here — intercepted earlier in ExecuteCommand handler
            return CommandResult::Error("Logcat should be opened via OpenLogcat message".into());
        }
        MenuCommand::OpenDevMode => {
            return CommandResult::Error(
                "Dev Mode should be opened via OpenDevMode message".into(),
            );
        }
        MenuCommand::OpenRomFlash => {
            return CommandResult::Error(
                "ROM Flasher should be opened via OpenRomFlash message".into(),
            );
        }
    };

    // Execute the ADB command using the ADB manager
    match model.adb_manager.execute(adb_command) {
        Ok(output) => CommandResult::Success(output),
        Err(e) => {
            let error_msg = format!("{}", e);

            // Add helpful context for common errors
            let enhanced_error = if error_msg.contains("Connection")
                || error_msg.contains("connection")
            {
                format!(
                    "{}\n\nTroubleshooting:\n• Make sure ADB server is running (adb start-server)\n• Check that device is connected (adb devices)\n• Verify USB debugging is enabled on device",
                    error_msg
                )
            } else if error_msg.contains("No device selected") {
                format!(
                    "{}\n\nPlease:\n• Connect an Android device via USB\n• Enable USB debugging on the device\n• Run 'List Devices' to detect your device",
                    error_msg
                )
            } else {
                error_msg
            };

            CommandResult::Error(enhanced_error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_menu_navigation() {
        let mut model = Model::new();
        model.state = AppState::Menu;

        let initial = model.menu.selected;
        update(&mut model, Message::MenuDown).await;
        assert!(model.menu.selected > initial);
        assert!(model.menu.entries[model.menu.selected].is_selectable());

        update(&mut model, Message::MenuUp).await;
        assert_eq!(model.menu.selected, initial);
    }

    #[tokio::test]
    async fn test_quit() {
        let mut model = Model::new();
        assert!(model.running);

        update(&mut model, Message::Quit).await;
        assert!(!model.running);
    }

    #[tokio::test]
    async fn test_scroll_boundaries() {
        let mut model = Model::new();
        model.wrapped_lines = vec!["Line 1".to_string(), "Line 2".to_string()];
        model.scroll_position = 0;

        // Can't scroll up from position 0
        update(&mut model, Message::ScrollUp).await;
        assert_eq!(model.scroll_position, 0);

        // Can scroll down
        update(&mut model, Message::ScrollDown).await;
        assert_eq!(model.scroll_position, 1);

        // Can't scroll past end
        update(&mut model, Message::ScrollDown).await;
        assert_eq!(model.scroll_position, 1);
    }

    #[tokio::test]
    async fn test_enter_exit_child_mode() {
        let mut model = Model::new();
        model.state = AppState::Menu;
        // EnterChild/ExitChild are no-ops in the flat menu — verify no panic
        // and that selection still points at a valid item afterwards.
        update(&mut model, Message::EnterChild).await;
        update(&mut model, Message::ExitChild).await;
        assert!(model.menu.entries[model.menu.selected].is_selectable());
    }

    #[tokio::test]
    async fn test_clear_results() {
        let mut model = Model::new();
        model.set_result("Test output".to_string());
        assert!(model.command_result.is_some());
        assert!(!model.result_lines.is_empty());

        update(&mut model, Message::ReturnToMenu).await;
        assert!(model.command_result.is_none());
        assert!(model.result_lines.is_empty());
        assert_eq!(model.state, AppState::Menu);
    }
}
