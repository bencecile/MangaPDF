use std::cmp::{Ord, Ordering};
use std::str::{Chars};

/// An iterator over the naturals of a string
pub struct NaturalIterator<'s> {
    chars: Chars<'s>,
    /// The latest numbers in the string
    numbers: String,
    leftover_char: Option<char>,
}
impl <'s> NaturalIterator<'s> {
    pub fn new(string: &'s str) -> NaturalIterator<'s> {
        NaturalIterator {
            chars: string.chars(),
            numbers: String::new(),
            leftover_char: None,
        }
    }

    /// This is just a helper method
    /// Returns None if the numbers string is empty
    fn make_number(&mut self) -> Option<Natural> {
        if self.numbers.len() > 0 {
            let natural_number = Natural::number(&self.numbers);
            self.numbers.clear();
            Some(natural_number)
        } else {
            None
        }
    }
}
impl <'s> Iterator for NaturalIterator<'s> {
    type Item = Natural;

    fn next(&mut self) -> Option<Self::Item> {
        // Check for a left-over that we have to return before we go to the next one
        if let Some(c) = self.leftover_char.take() {
            return Some(Natural::Char(c));
        }

        // Look at the next char
        if let Some(c) = self.chars.next() {
            // Put the number in the list if it is one
            if '0' <= c && c <= '9' {
                self.numbers.push(c);
                // Go through another iteration because we haven't found a number yet
                self.next()
            } else {
                // Parse our last numbers before we do the current char
                let natural_number = self.make_number();
                if natural_number.is_some() {
                    self.leftover_char = Some(c);
                    natural_number
                } else {
                    Some(Natural::Char(c))
                }
            }
        } else {
            // See if there's any left-over numbers that we need to convert
            self.make_number()
        }
    }
}

/// These are the different types of naturals that can exist in a string
#[derive(PartialEq, Eq)]
pub enum Natural {
    /// The first char of the number to compare against actual chars
    Number(char, u64),
    Char(char),
}
impl Natural {
    // Creates a number from the given string
    fn number(string: &str) -> Natural {
        // Take the first character from the string (assume it's there)
        let c = string.chars().next().unwrap();
        Natural::Number(c, string.parse().unwrap())
    }
}
impl Ord for Natural {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Natural::Number(c1, value1) => {
                match other {
                    Natural::Number(_, value2) => value1.cmp(value2),
                    Natural::Char(c2) => c1.cmp(c2),
                }
            },
            Natural::Char(c1) => {
                match other {
                    Natural::Number(c2, _) |
                    Natural::Char(c2) => c1.cmp(c2),
                }
            }
        }
    }
}
/// This implementation is needed for Ord
impl PartialOrd for Natural {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
