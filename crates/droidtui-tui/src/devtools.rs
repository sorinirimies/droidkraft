//! Dev Tools — build, run, and edit Android projects from the terminal.
//!
//! Provides Gradle build integration, APK install + launch, editor picker,
//! and a file browser — all without an IDE.

use std::collections::VecDeque;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc;

// ── Editor catalogue ──────────────────────────────────────────────────────────

/// A terminal editor that can be launched to edit files.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Editor {
    #[default]
    None,
    Helix,
    Neovim,
    Vim,
    Nano,
    Micro,
    Emacs,
}

impl Editor {
    pub fn label(&self) -> &'static str {
        match self {
            Editor::None => "none",
            Editor::Helix => "helix",
            Editor::Neovim => "nvim",
            Editor::Vim => "vim",
            Editor::Nano => "nano",
            Editor::Micro => "micro",
            Editor::Emacs => "emacs",
        }
    }

    pub fn binary(&self) -> Option<&'static str> {
        match self {
            Editor::None => None,
            Editor::Helix => Some("hx"),
            Editor::Neovim => Some("nvim"),
            Editor::Vim => Some("vim"),
            Editor::Nano => Some("nano"),
            Editor::Micro => Some("micro"),
            Editor::Emacs => Some("emacs"),
        }
    }

    pub fn all() -> &'static [Editor] {
        &[
            Editor::None,
            Editor::Helix,
            Editor::Neovim,
            Editor::Vim,
            Editor::Nano,
            Editor::Micro,
            Editor::Emacs,
        ]
    }
}

// ── Build variant ─────────────────────────────────────────────────────────────

/// Android build variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildVariant {
    pub name: String,
    pub task: String,
}

impl BuildVariant {
    pub fn common() -> Vec<Self> {
        vec![
            Self {
                name: "Debug".into(),
                task: "assembleDebug".into(),
            },
            Self {
                name: "Release".into(),
                task: "assembleRelease".into(),
            },
            Self {
                name: "Debug (Install)".into(),
                task: "installDebug".into(),
            },
            Self {
                name: "Release (Install)".into(),
                task: "installRelease".into(),
            },
            Self {
                name: "Clean".into(),
                task: "clean".into(),
            },
            Self {
                name: "Clean + Debug".into(),
                task: "clean assembleDebug".into(),
            },
            Self {
                name: "Lint".into(),
                task: "lint".into(),
            },
            Self {
                name: "Test".into(),
                task: "test".into(),
            },
            Self {
                name: "Connected Test".into(),
                task: "connectedAndroidTest".into(),
            },
        ]
    }
}

// ── Build output ──────────────────────────────────────────────────────────────

/// Status of the current build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildStatus {
    Idle,
    Building,
    Success,
    Failed,
}

// ── Dev Tools state ───────────────────────────────────────────────────────────

/// Full state for the Dev Tools mode.
#[derive(Debug)]
pub struct DevToolsState {
    // ── Project ───────────────────────────────────────────────────────────
    /// Root directory of the Android project (contains `gradlew`).
    pub project_dir: PathBuf,
    /// Whether a `gradlew` / `gradlew.bat` was detected.
    pub has_gradle: bool,

    // ── Editor ────────────────────────────────────────────────────────────
    /// Currently selected terminal editor.
    pub editor: Editor,
    /// Whether the editor picker panel is open.
    pub editor_picker_open: bool,
    /// Cursor position in the editor picker.
    pub editor_picker_cursor: usize,

    // ── Build ─────────────────────────────────────────────────────────────
    /// Available build variants.
    pub variants: Vec<BuildVariant>,
    /// Currently selected variant index.
    pub selected_variant: usize,
    /// Current build status.
    pub build_status: BuildStatus,
    /// Build output lines (ring buffer).
    pub build_output: VecDeque<String>,
    /// Channel receiver for build output from the background thread.
    build_receiver: Option<mpsc::Receiver<String>>,
    /// Whether variant picker is open.
    pub variant_picker_open: bool,
    /// Discovered Android application modules.
    pub app_modules: Vec<AppModule>,
    /// Index of the currently targeted app module (for Build & Install).
    pub selected_app_module: usize,

