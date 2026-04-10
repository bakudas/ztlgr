use indicatif::{ProgressBar, ProgressStyle};

/// Processing phases for ingest workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessingPhase {
    ReadingSource,
    Converting,
    SendingToLLM,
    CreatingNote,
    UpdatingIndex,
}

impl ProcessingPhase {
    fn message(&self) -> &'static str {
        match self {
            Self::ReadingSource => "Reading source file",
            Self::Converting => "Converting to Markdown",
            Self::SendingToLLM => "Processing with LLM",
            Self::CreatingNote => "Creating literature note",
            Self::UpdatingIndex => "Updating index",
        }
    }

    fn spinner_chars(&self) -> Vec<&'static str> {
        match self {
            Self::SendingToLLM => vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            _ => vec!["◐", "◓", "◑", "◒"],
        }
    }
}

/// Multi-stage progress indicator for CLI operations.
pub struct ProcessProgress {
    spinner: ProgressBar,
    current_phase: ProcessingPhase,
}

impl ProcessProgress {
    /// Create a new progress indicator.
    pub fn new() -> Self {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .expect("Invalid progress template"),
        );

        Self {
            spinner,
            current_phase: ProcessingPhase::ReadingSource,
        }
    }

    /// Advance to a new processing phase.
    pub fn set_phase(&mut self, phase: ProcessingPhase) {
        self.current_phase = phase;
        self.spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .expect("Invalid progress template")
                .tick_strings(&phase.spinner_chars()),
        );
        self.spinner.set_message(phase.message());
        self.spinner
            .enable_steady_tick(std::time::Duration::from_millis(80));
    }

    /// Mark the operation as successfully completed.
    pub fn finish_success(&self, message: &str) {
        self.spinner.finish_with_message(format!("✓ {}", message));
    }

    /// Mark the operation as failed.
    pub fn finish_error(&self, message: &str) {
        self.spinner.finish_with_message(format!("✗ {}", message));
    }

    /// Update the current message without changing phase.
    pub fn set_message(&self, message: &str) {
        self.spinner.set_message(message.to_string());
    }

    /// Print a secondary status line (doesn't affect spinner).
    pub fn println(&self, message: &str) {
        self.spinner.println(message);
    }
}

impl Default for ProcessProgress {
    fn default() -> Self {
        Self::new()
    }
}

/// Progress indicator for simple operations.
pub struct SimpleProgress {
    spinner: ProgressBar,
}

impl SimpleProgress {
    /// Create a new simple progress indicator.
    pub fn new(message: impl Into<String>) -> Self {
        let message = message.into();
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .expect("Invalid progress template"),
        );
        spinner.set_message(message);
        spinner.enable_steady_tick(std::time::Duration::from_millis(80));

        Self { spinner }
    }

    /// Finish with success message.
    pub fn finish_success(self, message: &str) {
        self.spinner.finish_with_message(format!("✓ {}", message));
    }

    /// Finish with error message.
    pub fn finish_error(self, message: &str) {
        self.spinner.finish_with_message(format!("✗ {}", message));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processing_phase_message() {
        assert_eq!(
            ProcessingPhase::ReadingSource.message(),
            "Reading source file"
        );
        assert_eq!(
            ProcessingPhase::SendingToLLM.message(),
            "Processing with LLM"
        );
    }

    #[test]
    fn test_processing_phase_spinner_chars() {
        assert_eq!(ProcessingPhase::SendingToLLM.spinner_chars().len(), 10);
        assert_eq!(ProcessingPhase::ReadingSource.spinner_chars().len(), 4);
    }

    #[test]
    fn test_process_progress_creation() {
        let progress = ProcessProgress::new();
        assert_eq!(progress.current_phase, ProcessingPhase::ReadingSource);
    }

    #[test]
    fn test_simple_progress_creation() {
        let _progress = SimpleProgress::new("Test operation");
    }
}
