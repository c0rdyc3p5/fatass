use std::env;
use std::path::Path;
use walkdir::WalkDir;
use colored::Colorize;
use std::time::Instant;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Debug, Clone)]
struct FileData {
    path: String,
    size: i64,
}

impl FileData {
    fn new(path: String, size: i64) -> FileData {
        FileData { path, size }
    }

    fn get_str_size(&self) -> String {
        let mut size = self.size as f64;
        let mut count = 0;
        let mut suffix = String::from("Bytes");

        let units: [&str; 8] = ["KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];

        for unit in &units {
            if size < 1024.0 {
                break;
            }
            size /= 1024.0;
            count += 1;
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

fn print_help() {
    println!("Usage: your_program [OPTIONS]");

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

fn reverse_binary_search_insert_index(arr: &[FileData], target_size: &i64) -> usize {
    let mut low = 0;
    let mut high = arr.len();

    while low != high {
        let mid = (low + high) / 2;

        match arr[mid].size.cmp(target_size) {
            std::cmp::Ordering::Equal => return mid,
            std::cmp::Ordering::Less => high = mid,
            std::cmp::Ordering::Greater => low = mid + 1,
        }
    }

    low
}

// Get args from command line
fn main() {
    let runtime_start = Instant::now();
    let args: Vec<String> = env::args().collect();
    let mut search_path: String = String::from("./");
    let mut fatass_count: i16 = 100;

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
            if let Ok(parsed_count) = count_value.parse::<i16>() {
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
    println!("{}", "Preparing ...".blue());
    let mut max_files: u64 = 0;
    for _entry in WalkDir::new(&search_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !e.file_type().is_dir())
        .filter(|e| e.metadata().map(|m| m.len()).unwrap_or(0) != 0)
    {
        max_files += 1;
    }
    let progress_bar = ProgressBar::new(max_files);
    progress_bar.set_style(ProgressStyle::with_template("[{elapsed_precise}] {bar:50.cyan/blue} {pos:>7}/{len:7} {msg}")
        .unwrap()
        .progress_chars("##-"));

    // Create an array to store biggest files
    let mut biggest_files: Vec<FileData> = Vec::with_capacity(fatass_count as usize);
    let mut reordered = false;
    for entry in WalkDir::new(&search_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !e.file_type().is_dir())
        .filter(|e| e.metadata().map(|m| m.len()).unwrap_or(0) != 0)
    {
        let file_data = FileData::new(
            entry.path().display().to_string(),
            entry.metadata().map(|m| m.len()).unwrap_or(0) as i64
        );

        if biggest_files.len() < fatass_count as usize {
            biggest_files.push(file_data);
        } else if biggest_files.len() == fatass_count as usize && reordered == false {
            biggest_files.sort_by(|a, b| b.size.cmp(&a.size));
            reordered = true;
        } else  {
            let index = reverse_binary_search_insert_index(&biggest_files, &file_data.size);
            biggest_files.insert(index, file_data);
            biggest_files.pop();
        }

        progress_bar.inc(1);
    }
    progress_bar.finish();

    for file_data in biggest_files {
        let path_str = format!("{}", file_data.path);
        let size_str = format!("({})", file_data.get_str_size());

        let path_colored = path_str.green();
        let size_colored = size_str.yellow();

        println!("{} {}", path_colored, size_colored);
    }

    let end_message = format!("Found the fattest {} files in {:?}", fatass_count, runtime_start.elapsed()).bright_cyan();
    println!("\n{}", end_message);
}
