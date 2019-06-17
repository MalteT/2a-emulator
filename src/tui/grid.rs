use tui::layout::Rect;
use unicode_width::UnicodeWidthStr;

use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;

// TODO: Verify all characters when collecting slots
const HORIZONTAL: char = '┄';
const VERTICAL: char = '┊';
const UPPER_RIGHT: char = '┐';
const LOWER_LEFT: char = '└';
const LOWER_RIGHT: char = '┘';

#[derive(Debug)]
pub struct StrGrid<'a> {
    grid: Vec<Cow<'a, str>>,
    // TODO: Rename to slots
    spots: HashMap<usize, Rect>,
    width: usize,
    height: usize,
}

#[derive(Debug)]
pub enum Error {
    InvalidTemplateError,
    SlotSizeMismatch,
    InvalidSlotID,
    NotARectangle,
}

impl StrGrid<'_> {
    /// Get the width of the grid.
    pub fn width(&self) -> usize {
        self.width
    }
    /// Get the height of the grid.
    pub fn height(&self) -> usize {
        self.height
    }
    /// Get char at position.
    pub fn get(&self, x: usize, y: usize) -> Option<char> {
        self.grid.get(y).and_then(|s| s.chars().nth(x))
    }
    /// Fill a spot with the given string block.
    pub fn put(&mut self, id: usize, s: &str) -> Result<(), Error> {
        let slot = self.spots.get(&id).ok_or(Error::InvalidSlotID)?;
        let lines: Vec<Cow<str>> = s.lines().map(|s| s.into()).collect();
        let height = lines.len();
        let width = lines.iter().map(|s| s.width()).max().unwrap_or(0);
        // Check correct dimensions
        if height != slot.height as usize || width != slot.width as usize {
            return Err(Error::SlotSizeMismatch);
        }
        for i in slot.x..slot.x + slot.width {
            for j in slot.y..slot.y + slot.height {
                let i = i as usize;
                let j = j as usize;
                let c = lines
                    .get(j - slot.y as usize)
                    .and_then(|l| l.chars().nth(i - slot.x as usize))
                    .unwrap_or(' ');
                let mut cs: Vec<_> = self.grid[j].chars().collect();
                cs[i] = c;
                let string: String = cs.iter().collect();
                self.grid[j] = string.into();
            }
        }

        Ok(())
    }
    /// Find and collect all "spots".
    /// Spots are blanks in the template, that can be filled and have to look like this:
    /// ```txt
    /// %n┄...┄┄┐
    /// ┊       ┊
    /// .       .
    /// .       .
    /// .       .
    /// ┊       ┊
    /// └┄┄...┄┄┘
    /// ```
    /// The `.`s are not part of the spot, but to demonstrate that these spots can have any
    /// size. They just have to be rectangular. The `n` must be replaced by the id, that
    /// this spot should have.
    ///
    /// # Panic
    /// This method panics if called twice.
    fn collect_spots(&mut self) -> Result<(), Error> {
        if !self.spots.is_empty() {
            panic!("`collect_spots` must not be called twice");
        }
        for j in 0..self.height() {
            for i in 0..self.width() {
                let c = self.get(i, j).unwrap_or(' ');
                let next_c = self.get(i + 1, j).unwrap_or(' ');
                if c == '%' && next_c != '%' {
                    let start_x = i;
                    let start_y = j;
                    let cs = self.grid[j].chars();
                    let number: usize = cs
                        .skip(i + 1)
                        .take_while(char::is_ascii_digit)
                        .collect::<String>()
                        .parse()
                        .map_err(|_| Error::InvalidTemplateError)?;
                    let mut width = 1;
                    let mut c = self.get(i + width, j);
                    while c != Some(UPPER_RIGHT) {
                        // Return error if no end is found
                        if c.is_none() {
                            return Err(Error::InvalidTemplateError);
                        }
                        width += 1;
                        c = self.get(i + width, j);
                    }
                    let mut height = 1;
                    while self.get(i, j + height) != Some(LOWER_LEFT) {
                        // Return error if no end is found
                        if self.get(i, j + height).is_none() {
                            return Err(Error::InvalidTemplateError);
                        }
                        // Return error if the character is invalid
                        height += 1;
                    }
                    // Error if the last corner cannot be found
                    if self.get(i + width, j + height) != Some(LOWER_RIGHT) {
                        return Err(Error::InvalidTemplateError);
                    }
                    self.spots.insert(
                        number,
                        Rect::new(
                            start_x as u16,
                            start_y as u16,
                            width as u16 + 1,
                            height as u16 + 1,
                        ),
                    );
                }
            }
        }
        Ok(())
    }
}

impl<'a> TryFrom<&'a str> for StrGrid<'a> {
    type Error = Error;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        let grid: Vec<Cow<str>> = s.lines().map(|s| s.into()).collect();
        let width = grid.iter().map(|s| s.len()).max().unwrap_or(0);
        let height = grid.len();
        let spots = HashMap::new();
        let mut ret = Self {
            grid,
            width,
            height,
            spots,
        };
        ret.collect_spots()?;
        Ok(ret)
    }
}

impl fmt::Display for StrGrid<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in &self.grid {
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
}
