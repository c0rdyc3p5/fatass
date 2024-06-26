use std::env;
use std::path::Path;
use walkdir::WalkDir;
use colored::Colorize;
use std::time::Instant;
use indicatif::{ProgressBar, ProgressStyle};
use tabled::{
    settings::{
        object::{Columns, Rows}, Alignment, Style,
        style::BorderColor,
        themes::Colorization, Color
    },
    Tabled,
    Table
};

#[derive(Debug, Clone)]
struct FileData {
    path: String,
    size: u64,
}

impl FileData {
    fn new(path: String, size: u64) -> FileData {
        FileData { path, size }
    }

    fn get_str_size(&self) -> String {
        let mut size = self.size as f64;
        let mut suffix = String::from("Bytes");

        let units: [&str; 8] = ["KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];

        for unit in units {
            if size < 1024.0 {
                break;
            }
            size /= 1024.0;
            suffix = unit.to_string();
        }

        let size_str = if size.fract() == 0.0 {
            format!("{:.0}", size)
        } else {
            format!("{:.2}", size)
        };

        format!("{} {}", size_str, suffix)
    }
}

#[allow(non_snake_case)]
#[derive(Tabled)]
struct FileDataTable {
    Path: String,
    Size: String,
}

#[allow(non_snake_case)]
impl FileDataTable {
    fn new(Path: String, Size: String) -> FileDataTable {
        FileDataTable { Path, Size }
    }
}

fn print_help() {
    println!("Usage: fatass [OPTIONS]");

    println!("\nOptions:");
    println!("  --help, -h           Show this help message and exit");
    println!("  --path, -p <PATH>    Set the search path (default: ./)");
    println!("  --count, -c <COUNT>  Set the fatass count (default: 100)");

    println!("\nExamples:");
    println!("  fatass --path /some/path --count 50");
    println!("  fatass -p /another/path -c 75");

    println!("\nNote:");
    println!("  If the provided path or count value contains spaces, enclose it in quotes.");
}

fn reverse_binary_search_insert_index(arr: &[FileData], target_size: &u64) -> Option<usize> {
    let mut low = 0;
    let mut high = arr.len();

    // Check if smaller than the smaller file, if so return none to skip
    if target_size < &arr[arr.len() - 1].size {
        return None;
    }

    while low != high {
        let mid = (low + high) / 2;

        match arr[mid].size.cmp(target_size) {
            std::cmp::Ordering::Equal => return Some(mid),
            std::cmp::Ordering::Less => high = mid,
            std::cmp::Ordering::Greater => low = mid + 1,
        }
    }

    Some(low)
}

// Get args from command line
fn main() {
    let runtime_start = Instant::now();
    let args: Vec<String> = env::args().collect();
    let mut search_path: String = String::from("./");
    let mut fatass_count: usize = 100;

    // Check if help was asked
    if let Some(_index) = args.iter().position(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return;
    }

    // Check if path was given, if so, set it
    if let Some(index) = args.iter().position(|arg| arg == "--path" || arg == "-p") {
        // Check if there is a value after "--path"
        if let Some(path_value) = args.get(index + 1) {
            let path = Path::new(path_value);

            if path.exists() {
                // Path is valid, assign it to search_path
                search_path = path.to_string_lossy().to_string();
            } else {
                eprintln!("{}", "Error: Invalid path. Please provide a valid path.".red());
                return;
            }
        } else {
            eprintln!("{}", "Error: No value provided after --path option.".red());
            return;
        }
    }

    // Check if count was, if so, set it
    if let Some(index) = args.iter().position(|arg| arg == "--count" || arg == "-c") {
        // Check if there is a value after "--count"
        if let Some(count_value) = args.get(index + 1) {
            if let Ok(parsed_count) = count_value.parse::<usize>() {
                fatass_count = parsed_count;
            } else {
                eprintln!("{}", "Error: Invalid count value. Please provide a valid number.".red());
                return;
            }
        } else {
            eprintln!("{}", "Error: No value provided after --count option.".red());
            return;
        }
    }

    // Count the number of file to check
    println!("{}", "Gathering files ...".cyan());

    let walker = WalkDir::new(&search_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !e.file_type().is_dir())
        .filter(|e| e.metadata().map(|m| m.len()).unwrap_or(0) != 0)
        .collect::<Vec<_>>();
    let total_files = walker.len() as u64;

    let progress_bar = ProgressBar::new(total_files);
    progress_bar.set_style(ProgressStyle::with_template("[{elapsed_precise}] {bar:50.cyan/blue} {pos:>7}/{len:7} {msg}")
        .unwrap()
        .progress_chars("##-"));

    // Create an array to store biggest files
    let mut biggest_files: Vec<FileData> = Vec::with_capacity(fatass_count);
    let mut reordered = false;
    for entry in walker
    {
        let file_data = FileData::new(
            entry.path().display().to_string(),
            entry.metadata().map(|m| m.len()).unwrap_or(0) as u64
        );

        if biggest_files.len() < fatass_count {
            // We fill the vec its not to its capacity
            biggest_files.push(file_data);
        } else if biggest_files.len() == fatass_count && reordered == false {
            // We reorder the current files in the vector because its at its capacity and we need it sorted for binary search
            biggest_files.sort_by(|a, b| b.size.cmp(&a.size));
            reordered = true;
        } else  {
            // We search where the current file should be in the vec, if none is return it means the current file is smaller than the smaller file in the vector
            if let Some(i) = reverse_binary_search_insert_index(&biggest_files, &file_data.size) {
                biggest_files.insert(i, file_data);
                biggest_files.pop();
            }
        }

        progress_bar.inc(1);
    }
    progress_bar.finish();

    let tabled_files: Vec<FileDataTable> = biggest_files.iter().map(|file_data| {
        FileDataTable::new(
            file_data.path.clone(),
            file_data.get_str_size()
        )
    }).collect();

    let mut table = Table::new(&tabled_files);
    table
        .with(Style::rounded())
        .with(BorderColor::filled(Color::FG_GREEN))
        .with(Colorization::columns([Color::FG_CYAN, Color::FG_BRIGHT_RED]))
        .with(Colorization::exact([Color::FG_GREEN], Rows::first()))
        .modify(Columns::last(), Alignment::right());

    println!("{}", table.to_string());

    let end_message = format!("Found the fattest {} files in {:?}", fatass_count, runtime_start.elapsed()).green();
    println!("{}", end_message);
}