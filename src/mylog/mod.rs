use colored::Colorize;
use std::fmt::Display;

pub fn nok<D>(text: D)
where
    D: Display,
{
    println!("{} {}", "[-]".red(), text.to_string().red())
}

pub fn _ok<D>(text: D)
where
    D: Display,
{
    println!("{} {}", "[+]".green(), text.to_string().green())
}
