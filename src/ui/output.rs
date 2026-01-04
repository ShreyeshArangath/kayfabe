use console::style;

pub struct Output;

impl Output {
    pub fn success(message: &str) {
        println!("{} {}", style("✓").green().bold(), message);
    }

    pub fn error(message: &str) {
        eprintln!("{} {}", style("✗").red().bold(), message);
    }

    pub fn warning(message: &str) {
        println!("{} {}", style("⚠").yellow().bold(), message);
    }

    pub fn info(message: &str) {
        println!("{} {}", style("ℹ").cyan().bold(), message);
    }

    pub fn step(current: usize, total: usize, message: &str) {
        println!(
            "{} {}",
            style(format!("[{}/{}]", current, total)).cyan(),
            message
        );
    }

    pub fn header(message: &str) {
        println!("\n{}", style(message).bold().underlined());
    }

    pub fn section(title: &str) {
        println!("\n{}", style(title).bold());
    }
}
