use crate::{
    console::console_color::ConsoleColor,
    pcp::atom::{Atom, AtomContent},
};

pub struct PcpPrinter<'a> {
    stack: Vec<i32>,
    color: &'a ConsoleColor,
}

impl<'a> PcpPrinter<'a> {
    pub fn new(color: &'a ConsoleColor) -> Self {
        Self {
            stack: Vec::new(),
            color,
        }
    }

    pub fn print(&mut self, atom: &Atom) {
        let indent = (0..self.stack.len())
            .map(|_| "  ")
            .collect::<Vec<_>>()
            .join("");
        println!(
            "{}{}{}{}",
            self.color.header(),
            indent,
            atom,
            self.color.footer()
        );
        if let AtomContent::Parent(parent) = atom.content() {
            self.stack.push(parent.count());
        } else {
            if let Some(last) = self.stack.last_mut() {
                *last -= 1;
                if *last == 0 {
                    self.stack.pop();
                }
            }
        }
    }
}

pub struct HttpPrinter<'a> {
    first: bool,
    color: &'a ConsoleColor,
}

impl<'a> HttpPrinter<'a> {
    pub fn new(color: &'a ConsoleColor) -> Self {
        Self { first: true, color }
    }

    pub fn print(&mut self, text: &str) {
        if self.first {
            println!(
                "{}==== HTTP Header Start ===={}",
                self.color.header(),
                self.color.footer(),
            );
            self.first = false;
        }
        print!(
            "{}{}{}",
            self.color.header(),
            text.replace("\r", "<CR>").replace("\n", "<LF>\n"),
            self.color.footer(),
        );
    }

    pub fn print_eos(&self) {
        println!("{}<EOS>{}", self.color.header(), self.color.footer());
        println!(
            "{}===== HTTP Header End ====={}",
            self.color.header(),
            self.color.footer()
        );
    }
}
