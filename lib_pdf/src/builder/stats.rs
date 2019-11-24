use std::{
    time::{Instant},
};

pub struct BuilderStats {
    pages: Vec<PageStat>,
    time_started: Instant,
}
impl BuilderStats {
    pub fn new() -> BuilderStats {
        BuilderStats {
            pages: Vec::new(),
            time_started: Instant::now(),
        }
    }

    pub fn add_page(&mut self, page: PageStat) { self.pages.push(page); }

    pub fn print_with_name(&self, name: &str) {
        let mut total_size = 0;
        let mut colour_count = 0;
        let mut gray_count = 0;
        let mut colour_size = 0;
        let mut gray_size = 0;
        let mut colour_indices = Vec::new();
        let mut gray_indices = colour_indices.clone();
        for (i, page) in self.pages.iter().enumerate() {
            match page {
                PageStat::Colour(size) => {
                    total_size += size;
                    colour_count += 1;
                    colour_size += size;
                    push_next_index(&mut colour_indices, i);
                },
                PageStat::Gray(size) => {
                    total_size += size;
                    gray_count += 1;
                    gray_size += size;
                    push_next_index(&mut gray_indices, i);
                },
            }
        }
        let colour_indices_pretty = make_pretty_indices(&colour_indices);
        let gray_indices_pretty = make_pretty_indices(&gray_indices);

        println!("{name} completed in {time:?}!
    Total Page Size: {total_size}

    Colour Page Count: {colour_count}
    Colour Page Total Size: {colour_size}
    Colour Page Indices: {colour_indices}

    Gray Page Count: {gray_count}
    Gray Page Total Size: {gray_size}
    Gray Page Indices: {gray_indices}",
            name=name, time=self.time_started.elapsed(),
            total_size=display_pretty_bytes(total_size),
            colour_count=colour_count,
            gray_count=gray_count,
            colour_size=display_pretty_bytes(colour_size),
            gray_size=display_pretty_bytes(gray_size),
            colour_indices=colour_indices_pretty,
            gray_indices=gray_indices_pretty);
    }
}

pub enum PageStat {
    // A colour page and the size in bytes
    Colour(u64),
    // A gray page and the size in bytes
    Gray(u64),
}

/// We need to make sure that all of the pages indices in the same list are adjacent
fn push_next_index(indices: &mut Vec< Vec<usize> >, index: usize) {
    let need_to_push = match indices.last_mut() {
        None => true,
        Some(index_list) => {
            let last_index = index_list.last().unwrap();
            if last_index + 1 == index {
                index_list.push(index);
                false
            } else {
                true
            }
        },
    };
    if need_to_push {
        indices.push(vec![index]);
    }
}

fn make_pretty_indices(indices: &[Vec<usize>]) -> String {
    let mut pretty_indices = String::new();
    for index_list in indices.iter().take(indices.len() - 1) {
        if let Some(pretty_index) = make_pretty_index(index_list) {
            pretty_indices += &pretty_index;
            pretty_indices += ", ";
        }
    }
    if let Some(last_index_list) = indices.last() {
        if let Some(pretty_index) = make_pretty_index(last_index_list) {
            pretty_indices += &pretty_index;
        }
    }

    pretty_indices
}
fn make_pretty_index(index_list: &Vec<usize>) -> Option<String> {
    if index_list.is_empty() {
        return None;
    }
    if index_list.len() == 1 {
        Some(index_list[0].to_string())
    } else {
        let index_iter = index_list.iter().map(|i| *i);
        let lower_bound = index_iter.clone().min().unwrap();
        let upper_bound = index_iter.max().unwrap();
        Some(format!("{} - {}", lower_bound, upper_bound))
    }
}

fn display_pretty_bytes(bytes: u64) -> String {
    let suffixes = vec![
        ("GB", 1 << 30),
        ("MB", 1 << 20),
        ("KB", 1 << 10),
    ];

    for (suffix, threshold) in suffixes {
        if bytes >= threshold {
            return format!("{:.3} {}", (bytes as f64) / (threshold as f64), suffix);
        }
    }
    format!("{} B", bytes)
}