    // ── File browser ──────────────────────────────────────────────────────
    /// File explorer state for browsing project files.
    pub file_explorer: Option<tui_file_explorer::FileExplorer>,

    // ── Focus ─────────────────────────────────────────────────────────────
    /// Which panel currently has keyboard focus.
    pub focus: DevFocus,

    // ── Status ────────────────────────────────────────────────────────────
    /// Status message shown at the bottom.
    pub status_message: Option<String>,
}

/// Which panel has focus in Dev Mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevFocus {
    /// File browser panel.
    FileBrowser,
    /// Build output / logcat panel.
    BuildOutput,
    /// Toolbar (variant picker).
    Toolbar,
}

/// Discovered Android application module inside the Gradle project.
#[derive(Debug, Clone)]
pub struct AppModule {
    /// Gradle module path, e.g. `:app` or `:apps:mobile`.
    pub gradle_path: String,
    /// Short display name, e.g. `app` or `mobile`.
    pub name: String,
    /// File-system path relative to the project root, e.g. `app` or `apps/mobile`.
    pub fs_path: String,
}

impl DevToolsState {
    pub fn new() -> Self {
        let start_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let (gradle_root, _) = find_gradle_root(&start_dir);
        let project_dir = gradle_root.unwrap_or_else(|| start_dir.clone());
        let has_gradle = project_dir.join("gradlew").exists()
            || project_dir.join("gradlew.bat").exists()
            || std::process::Command::new("which")
                .arg("gradle")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            || PathBuf::from(std::env::var("HOME").unwrap_or_default())
                .join(".sdkman/candidates/gradle/current/bin/gradle")
                .exists()
            || PathBuf::from("/opt/homebrew/bin/gradle").exists()
            || PathBuf::from("/usr/local/bin/gradle").exists();

        let explorer = tui_file_explorer::FileExplorer::new(start_dir, vec![]);

        let mut state = Self {
            project_dir: project_dir.clone(),
            has_gradle,
            editor: Editor::default(),
            editor_picker_open: false,
            editor_picker_cursor: 0,
            variants: BuildVariant::common(),
            selected_variant: 0,
            build_status: BuildStatus::Idle,
            build_output: VecDeque::with_capacity(5000),
            build_receiver: None,
            variant_picker_open: false,
            app_modules: Vec::new(),
            selected_app_module: 0,
            file_explorer: Some(explorer),
            focus: DevFocus::FileBrowser,
            status_message: None,
        };

        state.discover_app_modules();
        state.rebuild_variants();
        state
    }

    /// Set the project directory and re-detect Gradle.
    pub fn set_project_dir(&mut self, dir: PathBuf) {
        let (gradle_root, _) = find_gradle_root(&dir);
        self.project_dir = gradle_root.unwrap_or_else(|| dir.clone());
        self.has_gradle = self.resolve_gradle().is_some();
        self.file_explorer = Some(tui_file_explorer::FileExplorer::new(dir, vec![]));
        self.discover_app_modules();
        self.rebuild_variants();
    }

    /// Called after the file browser navigates to a new directory.
    /// If the browser is now inside a Gradle project that differs from the
    /// current `project_dir`, update the project root and re-discover modules.
    pub fn sync_project_from_browser(&mut self, browser_dir: &std::path::Path) {
        let (gradle_root, _) = find_gradle_root(browser_dir);
        if let Some(root) = gradle_root {
            if root != self.project_dir {
                self.project_dir = root;
                self.has_gradle = true;
                self.discover_app_modules();
                self.rebuild_variants();
                let module_count = self.app_modules.len();
                if module_count > 0 {
                    let names: Vec<&str> =
                        self.app_modules.iter().map(|m| m.name.as_str()).collect();
                    self.status_message = Some(format!(
                        "📱 Found {} app module{}: {}",
                        module_count,
                        if module_count == 1 { "" } else { "s" },
                        names.join(", ")
                    ));
                }
            }
        }
    }

