use std::fmt;

/// Represents a parsed command with its name and arguments
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Help,
    Rename(String),
    Move(String),
    Tag(Vec<String>),
    Delete,
    Export(String),
    Quit,
    Unknown(String),
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Help => write!(f, "help"),
            Command::Rename(title) => write!(f, "rename {}", title),
            Command::Move(folder) => write!(f, "move {}", folder),
            Command::Tag(tags) => write!(f, "tag {}", tags.join(" ")),
            Command::Delete => write!(f, "delete"),
            Command::Export(format) => write!(f, "export {}", format),
            Command::Quit => write!(f, "quit"),
            Command::Unknown(cmd) => write!(f, "unknown: {}", cmd),
        }
    }
}

/// Parses command strings into Command enum
pub struct CommandParser;

impl CommandParser {
    /// Parse a command string (without leading colon)
    pub fn parse(input: &str) -> Command {
        let input = input.trim();

        if input.is_empty() {
            return Command::Unknown(String::new());
        }

        // Split on first space to get command name and args
        let (cmd_name, args) = if let Some(space_idx) = input.find(' ') {
            (&input[..space_idx], &input[space_idx + 1..])
        } else {
            (input, "")
        };

        match cmd_name.to_lowercase().as_str() {
            "help" | "h" | "?" => Command::Help,
            "quit" | "q" | "exit" | "x" => Command::Quit,
            "delete" | "del" | "rm" => Command::Delete,
            "rename" | "rn" => {
                let title = args.trim();
                if title.is_empty() {
                    Command::Unknown("rename: missing title argument".to_string())
                } else {
                    Command::Rename(title.to_string())
                }
            }
            "move" | "mv" => {
                let folder = args.trim();
                if folder.is_empty() {
                    Command::Unknown("move: missing folder argument".to_string())
                } else {
                    Command::Move(folder.to_lowercase())
                }
            }
            "tag" => {
                let tags: Vec<String> = args.split_whitespace().map(|t| t.to_string()).collect();
                if tags.is_empty() {
                    Command::Unknown("tag: missing tag arguments".to_string())
                } else {
                    Command::Tag(tags)
                }
            }
            "export" | "exp" => {
                let format = args.trim();
                if format.is_empty() {
                    Command::Unknown("export: missing format argument".to_string())
                } else {
                    Command::Export(format.to_lowercase())
                }
            }
            _ => Command::Unknown(input.to_string()),
        }
    }

    /// Validate if a folder name is valid for `:move` command
    pub fn is_valid_folder(folder: &str) -> bool {
        matches!(
            folder.to_lowercase().as_str(),
            "daily" | "fleeting" | "permanent" | "literature" | "index" | "reference"
        )
    }
}

/// Represents the result of a command execution
#[derive(Debug, Clone, PartialEq)]
pub enum CommandResult {
    Success(String),
    Error(String),
}

/// Executes parsed commands and returns results
pub struct CommandExecutor;

impl CommandExecutor {
    /// Execute a command and return the result as a message
    pub fn execute(cmd: &Command, context: &CommandContext) -> CommandResult {
        match cmd {
            Command::Help => CommandExecutor::help(),
            Command::Quit => CommandResult::Success("Goodbye!".to_string()),
            Command::Delete => CommandExecutor::delete(context),
            Command::Rename(title) => CommandExecutor::rename(context, title),
            Command::Move(folder) => CommandExecutor::move_note(context, folder),
            Command::Tag(tags) => CommandExecutor::tag(context, tags),
            Command::Export(format) => CommandExecutor::export(context, format),
            Command::Unknown(cmd) => CommandResult::Error(format!("Unknown command: {}", cmd)),
        }
    }

    fn help() -> CommandResult {
        let help_text = r#"Available commands:
:help, :h, :?            - Show this help message
:quit, :q, :exit, :x     - Quit the application
:rename <title>          - Rename current note to <title>
:move <folder>           - Move note to folder (daily, fleeting, permanent, literature, index, reference)
:tag <tags>              - Add tags to current note
:delete, :del, :rm       - Delete current note (with confirmation)
:export <format>         - Export note to format (pdf, html, markdown, txt, docx)"#;
        CommandResult::Success(help_text.to_string())
    }

