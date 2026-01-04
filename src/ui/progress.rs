use indicatif::{ProgressBar as IndicatifBar, ProgressStyle};

pub struct ProgressBar {
    bar: IndicatifBar,
}

impl ProgressBar {
    pub fn new(len: u64, message: &str) -> Self {
        let bar = IndicatifBar::new(len);
        bar.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
                )
                .unwrap()
                .progress_chars("#>-"),
        );
        bar.set_message(message.to_string());

        Self { bar }
    }

    pub fn spinner(message: &str) -> Self {
        let bar = IndicatifBar::new_spinner();
        bar.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        bar.set_message(message.to_string());
        bar.enable_steady_tick(std::time::Duration::from_millis(100));

        Self { bar }
    }

    pub fn inc(&self, delta: u64) {
        self.bar.inc(delta);
    }

    pub fn set_message(&self, message: &str) {
        self.bar.set_message(message.to_string());
    }

    pub fn finish_with_message(&self, message: &str) {
        self.bar.finish_with_message(message.to_string());
    }

    pub fn finish(&self) {
        self.bar.finish();
    }
}
