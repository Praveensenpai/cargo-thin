# CODEBASE.md: cargo-thin Semantic Digest

> **Notice**: AI-optimized semantic index. Do not write narrative prose. Keep token density high.

## 1. System Topology & Data Flow
```text
Entrypoint ──> CLI/Parser ──> Domain Logic ──> Infra/IO
```

## 2. Global Constraints & Architecture Patterns
- **Primary Language**: Rust 2021 edition
- **Architectural Paradigm**: Role-based (domain/, infra/, api/cli/, tui/)
- **Hard Constraints**: <400 lines/file, <60 lines/fn, zero production unwrap(), 0 warnings.
- **Target Distribution**: Linux x86_64 standalone binary

## 3. Module & Interface Skeleton

### `src/cli.rs` (Role: cli, Lines: 137)
- **Responsibility**: Core cli logic in src/cli.rs
- **Imports**: use clap :: Parser , use std :: io , use std :: path :: { Path , PathBuf } , use crate :: domain :: cleaner :: clean_project , use crate :: domain :: project :: { format_bytes , ProjectInfo } , use crate :: domain :: scanner :: scan_projects 
- **Types & Enums**:
  ```rust
  pub struct CliArgs
  ```
- **Public Functions & Signatures**:
  ```rust
  fn run_headless (args : & CliArgs) -> io :: Result < () >
  ```

### `src/domain/cleaner.rs` (Role: domain, Lines: 207)
- **Responsibility**: Core domain logic in src/domain/cleaner.rs
- **Imports**: use serde :: { Deserialize , Serialize } , use std :: fs , use std :: io , use std :: os :: unix :: fs :: PermissionsExt , use std :: path :: { Path , PathBuf } , use thiserror :: Error , use crate :: domain :: elf :: is_valid_elf_executable , use crate :: domain :: project :: { BinaryArtifact , ProfileKind , ProjectInfo } 
- **Types & Enums**:
  ```rust
  pub enum CleanError
  pub struct CleanReport
  ```
- **Public Functions & Signatures**:
  ```rust
  fn clean_project (project : & ProjectInfo , keep_both : bool) -> Result < CleanReport , CleanError >
  ```

### `src/domain/elf.rs` (Role: domain, Lines: 81)
- **Responsibility**: Core domain logic in src/domain/elf.rs
- **Imports**: use std :: fs :: File , use std :: io :: Read , use std :: os :: unix :: fs :: PermissionsExt , use std :: path :: Path 
- **Public Functions & Signatures**:
  ```rust
  fn is_valid_elf_executable (path : & Path) -> bool
  ```

### `src/domain/project.rs` (Role: domain, Lines: 127)
- **Responsibility**: Core domain logic in src/domain/project.rs
- **Imports**: use serde :: { Deserialize , Serialize } , use std :: path :: PathBuf 
- **Types & Enums**:
  ```rust
  pub enum ProfileKind
  pub struct BinaryArtifact
  pub struct TargetBreakdown
  pub struct ProjectInfo
  pub enum SortOrder
  ```
- **Public Functions & Signatures**:
  ```rust
  fn preserved_binaries_bytes (& self , keep_both : bool) -> u64
  fn reclaimable_bytes (& self , keep_both : bool) -> u64
  fn reclaimable_percent (& self , keep_both : bool) -> f64
  fn next (self) -> Self
  fn label (self) -> & 'static str
  fn format_bytes (bytes : u64) -> String
  ```

### `src/domain/scanner.rs` (Role: domain, Lines: 144)
- **Responsibility**: Core domain logic in src/domain/scanner.rs
- **Imports**: use std :: fs , use std :: path :: Path , use toml :: Value , use walkdir :: WalkDir , use crate :: domain :: elf :: is_valid_elf_executable , use crate :: domain :: project :: { BinaryArtifact , ProfileKind , ProjectInfo , TargetBreakdown } 
- **Public Functions & Signatures**:
  ```rust
  fn scan_projects (root : & Path) -> Vec < ProjectInfo >
  ```

### `src/domain.rs` (Role: domain, Lines: 6)
- **Responsibility**: Core domain logic in src/domain.rs
- **Imports**: pub use scanner :: scan_projects 

### `src/lib.rs` (Role: general, Lines: 3)
- **Responsibility**: Core general logic in src/lib.rs

### `src/main.rs` (Role: general, Lines: 92)
- **Responsibility**: Core general logic in src/main.rs
- **Imports**: use clap :: Parser , use crossterm :: event :: { self , Event } , use crossterm :: execute , use crossterm :: terminal :: { disable_raw_mode , enable_raw_mode , EnterAlternateScreen , LeaveAlternateScreen , } , use ratatui :: backend :: CrosstermBackend , use ratatui :: Terminal , use std :: io :: { self , stdout , IsTerminal } , use std :: time :: Duration , use cargo_thin :: cli :: { run_headless , CliArgs } , use cargo_thin :: domain :: scan_projects , use cargo_thin :: ui :: app :: AppMode , use cargo_thin :: ui :: { handle_event , render_app , App } 