    fn rename(ctx: &CommandContext, title: &str) -> CommandResult {
        if title.trim().is_empty() {
            return CommandResult::Error("Title cannot be empty".to_string());
        }
        if ctx.note_id.is_none() {
            return CommandResult::Error("No note selected".to_string());
        }
        CommandResult::Success(format!("Renamed to: {}", title))
    }

    fn move_note(ctx: &CommandContext, folder: &str) -> CommandResult {
        if ctx.note_id.is_none() {
            return CommandResult::Error("No note selected".to_string());
        }
        if !CommandParser::is_valid_folder(folder) {
            return CommandResult::Error(format!("Invalid folder: {}. Valid folders are: daily, fleeting, permanent, literature, index, reference", folder));
        }
        CommandResult::Success(format!("Moved to: {}", folder))
    }

    fn tag(ctx: &CommandContext, tags: &[String]) -> CommandResult {
        if ctx.note_id.is_none() {
            return CommandResult::Error("No note selected".to_string());
        }
        if tags.is_empty() {
            return CommandResult::Error("At least one tag required".to_string());
        }
        CommandResult::Success(format!("Added tags: {}", tags.join(", ")))
    }

    fn delete(ctx: &CommandContext) -> CommandResult {
        if ctx.note_id.is_none() {
            return CommandResult::Error("No note selected".to_string());
        }
        CommandResult::Success("Delete confirmation dialog will be shown".to_string())
    }

    fn export(ctx: &CommandContext, format: &str) -> CommandResult {
        if ctx.note_id.is_none() {
            return CommandResult::Error("No note selected".to_string());
        }
        let valid_formats = ["pdf", "html", "markdown", "md", "txt", "docx"];
        if !valid_formats.contains(&format) {
            return CommandResult::Error(format!(
                "Invalid export format: {}. Valid formats are: {}",
                format,
                valid_formats.join(", ")
            ));
        }
        CommandResult::Success(format!("Exporting to: {}", format))
    }
}

/// Context information needed by command executor
#[derive(Debug, Clone)]
pub struct CommandContext {
    pub note_id: Option<String>,
    #[allow(dead_code)]
    pub note_title: Option<String>,
}

impl CommandContext {
    pub fn new() -> Self {
        Self {
            note_id: None,
            note_title: None,
        }
    }

