use color_eyre::eyre::{Error, OptionExt};

#[derive(Debug)]
pub enum EntryStyle {
    Folder,
    File,
    Both,
}

impl EntryStyle {
    pub fn from_str(s: &str) -> Self {
        match s {
            "-" => EntryStyle::File,
            "+" => EntryStyle::Both,
            _ => EntryStyle::Folder,
        }
    }
}

#[derive(Debug)]
pub enum LineKind<'a> {
    Area(u8, u8, &'a str),
    Category(u8, &'a str),
    Folder(u8, EntryStyle, &'a str),
    ExtendedFolder(&'a str, EntryStyle, &'a str),
}

fn parse_entry(trimmed: &str) -> Result<(u8, &str, Option<EntryStyle>), Error> {
    let mut parts = trimmed.splitn(2, ' ');
    let id = parts.next().ok_or_eyre("no id found")?;
    let rest = parts.next().unwrap_or_default();
    let style = match rest.chars().next() {
        Some('+') => Some(EntryStyle::Both),
        Some('-') => Some(EntryStyle::File),
        _ => None,
    };

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
) -> Result<(&'a str, EntryStyle, &'a str), Error> {
    let mut parts = line.splitn(2, ' ');
    let id = parts.next().ok_or_eyre("no id found")?;
    let rest = parts.next().unwrap_or_default();
    let style = match rest.chars().next() {
        Some('+') => EntryStyle::Both,
        Some('-') => EntryStyle::File,
        _ => EntryStyle::Folder,
    };

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
            let (id, topic, _) = parse_entry(trimmed)?;
            Ok(LineKind::Category(id, topic))
        }
        2 => {
            let (id, topic, style) = parse_entry(trimmed)?;
            Ok(LineKind::Folder(
                id,
                style.unwrap_or(EntryStyle::Folder),
                topic,
            ))
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