    /// Walk the project tree and find all modules that apply the
    /// `com.android.application` plugin (i.e. modules that produce an APK).
    pub fn discover_app_modules(&mut self) {
        self.app_modules.clear();
        self.selected_app_module = 0;
        discover_app_modules_recursive(&self.project_dir, &self.project_dir, &mut self.app_modules);
        // Sort by gradle path for deterministic order
        self.app_modules
            .sort_by(|a, b| a.gradle_path.cmp(&b.gradle_path));
    }

    /// Regenerate the build-variant list based on discovered app modules.
    ///
    /// When app modules are found the variants become module-specific (e.g.
    /// `:app:assembleDebug`); otherwise fall back to generic tasks.
    pub fn rebuild_variants(&mut self) {
        if self.app_modules.is_empty() {
            self.variants = BuildVariant::common();
        } else if self.app_modules.len() == 1 {
            let m = &self.app_modules[0];
            self.variants = vec![
                BuildVariant {
                    name: format!("{}: Debug", m.name),
                    task: format!("{}:assembleDebug", m.gradle_path),
                },
                BuildVariant {
                    name: format!("{}: Release", m.name),
                    task: format!("{}:assembleRelease", m.gradle_path),
                },
                BuildVariant {
                    name: format!("{}: Install Debug", m.name),
                    task: format!("{}:installDebug", m.gradle_path),
                },
                BuildVariant {
                    name: format!("{}: Install Release", m.name),
                    task: format!("{}:installRelease", m.gradle_path),
                },
                BuildVariant {
                    name: "Clean".into(),
                    task: "clean".into(),
                },
                BuildVariant {
                    name: format!("Clean + {} Debug", m.name),
                    task: format!("clean {}:assembleDebug", m.gradle_path),
                },
                BuildVariant {
                    name: format!("{}: Lint", m.name),
                    task: format!("{}:lint", m.gradle_path),
                },
                BuildVariant {
                    name: format!("{}: Test", m.name),
                    task: format!("{}:test", m.gradle_path),
                },
            ];
        } else {
            let mut variants = Vec::new();
            for m in &self.app_modules {
                variants.push(BuildVariant {
                    name: format!("{}: Debug", m.name),
                    task: format!("{}:assembleDebug", m.gradle_path),
                });
                variants.push(BuildVariant {
                    name: format!("{}: Install Debug", m.name),
                    task: format!("{}:installDebug", m.gradle_path),
                });
                variants.push(BuildVariant {
                    name: format!("{}: Release", m.name),
                    task: format!("{}:assembleRelease", m.gradle_path),
                });
            }
            variants.push(BuildVariant {
                name: "Clean".into(),
                task: "clean".into(),
            });
            self.variants = variants;
        }
        self.selected_variant = 0;
    }

    /// Resolve the best available Gradle executable using multiple strategies:
    ///
    /// 1. `gradlew` / `gradlew.bat` wrapper found by walking up from `project_dir`
    /// 2. System-wide `gradle` on PATH (`which gradle`)
    /// 3. SDKMAN managed install (`~/.sdkman/candidates/gradle/current/bin/gradle`)
    /// 4. Homebrew install (`/opt/homebrew/bin/gradle` or `/usr/local/bin/gradle`)
    ///
    /// Returns `(executable_path_or_name, is_wrapper, label)`.
    fn resolve_gradle(&self) -> Option<GradleExecutable> {
        // Strategy 1: local gradlew wrapper (preferred — uses project-pinned version)
        if let Some(root) = find_gradle_root(&self.project_dir).0 {
            let wrapper = if cfg!(windows) {
                root.join("gradlew.bat")
            } else {
                root.join("gradlew")
            };
            if wrapper.exists() {
                return Some(GradleExecutable {
                    path: wrapper,
                    _is_wrapper: true,
                    label: "gradlew".into(),
                });
            }
        }

        // Strategy 2: system-wide `gradle` on PATH
        let which_result = std::process::Command::new("which")
            .arg("gradle")
            .output()
            .ok()
            .filter(|o| o.status.success());

        if let Some(output) = which_result {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path_str.is_empty() {
                let path = PathBuf::from(&path_str);
                if path.exists() {
                    return Some(GradleExecutable {
                        path,
                        _is_wrapper: false,
                        label: "gradle (system)".into(),
                    });
                }
            }
        }

        // Strategy 3: SDKMAN
        if let Ok(home) = std::env::var("HOME") {
            let sdkman = PathBuf::from(&home).join(".sdkman/candidates/gradle/current/bin/gradle");
            if sdkman.exists() {
                return Some(GradleExecutable {
                    path: sdkman,
                    _is_wrapper: false,
                    label: "gradle (sdkman)".into(),
                });
            }
        }

        // Strategy 4: Homebrew (Apple Silicon and Intel paths)
        for brew_path in &["/opt/homebrew/bin/gradle", "/usr/local/bin/gradle"] {
            let path = PathBuf::from(brew_path);
            if path.exists() {
                return Some(GradleExecutable {
                    path,
                    _is_wrapper: false,
                    label: "gradle (homebrew)".into(),
                });
            }
        }

        None
    }