    pub fn with_note(note_id: String, note_title: String) -> Self {
        Self {
            note_id: Some(note_id),
            note_title: Some(note_title),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // HELP COMMAND TESTS
    // ============================================================================

    #[test]
    fn test_parse_help_command() {
        assert_eq!(CommandParser::parse("help"), Command::Help);
    }

    #[test]
    fn test_parse_help_shorthand() {
        assert_eq!(CommandParser::parse("h"), Command::Help);
        assert_eq!(CommandParser::parse("?"), Command::Help);
    }

    #[test]
    fn test_parse_help_case_insensitive() {
        assert_eq!(CommandParser::parse("HELP"), Command::Help);
        assert_eq!(CommandParser::parse("Help"), Command::Help);
        assert_eq!(CommandParser::parse("HeLP"), Command::Help);
    }

    #[test]
    fn test_parse_help_with_whitespace() {
        assert_eq!(CommandParser::parse("  help  "), Command::Help);
        assert_eq!(CommandParser::parse("\thelp\t"), Command::Help);
    }

    // ============================================================================
    // DELETE COMMAND TESTS
    // ============================================================================

    #[test]
    fn test_parse_delete_command() {
        assert_eq!(CommandParser::parse("delete"), Command::Delete);
    }

    #[test]
    fn test_parse_delete_shorthand() {
        assert_eq!(CommandParser::parse("del"), Command::Delete);
        assert_eq!(CommandParser::parse("rm"), Command::Delete);
    }

    #[test]
    fn test_parse_delete_case_insensitive() {
        assert_eq!(CommandParser::parse("DELETE"), Command::Delete);
        assert_eq!(CommandParser::parse("Delete"), Command::Delete);
    }

    // ============================================================================
    // RENAME COMMAND TESTS
    // ============================================================================

    #[test]
    fn test_parse_rename_command() {
        assert_eq!(
            CommandParser::parse("rename New Title"),
            Command::Rename("New Title".to_string())
        );
    }

    #[test]
    fn test_parse_rename_single_word() {
        assert_eq!(
            CommandParser::parse("rename Title"),
            Command::Rename("Title".to_string())
        );
    }

    #[test]
    fn test_parse_rename_with_special_chars() {
        assert_eq!(
            CommandParser::parse("rename My Note (2024)"),
            Command::Rename("My Note (2024)".to_string())
        );
    }

    #[test]
    fn test_parse_rename_shorthand() {
        assert_eq!(
            CommandParser::parse("rn New Title"),
            Command::Rename("New Title".to_string())
        );
    }

    #[test]
    fn test_parse_rename_missing_argument() {
        let result = CommandParser::parse("rename");
        assert!(matches!(result, Command::Unknown(_)));
    }

    #[test]
    fn test_parse_rename_case_insensitive() {
        assert_eq!(
            CommandParser::parse("RENAME New Title"),
            Command::Rename("New Title".to_string())
        );
    }

    // ============================================================================
    // MOVE COMMAND TESTS
    // ============================================================================

    #[test]
    fn test_parse_move_command() {
        assert_eq!(
            CommandParser::parse("move daily"),
            Command::Move("daily".to_string())
        );
    }

    #[test]
    fn test_parse_move_all_valid_folders() {
        let folders = [
            "daily",
            "fleeting",
            "permanent",
            "literature",
            "index",
            "reference",
        ];
        for folder in &folders {
            assert_eq!(
                CommandParser::parse(&format!("move {}", folder)),
                Command::Move(folder.to_string())
            );
        }
    }

    #[test]
    fn test_parse_move_case_insensitive() {
        assert_eq!(
            CommandParser::parse("move DAILY"),
            Command::Move("daily".to_string())
        );
        assert_eq!(
            CommandParser::parse("move Daily"),
            Command::Move("daily".to_string())
        );
    }

    #[test]
    fn test_parse_move_shorthand() {
        assert_eq!(
            CommandParser::parse("mv permanent"),
            Command::Move("permanent".to_string())
        );
    }

    #[test]
    fn test_parse_move_missing_argument() {
        let result = CommandParser::parse("move");
        assert!(matches!(result, Command::Unknown(_)));
    }

    // ============================================================================
    // TAG COMMAND TESTS
    // ============================================================================

    #[test]
    fn test_parse_tag_command() {
        assert_eq!(
            CommandParser::parse("tag rust programming"),
            Command::Tag(vec!["rust".to_string(), "programming".to_string()])
        );
    }

    #[test]
    fn test_parse_tag_single_tag() {
        assert_eq!(
            CommandParser::parse("tag rust"),
            Command::Tag(vec!["rust".to_string()])
        );
    }

    #[test]
    fn test_parse_tag_multiple_tags() {
        assert_eq!(
            CommandParser::parse("tag ai ml nlp deep-learning"),
            Command::Tag(vec![
                "ai".to_string(),
                "ml".to_string(),
                "nlp".to_string(),
                "deep-learning".to_string()
            ])
        );
    }

    #[test]
    fn test_parse_tag_case_preserved() {
        assert_eq!(
            CommandParser::parse("tag Rust Programming"),
            Command::Tag(vec!["Rust".to_string(), "Programming".to_string()])
        );
    }

    #[test]
    fn test_parse_tag_missing_argument() {
        let result = CommandParser::parse("tag");
        assert!(matches!(result, Command::Unknown(_)));
    }

    // ============================================================================
    // EXPORT COMMAND TESTS
    // ============================================================================

    #[test]
    fn test_parse_export_command() {
        assert_eq!(
            CommandParser::parse("export pdf"),
            Command::Export("pdf".to_string())
        );
    }

    #[test]
    fn test_parse_export_various_formats() {
        let formats = ["pdf", "html", "markdown", "md", "txt", "docx"];
        for fmt in &formats {
            assert_eq!(
                CommandParser::parse(&format!("export {}", fmt)),
                Command::Export(fmt.to_string())
            );
        }
    }

    #[test]
    fn test_parse_export_shorthand() {
        assert_eq!(
            CommandParser::parse("exp html"),
            Command::Export("html".to_string())
        );
    }

    #[test]
    fn test_parse_export_case_insensitive() {
        assert_eq!(
            CommandParser::parse("export PDF"),
            Command::Export("pdf".to_string())
        );
    }

    #[test]
    fn test_parse_export_missing_argument() {
        let result = CommandParser::parse("export");
        assert!(matches!(result, Command::Unknown(_)));
    }

    // ============================================================================
    // UNKNOWN COMMAND TESTS
    // ============================================================================

    #[test]
    fn test_parse_unknown_command() {
        let result = CommandParser::parse("foobar");
        assert!(matches!(result, Command::Unknown(_)));
    }

    #[test]
    fn test_parse_empty_string() {
        let result = CommandParser::parse("");
        assert!(matches!(result, Command::Unknown(_)));
    }

    #[test]
    fn test_parse_only_whitespace() {
        let result = CommandParser::parse("   \t  ");
        assert!(matches!(result, Command::Unknown(_)));
    }

    // ============================================================================
    // FOLDER VALIDATION TESTS
    // ============================================================================

    #[test]
    fn test_is_valid_folder_all_valid() {
        let valid_folders = [
            "daily",
            "fleeting",
            "permanent",
            "literature",
            "index",
            "reference",
        ];
        for folder in &valid_folders {
            assert!(CommandParser::is_valid_folder(folder));
        }
    }

    #[test]
    fn test_is_valid_folder_case_insensitive() {
        assert!(CommandParser::is_valid_folder("DAILY"));
        assert!(CommandParser::is_valid_folder("Daily"));
        assert!(CommandParser::is_valid_folder("PERMANENT"));
    }

    #[test]
    fn test_is_valid_folder_invalid() {
        assert!(!CommandParser::is_valid_folder("invalid"));
        assert!(!CommandParser::is_valid_folder("folder"));
        assert!(!CommandParser::is_valid_folder("notes"));
    }

    // ============================================================================
    // COMMAND DISPLAY TESTS
    // ============================================================================

    #[test]
    fn test_command_display_help() {
        assert_eq!(format!("{}", Command::Help), "help");
    }

    #[test]
    fn test_command_display_delete() {
        assert_eq!(format!("{}", Command::Delete), "delete");
    }

    #[test]
    fn test_command_display_rename() {
        assert_eq!(
            format!("{}", Command::Rename("My Title".to_string())),
            "rename My Title"
        );
    }

    #[test]
    fn test_command_display_move() {
        assert_eq!(
            format!("{}", Command::Move("daily".to_string())),
            "move daily"
        );
    }

    #[test]
    fn test_command_display_tag() {
        assert_eq!(
            format!(
                "{}",
                Command::Tag(vec!["rust".to_string(), "ai".to_string()])
            ),
            "tag rust ai"
        );
    }

    #[test]
    fn test_command_display_export() {
        assert_eq!(
            format!("{}", Command::Export("pdf".to_string())),
            "export pdf"
        );
    }

    // ============================================================================
    // COMPLEX PARSING TESTS
    // ============================================================================

    #[test]
    fn test_parse_rename_with_extra_spaces() {
        assert_eq!(
            CommandParser::parse("rename   New   Title"),
            Command::Rename("New   Title".to_string())
        );
    }

    #[test]
    fn test_parse_tag_with_extra_spaces() {
        assert_eq!(
            CommandParser::parse("tag  rust  programming"),
            Command::Tag(vec!["rust".to_string(), "programming".to_string()])
        );
    }

    #[test]
    fn test_parse_move_with_whitespace() {
        assert_eq!(
            CommandParser::parse("  move   daily  "),
            Command::Move("daily".to_string())
        );
    }

    #[test]
    fn test_command_result_success() {
        let result = CommandResult::Success("Renamed successfully".to_string());
        assert_eq!(
            result,
            CommandResult::Success("Renamed successfully".to_string())
        );
    }

    #[test]
    fn test_command_result_error() {
        let result = CommandResult::Error("Note not found".to_string());
        assert_eq!(result, CommandResult::Error("Note not found".to_string()));
    }

    // ============================================================================
    // COMMAND EXECUTOR TESTS - HELP COMMAND
    // ============================================================================

    #[test]
    fn test_execute_help() {
        let ctx = CommandContext::new();
        let result = CommandExecutor::execute(&Command::Help, &ctx);
        match result {
            CommandResult::Success(msg) => {
                assert!(msg.contains("Available commands"));
                assert!(msg.contains(":help"));
                assert!(msg.contains(":rename"));
                assert!(msg.contains(":move"));
            }
            _ => panic!("Expected success"),
        }
    }

    // ============================================================================
    // COMMAND EXECUTOR TESTS - RENAME COMMAND
    // ============================================================================

    #[test]
    fn test_execute_rename_success() {
        let ctx = CommandContext::with_note("note-1".to_string(), "Old Title".to_string());
        let result = CommandExecutor::execute(&Command::Rename("New Title".to_string()), &ctx);
        assert_eq!(
            result,
            CommandResult::Success("Renamed to: New Title".to_string())
        );
    }

    #[test]
    fn test_execute_rename_no_note_selected() {
        let ctx = CommandContext::new();
        let result = CommandExecutor::execute(&Command::Rename("New Title".to_string()), &ctx);
        assert_eq!(result, CommandResult::Error("No note selected".to_string()));
    }

    #[test]
    fn test_execute_rename_empty_title() {
        let ctx = CommandContext::with_note("note-1".to_string(), "Old Title".to_string());
        let result = CommandExecutor::execute(&Command::Rename("   ".to_string()), &ctx);
        assert_eq!(
            result,
            CommandResult::Error("Title cannot be empty".to_string())
        );
    }

    // ============================================================================
    // COMMAND EXECUTOR TESTS - MOVE COMMAND
    // ============================================================================

    #[test]
    fn test_execute_move_success() {
        let ctx = CommandContext::with_note("note-1".to_string(), "Title".to_string());
        let result = CommandExecutor::execute(&Command::Move("daily".to_string()), &ctx);
        assert_eq!(
            result,
            CommandResult::Success("Moved to: daily".to_string())
        );
    }

    #[test]
    fn test_execute_move_all_valid_folders() {
        let folders = [
            "daily",
            "fleeting",
            "permanent",
            "literature",
            "index",
            "reference",
        ];
        let ctx = CommandContext::with_note("note-1".to_string(), "Title".to_string());

        for folder in &folders {
            let result = CommandExecutor::execute(&Command::Move(folder.to_string()), &ctx);
            assert!(matches!(result, CommandResult::Success(_)));
        }
    }

    #[test]
    fn test_execute_move_invalid_folder() {
        let ctx = CommandContext::with_note("note-1".to_string(), "Title".to_string());
        let result = CommandExecutor::execute(&Command::Move("invalid".to_string()), &ctx);
        assert!(matches!(result, CommandResult::Error(_)));
        match result {
            CommandResult::Error(msg) => assert!(msg.contains("Invalid folder")),
            _ => panic!("Expected error"),
        }
    }

    #[test]
    fn test_execute_move_no_note_selected() {
        let ctx = CommandContext::new();
        let result = CommandExecutor::execute(&Command::Move("daily".to_string()), &ctx);
        assert_eq!(result, CommandResult::Error("No note selected".to_string()));
    }

    // ============================================================================
    // COMMAND EXECUTOR TESTS - TAG COMMAND
    // ============================================================================

    #[test]
    fn test_execute_tag_success() {
        let ctx = CommandContext::with_note("note-1".to_string(), "Title".to_string());
        let tags = vec!["rust".to_string(), "programming".to_string()];
        let result = CommandExecutor::execute(&Command::Tag(tags.clone()), &ctx);
        assert!(matches!(result, CommandResult::Success(_)));
        match result {
            CommandResult::Success(msg) => assert!(msg.contains("Added tags")),
            _ => panic!("Expected success"),
        }
    }

    #[test]
    fn test_execute_tag_single_tag() {
        let ctx = CommandContext::with_note("note-1".to_string(), "Title".to_string());
        let tags = vec!["rust".to_string()];
        let result = CommandExecutor::execute(&Command::Tag(tags), &ctx);
        assert!(matches!(result, CommandResult::Success(_)));
    }

    #[test]
    fn test_execute_tag_no_note_selected() {
        let ctx = CommandContext::new();
        let tags = vec!["rust".to_string()];
        let result = CommandExecutor::execute(&Command::Tag(tags), &ctx);
        assert_eq!(result, CommandResult::Error("No note selected".to_string()));
    }

    #[test]
    fn test_execute_tag_empty_tags() {
        let ctx = CommandContext::with_note("note-1".to_string(), "Title".to_string());
        let tags: Vec<String> = vec![];
        let result = CommandExecutor::execute(&Command::Tag(tags), &ctx);
        assert_eq!(
            result,
            CommandResult::Error("At least one tag required".to_string())
        );
    }

    // ============================================================================
    // COMMAND EXECUTOR TESTS - DELETE COMMAND
    // ============================================================================

    #[test]
    fn test_execute_delete_success() {
        let ctx = CommandContext::with_note("note-1".to_string(), "Title".to_string());
        let result = CommandExecutor::execute(&Command::Delete, &ctx);
        assert!(matches!(result, CommandResult::Success(_)));
    }

    #[test]
    fn test_execute_delete_no_note_selected() {
        let ctx = CommandContext::new();
        let result = CommandExecutor::execute(&Command::Delete, &ctx);
        assert_eq!(result, CommandResult::Error("No note selected".to_string()));
    }

    // ============================================================================
    // COMMAND EXECUTOR TESTS - EXPORT COMMAND
    // ============================================================================

    #[test]
    fn test_execute_export_success() {
        let ctx = CommandContext::with_note("note-1".to_string(), "Title".to_string());
        let result = CommandExecutor::execute(&Command::Export("pdf".to_string()), &ctx);
        assert_eq!(
            result,
            CommandResult::Success("Exporting to: pdf".to_string())
        );
    }

    #[test]
    fn test_execute_export_all_valid_formats() {
        let formats = ["pdf", "html", "markdown", "md", "txt", "docx"];
        let ctx = CommandContext::with_note("note-1".to_string(), "Title".to_string());

        for format in &formats {
            let result = CommandExecutor::execute(&Command::Export(format.to_string()), &ctx);
            assert!(matches!(result, CommandResult::Success(_)));
        }
    }

    #[test]
    fn test_execute_export_invalid_format() {
        let ctx = CommandContext::with_note("note-1".to_string(), "Title".to_string());
        let result = CommandExecutor::execute(&Command::Export("xyz".to_string()), &ctx);
        assert!(matches!(result, CommandResult::Error(_)));
        match result {
            CommandResult::Error(msg) => assert!(msg.contains("Invalid export format")),
            _ => panic!("Expected error"),
        }
    }

    #[test]
    fn test_execute_export_no_note_selected() {
        let ctx = CommandContext::new();
        let result = CommandExecutor::execute(&Command::Export("pdf".to_string()), &ctx);
        assert_eq!(result, CommandResult::Error("No note selected".to_string()));
    }

    // ============================================================================
    // COMMAND EXECUTOR TESTS - UNKNOWN COMMAND
    // ============================================================================

    #[test]
    fn test_execute_unknown_command() {
        let ctx = CommandContext::new();
        let result = CommandExecutor::execute(&Command::Unknown("foobar".to_string()), &ctx);
        assert!(matches!(result, CommandResult::Error(_)));
        match result {
            CommandResult::Error(msg) => assert!(msg.contains("Unknown command")),
            _ => panic!("Expected error"),
        }
    }

    // ============================================================================
    // COMMAND CONTEXT TESTS
    // ============================================================================

    #[test]
    fn test_command_context_new() {
        let ctx = CommandContext::new();
        assert_eq!(ctx.note_id, None);
        assert_eq!(ctx.note_title, None);
    }

    #[test]
    fn test_command_context_with_note() {
        let ctx = CommandContext::with_note("note-1".to_string(), "Title".to_string());
        assert_eq!(ctx.note_id, Some("note-1".to_string()));
        assert_eq!(ctx.note_title, Some("Title".to_string()));
    }
}