### `src/ui/app.rs` (Role: tui, Lines: 256)
- **Responsibility**: Core tui logic in src/ui/app.rs
- **Imports**: use std :: sync :: mpsc , use crate :: domain :: cleaner :: { clean_project , CleanReport } , use crate :: domain :: project :: { ProjectInfo , SortOrder } 
- **Types & Enums**:
  ```rust
  pub enum CleanEvent
  pub enum AppMode
  pub struct App
  ```
- **Public Functions & Signatures**:
  ```rust
  fn new (projects : Vec < ProjectInfo > , keep_both : bool) -> Self
  fn selected_project (& self) -> Option < & ProjectInfo >
  fn move_up (& mut self)
  fn move_down (& mut self)
  fn toggle_selection (& mut self)
  fn toggle_all (& mut self)
  fn cycle_sort (& mut self)
  fn toggle_keep_both (& mut self)
  fn total_target_bytes (& self) -> u64
  fn total_reclaimable_bytes (& self) -> u64
  fn selected_count (& self) -> usize
  fn selected_reclaimable_bytes (& self) -> u64
  fn start_clean (& mut self)
  fn poll_clean (& mut self)
  ```

### `src/ui/events.rs` (Role: tui, Lines: 70)
- **Responsibility**: Core tui logic in src/ui/events.rs
- **Imports**: use crossterm :: event :: { KeyCode , KeyEvent , KeyModifiers } , use crate :: ui :: app :: { App , AppMode } 
- **Public Functions & Signatures**:
  ```rust
  fn handle_event (app : & mut App , key : KeyEvent)
  ```

### `src/ui/modals.rs` (Role: tui, Lines: 238)
- **Responsibility**: Core tui logic in src/ui/modals.rs
- **Imports**: use ratatui :: layout :: { Alignment , Constraint , Direction , Layout , Rect } , use ratatui :: text :: { Line , Span } , use ratatui :: widgets :: { Block , Borders , Clear , Paragraph } , use ratatui :: Frame , use crate :: domain :: cleaner :: CleanReport , use crate :: domain :: project :: format_bytes , use crate :: ui :: app :: App , use crate :: ui :: theme :: Theme 
- **Public Functions & Signatures**:
  ```rust
  fn render_confirm_modal (frame : & mut Frame , app : & App)
  fn render_summary_modal (frame : & mut Frame , reports : & [CleanReport])
  fn render_help_modal (frame : & mut Frame)
  fn render_cleaning_modal (frame : & mut Frame , current : usize , total : usize , current_name : & str , tick : u8 ,)
  ```

### `src/ui/render.rs` (Role: tui, Lines: 273)
- **Responsibility**: Core tui logic in src/ui/render.rs
- **Imports**: use ratatui :: layout :: { Constraint , Direction , Layout , Rect } , use ratatui :: text :: { Line , Span } , use ratatui :: widgets :: { Block , Borders , Paragraph , Row , Table , TableState } , use ratatui :: Frame , use crate :: domain :: project :: { format_bytes , ProjectInfo } , use crate :: ui :: app :: { App , AppMode } , use crate :: ui :: modals :: { render_cleaning_modal , render_confirm_modal , render_help_modal , render_summary_modal , } , use crate :: ui :: theme :: { standard_block , Theme } 
- **Public Functions & Signatures**:
  ```rust
  fn render_app (frame : & mut Frame , app : & App)
  ```

### `src/ui/theme.rs` (Role: tui, Lines: 67)
- **Responsibility**: Core tui logic in src/ui/theme.rs
- **Imports**: use ratatui :: style :: { Color , Modifier , Style } , use ratatui :: widgets :: { Block , Borders , Padding } 
- **Types & Enums**:
  ```rust
  pub struct Theme
  ```
- **Public Functions & Signatures**:
  ```rust
  fn standard_block < 'a > (title : & 'a str , focused : bool) -> Block < 'a >
  ```

### `src/ui.rs` (Role: tui, Lines: 9)
- **Responsibility**: Core tui logic in src/ui.rs
- **Imports**: pub use app :: { App , AppMode } , pub use events :: handle_event , pub use render :: render_app 

## 4. Execution Lifecycle Trace
1. **Startup**: Entrypoint parses CLI flags & dispatches command.
2. **Execution**: Core domain logic processes inputs and evaluates rules.
3. **Persistence / I/O**: Domain logic calls infra for disk/terminal I/O.
4. **Exit**: Graceful termination with standard exit codes.

## 5. Verification Commands
```bash
cargo build --release --target x86_64-unknown-linux-gnu
cargo test --all-targets
cargo clippy --all-targets -- -D warnings && cargo fmt --check
```