    /// Start a build in a background thread using the currently selected variant.
    pub fn start_build(&mut self) {
        let variant = self.variants[self.selected_variant].clone();
        self.start_build_variant(&variant);
    }

    /// Build and install the debug APK for the currently selected app module.
    /// Uses Gradle's `installDebug` task which handles both building AND pushing.
    pub fn build_and_install(&mut self) {
        if self.app_modules.is_empty() {
            self.status_message = Some(
                "No app modules found — navigate the file browser into an Android project first."
                    .into(),
            );
            return;
        }
        let module = &self.app_modules[self.selected_app_module];
        let variant = BuildVariant {
            name: format!("{}: Build & Install Debug", module.name),
            task: format!("{}:installDebug", module.gradle_path),
        };
        self.start_build_variant(&variant);
    }

    /// Internal: run a Gradle build with the given variant/task.
    fn start_build_variant(&mut self, variant: &BuildVariant) {
        if self.build_status == BuildStatus::Building {
            return; // already building
        }

        let gradle = match self.resolve_gradle() {
            Some(g) => g,
            None => {
                self.status_message = Some(
                    "Gradle not found. Install gradle or add a gradlew wrapper to your project."
                        .into(),
                );
                return;
            }
        };

        let tasks: Vec<String> = variant
            .task
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        let project_dir = self.project_dir.clone();

        self.build_status = BuildStatus::Building;
        self.build_output.clear();
        self.build_output.push_back(format!(
            "▶ {} {} [{}]",
            gradle.label,
            variant.task,
            project_dir.display()
        ));
        self.build_output.push_back(String::new());

        let wrapper = gradle.path;

        let (tx, rx) = mpsc::channel::<String>();
        self.build_receiver = Some(rx);

        std::thread::spawn(move || {
            use std::io::BufRead;

            let mut cmd = Command::new(&wrapper);
            cmd.args(&tasks);
            cmd.current_dir(&project_dir);
            cmd.stdout(std::process::Stdio::piped());
            cmd.stderr(std::process::Stdio::piped());

            match cmd.spawn() {
                Ok(mut child) => {
                    // Read stdout
                    if let Some(stdout) = child.stdout.take() {
                        let reader = std::io::BufReader::new(stdout);
                        for line in reader.lines() {
                            match line {
                                Ok(l) => {
                                    if tx.send(l).is_err() {
                                        return;
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                    }
                    // Read stderr
                    if let Some(stderr) = child.stderr.take() {
                        let reader = std::io::BufReader::new(stderr);
                        for line in reader.lines() {
                            match line {
                                Ok(l) => {
                                    if tx.send(format!("ERR: {}", l)).is_err() {
                                        return;
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                    }

                    match child.wait() {
                        Ok(status) => {
                            if status.success() {
                                let _ = tx.send("── BUILD SUCCESSFUL ──".into());
                            } else {
                                let _ = tx.send(format!(
                                    "── BUILD FAILED (exit code: {}) ──",
                                    status.code().unwrap_or(-1)
                                ));
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(format!("── BUILD ERROR: {} ──", e));
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(format!("── FAILED TO START: {} ──", e));
                }
            }
        });
    }

    /// Poll build output from the background thread. Call on each tick.
    pub fn poll_build_output(&mut self) {
        let receiver = match &self.build_receiver {
            Some(rx) => rx,
            None => return,
        };

        for _ in 0..100 {
            match receiver.try_recv() {
                Ok(line) => {
                    // Detect build completion
                    if line.contains("BUILD SUCCESSFUL") {
                        self.build_status = BuildStatus::Success;
                        self.status_message = Some("✅ Build successful".into());
                    } else if line.contains("BUILD FAILED")
                        || line.contains("FAILED TO START")
                        || line.contains("BUILD ERROR")
                    {
                        self.build_status = BuildStatus::Failed;
                        self.status_message = Some("❌ Build failed".into());
                    }
                    self.build_output.push_back(line);
                    // Cap at 5000 lines
                    while self.build_output.len() > 5000 {
                        self.build_output.pop_front();
                    }
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    if self.build_status == BuildStatus::Building {
                        self.build_status = BuildStatus::Idle;
                    }
                    self.build_receiver = None;
                    break;
                }
            }
        }
    }

    /// Install the debug APK and launch the main activity.
    pub fn run_app(&mut self) -> Result<(), String> {
        // Determine which module to look for
        let module_fs_path = if !self.app_modules.is_empty() {
            self.app_modules[self.selected_app_module].fs_path.clone()
        } else {
            "app".to_string()
        };

        let apk_dir = self
            .project_dir
            .join(&module_fs_path)
            .join("build/outputs/apk/debug");
        let module_name = std::path::Path::new(&module_fs_path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let apk_path = if apk_dir.join(format!("{}-debug.apk", module_name)).exists() {
            apk_dir.join(format!("{}-debug.apk", module_name))
        } else if apk_dir.join("app-debug.apk").exists() {
            apk_dir.join("app-debug.apk")
        } else {
            // Try to find any APK
            std::fs::read_dir(&apk_dir)
                .ok()
                .and_then(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .find(|e| e.path().extension().map(|x| x == "apk").unwrap_or(false))
                        .map(|e| e.path())
                })
                .ok_or_else(|| format!("No APK found in {}", apk_dir.display()))?
        };

        self.status_message = Some(format!("Installing {}…", apk_path.display()));

        let output = Command::new("adb")
            .args(["install", "-r", &apk_path.display().to_string()])
            .output()
            .map_err(|e| format!("Failed to run adb install: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Install failed: {}", stderr));
        }

        self.status_message = Some("✅ APK installed. Launching…".into());

        let stdout = String::from_utf8_lossy(&output.stdout);
        self.build_output
            .push_back(format!("Install: {}", stdout.trim()));

        Ok(())
    }

    /// Toggle the editor picker panel.
    pub fn toggle_editor_picker(&mut self) {
        self.editor_picker_open = !self.editor_picker_open;
        if self.editor_picker_open {
            // Sync cursor to current editor
            self.editor_picker_cursor = Editor::all()
                .iter()
                .position(|e| e == &self.editor)
                .unwrap_or(0);
        }
    }

    /// Move editor picker cursor up.
    pub fn editor_picker_up(&mut self) {
        let len = Editor::all().len();
        self.editor_picker_cursor = if self.editor_picker_cursor == 0 {
            len - 1
        } else {
            self.editor_picker_cursor - 1
        };
    }

    /// Move editor picker cursor down.
    pub fn editor_picker_down(&mut self) {
        let len = Editor::all().len();
        self.editor_picker_cursor = (self.editor_picker_cursor + 1) % len;
    }

    /// Confirm editor picker selection.
    pub fn editor_picker_confirm(&mut self) {
        self.editor = Editor::all()[self.editor_picker_cursor].clone();
        self.editor_picker_open = false;
        self.status_message = Some(format!("Editor set to \"{}\"", self.editor.label()));
    }

    /// Toggle variant picker.
    pub fn toggle_variant_picker(&mut self) {
        self.variant_picker_open = !self.variant_picker_open;
    }

    /// Cycle to next build variant.
    pub fn next_variant(&mut self) {
        self.selected_variant = (self.selected_variant + 1) % self.variants.len();
    }

    /// Cycle to previous build variant.
    pub fn prev_variant(&mut self) {
        let len = self.variants.len();
        self.selected_variant = if self.selected_variant == 0 {
            len - 1
        } else {
            self.selected_variant - 1
        };
    }

    /// Cycle focus between panels.
    pub fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            DevFocus::FileBrowser => DevFocus::BuildOutput,
            DevFocus::BuildOutput => DevFocus::Toolbar,
            DevFocus::Toolbar => DevFocus::FileBrowser,
        };
    }

    /// Get the currently selected variant.
    pub fn current_variant(&self) -> &BuildVariant {
        &self.variants[self.selected_variant]
    }
}

impl Default for DevToolsState {
    fn default() -> Self {
        Self::new()
    }
}

/// Describes a resolved Gradle executable.
struct GradleExecutable {
    /// Absolute path to the executable.
    path: PathBuf,
    /// True when this is a `gradlew` project wrapper (vs a system install).
    _is_wrapper: bool,
    /// Human-readable label shown in the build output header.
    label: String,
}

/// Walk up the directory tree from `start` looking for `gradlew` or
/// `gradlew.bat`.  Returns `(Some(root_dir), true)` on success or
/// `(None, false)` if neither is found all the way up to `/`.
fn find_gradle_root(start: &std::path::Path) -> (Option<PathBuf>, bool) {
    let mut dir = start.to_path_buf();
    loop {
        if dir.join("gradlew").exists() || dir.join("gradlew.bat").exists() {
            return (Some(dir), true);
        }
        if !dir.pop() {
            return (None, false);
        }
    }
}

/// Recursively walk `dir` looking for `build.gradle.kts` / `build.gradle`
/// files that contain the `com.android.application` plugin declaration.
fn discover_app_modules_recursive(
    root: &std::path::Path,
    dir: &std::path::Path,
    modules: &mut Vec<AppModule>,
) {
    // Don't scan the project root's own build file — it's the umbrella.
    if dir != root {
        for build_file in &["build.gradle.kts", "build.gradle"] {
            let path = dir.join(build_file);
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if content.contains("com.android.application") {
                        let rel = dir.strip_prefix(root).unwrap_or(dir);
                        let gradle_path = format!(":{}", rel.to_string_lossy().replace('/', ":"));
                        let name = dir
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        modules.push(AppModule {
                            gradle_path,
                            name,
                            fs_path: rel.to_string_lossy().to_string(),
                        });
                    }
                }
                break; // found a build file, don't check both
            }
        }
    }

    // Recurse into subdirectories, skipping well-known non-module dirs
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if !name.starts_with('.')
                    && name != "build"
                    && name != "buildSrc"
                    && name != "gradle"
                    && name != "node_modules"
                {
                    discover_app_modules_recursive(root, &path, modules);
                }
            }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_all() {
        assert_eq!(Editor::all().len(), 7);
    }

    #[test]
    fn test_editor_labels() {
        assert_eq!(Editor::Helix.label(), "helix");
        assert_eq!(Editor::Neovim.label(), "nvim");
        assert_eq!(Editor::None.label(), "none");
    }

    #[test]
    fn test_editor_binaries() {
        assert_eq!(Editor::Helix.binary(), Some("hx"));
        assert_eq!(Editor::Neovim.binary(), Some("nvim"));
        assert_eq!(Editor::None.binary(), None);
    }

    #[test]
    fn test_build_variants() {
        let variants = BuildVariant::common();
        assert!(variants.len() >= 5);
        assert_eq!(variants[0].name, "Debug");
        assert_eq!(variants[0].task, "assembleDebug");
    }

    #[test]
    fn test_devtools_state_new() {
        let state = DevToolsState::new();
        assert_eq!(state.build_status, BuildStatus::Idle);
        assert_eq!(state.editor, Editor::None);
        assert!(!state.editor_picker_open);
        assert!(state.file_explorer.is_some());
    }

    #[test]
    fn test_find_gradle_root_walks_up() {
        // Create a temp dir tree: root/sub/deep
        let tmp = std::env::temp_dir().join("droidtui_test_gradle");
        let sub = tmp.join("sub").join("deep");
        let _ = std::fs::create_dir_all(&sub);
        // Place a fake gradlew at root
        let _ = std::fs::write(tmp.join("gradlew"), "#!/bin/sh\n");

        let (found, has) = find_gradle_root(&sub);
        assert!(has, "should find gradlew by walking up");
        assert_eq!(found.unwrap(), tmp);

        // Cleanup
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_find_gradle_root_not_found() {
        // A path that definitely doesn't have gradlew above it
        let (found, has) = find_gradle_root(std::path::Path::new("/tmp/nonexistent_droidtui_xyz"));
        assert!(!has);
        assert!(found.is_none());
    }

    #[test]
    fn test_editor_picker_nav() {
        let mut state = DevToolsState::new();
        state.toggle_editor_picker();
        assert!(state.editor_picker_open);

        state.editor_picker_down();
        assert_eq!(state.editor_picker_cursor, 1);

        state.editor_picker_up();
        assert_eq!(state.editor_picker_cursor, 0);

        // Wrap around
        state.editor_picker_up();
        assert_eq!(state.editor_picker_cursor, Editor::all().len() - 1);
    }

    #[test]
    fn test_editor_picker_confirm() {
        let mut state = DevToolsState::new();
        state.toggle_editor_picker();
        state.editor_picker_cursor = 1; // Helix
        state.editor_picker_confirm();
        assert_eq!(state.editor, Editor::Helix);
        assert!(!state.editor_picker_open);
    }

    #[test]
    fn test_variant_cycling() {
        let mut state = DevToolsState::new();
        let initial = state.selected_variant;
        state.next_variant();
        assert_ne!(state.selected_variant, initial);
        state.prev_variant();
        assert_eq!(state.selected_variant, initial);
    }

    #[test]
    fn test_focus_cycling() {
        let mut state = DevToolsState::new();
        assert_eq!(state.focus, DevFocus::FileBrowser);
        state.cycle_focus();
        assert_eq!(state.focus, DevFocus::BuildOutput);
        state.cycle_focus();
        assert_eq!(state.focus, DevFocus::Toolbar);
        state.cycle_focus();
        assert_eq!(state.focus, DevFocus::FileBrowser);
    }

    #[test]
    fn test_build_status_default() {
        assert_eq!(BuildStatus::Idle, BuildStatus::Idle);
        assert_ne!(BuildStatus::Idle, BuildStatus::Building);
    }

    #[test]
    fn test_discover_app_modules_with_app_plugin() {
        let tmp = std::env::temp_dir().join("droidtui_test_app_modules");
        let app_dir = tmp.join("app");
        let lib_dir = tmp.join("lib");
        let _ = std::fs::create_dir_all(&app_dir);
        let _ = std::fs::create_dir_all(&lib_dir);
        let _ = std::fs::write(tmp.join("gradlew"), "#!/bin/sh\n");
        let _ = std::fs::write(
            app_dir.join("build.gradle.kts"),
            "plugins {\n    id(\"com.android.application\")\n}\n",
        );
        let _ = std::fs::write(
            lib_dir.join("build.gradle.kts"),
            "plugins {\n    id(\"com.android.library\")\n}\n",
        );

        let mut modules = Vec::new();
        discover_app_modules_recursive(&tmp, &tmp, &mut modules);

        assert_eq!(modules.len(), 1, "should find exactly 1 app module");
        assert_eq!(modules[0].name, "app");
        assert_eq!(modules[0].gradle_path, ":app");
        assert_eq!(modules[0].fs_path, "app");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_rebuild_variants_single_module() {
        let mut state = DevToolsState::new();
        state.app_modules = vec![AppModule {
            gradle_path: ":app".into(),
            name: "app".into(),
            fs_path: "app".into(),
        }];
        state.rebuild_variants();
        assert!(state.variants[0].task.contains(":app:"));
        assert!(state
            .variants
            .iter()
            .any(|v| v.task.contains("assembleDebug")));
        assert!(state
            .variants
            .iter()
            .any(|v| v.task.contains("installDebug")));
    }

    #[test]
    fn test_rebuild_variants_no_modules() {
        let mut state = DevToolsState::new();
        state.app_modules.clear();
        state.rebuild_variants();
        assert_eq!(state.variants[0].name, "Debug");
        assert_eq!(state.variants[0].task, "assembleDebug");
    }
}
