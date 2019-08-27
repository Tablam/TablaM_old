use tablam_core;

mod ast;
mod interpreter;
mod lexer;

#[cfg(test)]
mod tests;

fn main() {
    println!("Guess the number!");

    println!("Please input your guess.");
}
