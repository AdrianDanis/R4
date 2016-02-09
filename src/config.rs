//! Boot configuration information for the kernel
use core::str::Split;
use util::*;

/// Boot configuration parameters
pub struct BootConfig<'a> {
    /// Command line option string that is typically passed at run time by
    /// the boot loader
    cmdline: &'a str,
}

impl<'a> BootConfig<'a> {
    /// Construct a new instance of the configuration
    pub fn new(s: &'a str) -> BootConfig<'a> {
        BootConfig { cmdline: s }
    }
    /// Return an iterator over the raw parts of the command line
    pub fn cmdline_iter(&self) -> Split<'a, char> {
        self.cmdline.split(' ')
    }
    /// Iterate over any A=B pairs in the command line
    pub fn cmdline_option_iter(&self) -> CommandLineOptionIter<'a> {
        CommandLineOptionIter { splits: self.cmdline_iter() }
    }
    /// Find a particular command line value
    pub fn cmdline_option_find(&self, name: &str) -> Option<CommandLineOption> {
        self.cmdline_option_iter().find(|ref s| s.name == name)
    }
    /// Retrieve an option by integer value
    pub fn cmdline_option_from_str<T: FromStrExt>(&self, name: &str) -> Option<T> {
        self.cmdline_option_find(name)
            .and_then(|ref option| T::from_str_prefix(option.value).ok())
    }
}

pub struct CommandLineOption<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

pub struct CommandLineOptionIter<'a> {
    splits: Split<'a, char>,
}

impl<'a> Iterator for CommandLineOptionIter<'a> {
    type Item = CommandLineOption<'a>;
    fn next(&mut self) -> Option<CommandLineOption<'a>> {
        loop {
            match self.splits.next() {
                Some(s) => {
                    let mut iter = s.split('=');
                    match iter.next() {
                    Some(left) => match iter.next() {
                        Some(right) => return Some(CommandLineOption{name: left, value: right}),
                        None => (),
                    },
                    None => (),
                }},
                None => return None,
            };
        }
    }
}
