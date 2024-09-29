use color_eyre::eyre::{Error, OptionExt};

use crate::model::FolderKind;

#[derive(Debug)]
pub enum LineKind<'a> {
    Area(u8, u8, &'a str),
    Category(u8, &'a str),
    Folder(u8, FolderKind, &'a str),
    ExtendedFolder(&'a str, FolderKind, &'a str),
}

fn parse_entry(line_no: usize, trimmed: &str) -> Result<(u8, &str, Option<FolderKind>), Error> {
    let mut parts = trimmed.splitn(2, ' ');
    let id = parts.next().ok_or_eyre("no id found")?;
    let rest = parts.next().unwrap_or_default();
    let style = rest.chars().next().map(|c| FolderKind::from_char(c));
    let id = id.parse()?;
    Ok((id, rest, style))
}

fn parse_area_entry(trimmed: &str) -> Result<(u8, u8, &str), Error> {
    let mut parts = trimmed.splitn(2, ' ');
    let id = parts.next().ok_or_eyre("no id range found")?;
    let rest = parts.next().unwrap_or_default();

    let mut id_parts = id.splitn(2, '-');
    let start = id_parts.next().ok_or_eyre("no start id found")?.parse()?;
    let end = id_parts.next().ok_or_eyre("no end id found")?.parse()?;
    Ok((start, end, rest))
}

fn parse_extended_folder<'a>(
    line_no: usize,
    line: &'a str,
) -> Result<(&'a str, FolderKind, &'a str), Error> {
    let mut parts = line.splitn(2, ' ');
    let id = parts.next().ok_or_eyre("no id found")?;
    let rest = parts.next().unwrap_or_default();
    let style = rest
        .chars()
        .next()
        .map(|c| FolderKind::from_char(c))
        .unwrap_or_default();

    Ok((id, style, rest))
}

pub fn parse_line<'a>(line_no: usize, line: &'a str) -> Result<LineKind<'a>, Error> {
    let trimmed = line.trim();
    let line_length = line.len();
    let left_trim_length = line.trim_start().len();
    let indent = line_length - left_trim_length;
    Ok(match indent {
        0 => {
            let (start, end, topic) = parse_area_entry(trimmed)?;
            Ok(LineKind::Area(start, end, topic))
        }
        1 => {
            let (id, topic, _) = parse_entry(line_no, trimmed)?;
            Ok(LineKind::Category(id, topic))
        }
        2 => {
            let (id, topic, style) = parse_entry(line_no, trimmed)?;
            Ok(LineKind::Folder(id, style.unwrap_or_default(), topic))
        }
        3 => {
            let (id, style, topic) = parse_extended_folder(line_no, trimmed)?;
            Ok(LineKind::ExtendedFolder(id, style, topic))
        }
        _ => Err(Error::msg(format!(
            "Config file is not formatted correctly, error@{line_no}",
        ))),
    }?)
}
