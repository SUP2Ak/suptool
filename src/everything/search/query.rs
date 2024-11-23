use std::str::FromStr;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub text: String,
    pub filters: Vec<SearchFilter>,
    pub regex: Option<Regex>,
}

#[derive(Debug, Clone)]
pub enum SearchFilter {
    Size(SizeFilter),
    Date(DateFilter),
    Path(String),
    Type(FileType),
}

#[derive(Debug, Clone)]
pub enum SizeFilter {
    Exact(u64),
    Greater(u64),
    Less(u64),
    Range(u64, u64),
}

#[derive(Debug, Clone)]
pub enum DateFilter {
    Before(i64),
    After(i64),
    Between(i64, i64),
}

#[derive(Debug, Clone)]
pub enum FileType {
    File,
    Directory,
    Extension(String),
}

impl SearchQuery {
    pub fn parse(query: &str) -> Self {
        let mut filters = Vec::new();
        let mut text_parts = Vec::new();
        let mut regex = None;

        for part in query.split_whitespace() {
            if let Some(filter) = part.strip_prefix("size:") {
                if let Ok(size_filter) = SizeFilter::from_str(filter) {
                    filters.push(SearchFilter::Size(size_filter));
                }
            } else if let Some(filter) = part.strip_prefix("date:") {
                if let Ok(date_filter) = DateFilter::from_str(filter) {
                    filters.push(SearchFilter::Date(date_filter));
                }
            } else if let Some(path) = part.strip_prefix("path:") {
                filters.push(SearchFilter::Path(path.to_string()));
            } else if let Some(r) = part.strip_prefix("regex:") {
                if let Ok(re) = Regex::new(r) {
                    regex = Some(re);
                }
            } else {
                text_parts.push(part);
            }
        }

        Self {
            text: text_parts.join(" "),
            filters,
            regex,
        }
    }
}

impl FromStr for SizeFilter {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(range) = s.split_once("..") {
            let start = range.0.parse().map_err(|_| "Invalid size")?;
            let end = range.1.parse().map_err(|_| "Invalid size")?;
            Ok(SizeFilter::Range(start, end))
        } else if let Some(size) = s.strip_prefix(">") {
            let size = size.parse().map_err(|_| "Invalid size")?;
            Ok(SizeFilter::Greater(size))
        } else if let Some(size) = s.strip_prefix("<") {
            let size = size.parse().map_err(|_| "Invalid size")?;
            Ok(SizeFilter::Less(size))
        } else {
            let size = s.parse().map_err(|_| "Invalid size")?;
            Ok(SizeFilter::Exact(size))
        }
    }
}

impl FromStr for DateFilter {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(range) = s.split_once("..") {
            let start = range.0.parse().map_err(|_| "Invalid date")?;
            let end = range.1.parse().map_err(|_| "Invalid date")?;
            Ok(DateFilter::Between(start, end))
        } else if let Some(date) = s.strip_prefix(">") {
            let date = date.parse().map_err(|_| "Invalid date")?;
            Ok(DateFilter::After(date))
        } else if let Some(date) = s.strip_prefix("<") {
            let date = date.parse().map_err(|_| "Invalid date")?;
            Ok(DateFilter::Before(date))
        } else {
            Err("Invalid date format")
        }
    }
}
