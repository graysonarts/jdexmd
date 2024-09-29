use color_eyre::eyre::{Error, OptionExt};

use crate::model::FolderKind;

/// The different kinds of lines that can be parsed
#[derive(Debug)]
pub enum ParsedKind<'topic> {
    /// Area with a range of ids
    Area(u8, u8, &'topic str),
    /// Category with a number id
    Category(u8, &'topic str),
    /// Folder with a number id
    Folder(u8, FolderKind, &'topic str),
    /// Extended folder with a string id
    ExtendedFolder(&'topic str, FolderKind, &'topic str),
}

/// Parses a generic line which could be a category, system, or folder
fn parse_entry(_line_no: usize, trimmed: &str) -> Result<(u8, &str, Option<FolderKind>), Error> {
    let mut parts = trimmed.splitn(2, ' ');
    let id = parts.next().ok_or_eyre("no id found")?;
    let rest = parts.next().unwrap_or_default();
    let style = rest.chars().next().map(FolderKind::from_char);
    let parsed_id = id.parse()?;
    Ok((parsed_id, rest, style))
}

/// Parses a line that is an area
fn parse_area_entry(trimmed: &str) -> Result<(u8, u8, &str), Error> {
    let mut parts = trimmed.splitn(2, ' ');
    let id = parts.next().ok_or_eyre("no id range found")?;
    let rest = parts.next().unwrap_or_default();

    let mut id_parts = id.splitn(2, '-');
    let start = id_parts.next().ok_or_eyre("no start id found")?.parse()?;
    let end = id_parts.next().ok_or_eyre("no end id found")?.parse()?;
    Ok((start, end, rest))
}

/// Parses a line that is an extended folder
fn parse_extended_folder(_line_no: usize, line: &str) -> Result<(&str, FolderKind, &str), Error> {
    let mut parts = line.splitn(2, ' ');
    let id = parts.next().ok_or_eyre("no id found")?;
    let rest = parts.next().unwrap_or_default();
    let style = rest
        .chars()
        .next()
        .map(FolderKind::from_char)
        .unwrap_or_default();

    Ok((id, style, rest))
}

/// Parses a single line into a `LineKind`
pub fn parse_single(line_no: usize, line: &str) -> Result<ParsedKind<'_>, Error> {
    let trimmed = line.trim();
    let line_length = line.len();
    let left_trim_length = line.trim_start().len();
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "The likelihood of this overflowing is pretty low"
    )]
    let indent = line_length - left_trim_length;
    match indent {
        0 => {
            let (start, end, topic) = parse_area_entry(trimmed)?;
            Ok(ParsedKind::Area(start, end, topic))
        }
        1 => {
            let (id, topic, _) = parse_entry(line_no, trimmed)?;
            Ok(ParsedKind::Category(id, topic))
        }
        2 => {
            let (id, topic, style) = parse_entry(line_no, trimmed)?;
            Ok(ParsedKind::Folder(id, style.unwrap_or_default(), topic))
        }
        3 => {
            let (id, style, topic) = parse_extended_folder(line_no, trimmed)?;
            Ok(ParsedKind::ExtendedFolder(id, style, topic))
        }
        _ => Err(Error::msg(format!(
            "Config file is not formatted correctly, error@{line_no}",
        ))),
    }
}
