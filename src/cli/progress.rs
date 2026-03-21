use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::core::installer::{InstallReporter, InstallTaskReporter};

pub struct CliProgressReporter {
    multi: MultiProgress,
}

impl CliProgressReporter {
    pub fn new() -> std::sync::Arc<Self> {
        std::sync::Arc::new(Self {
            multi: MultiProgress::new(),
        })
    }
}

impl InstallReporter for CliProgressReporter {
    #[allow(clippy::literal_string_with_formatting_args)]
    fn create_task(&self, name: &str) -> Box<dyn InstallTaskReporter> {
        let pb = self.multi.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::with_template("{spinner:.cyan} {msg}")
                .unwrap()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
        );
        pb.set_message(format!("Syncing: {name}"));

        Box::new(CliTaskReporter { pb })
    }

    #[allow(clippy::literal_string_with_formatting_args)]
    fn create_spinner(&self, msg: &str) -> Box<dyn InstallTaskReporter> {
        let pb = self.multi.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::with_template("{spinner:.magenta} {msg}")
                .unwrap()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈"),
        );
        pb.set_message(msg.to_string());
        Box::new(CliTaskReporter { pb })
    }
}

struct CliTaskReporter {
    pb: ProgressBar,
}

impl InstallTaskReporter for CliTaskReporter {
    fn set_message(&self, msg: String) {
        self.pb.set_message(msg);
    }

    fn finish_success(&self, msg: String) {
        self.pb.finish_with_message(format!("\x1b[32m✓\x1b[0m {msg}"));
    }

    fn finish_error(&self, msg: String) {
        self.pb.finish_with_message(format!("\x1b[31m✗\x1b[0m {msg}"));
    }

    fn warning(&self, msg: String) {
        self.pb.println(format!("\x1b[33m⚠ Warning:\x1b[0m {msg}"));
    }

    fn print_line(&self, msg: String) {
        self.pb.println(msg);
    }

    fn finish_and_clear(&self) {
        self.pb.finish_and_clear();
    }
}
