use colored::*;
use rayon::prelude::*;
use std::cmp;
use std::env;
use std::fmt::Debug;
use std::fmt::Display;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::sync::Mutex;

fn read_lines_to_vec(file_path: &str) -> io::Result<Vec<String>> {
    let path = Path::new(file_path);
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut lines = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(content) => lines.push(content),
            Err(_) => continue, // Optionally handle read errors
        }
    }

    Ok(lines)
}

fn save_vec_to_file<T: Display>(vec: Vec<T>, file_path: &str) -> io::Result<()> {
    let file = File::create(file_path)?; // Create (or overwrite) the file
    let mut writer = io::BufWriter::new(file); // Use a buffered writer for efficiency

    for item in vec {
        writeln!(writer, "{}", item)?; // Writes each item on a new line
    }
    Ok(())
}

fn zip_merge<'a, T>(arr1: &'a [T], arr2: &'a [T]) -> Vec<T>
where
    T: PartialEq + Clone + Debug, // T needs to implement PartialEq for comparison, and Clone for copying elements
{
    // Determine the longest and shortest arrays upfront
    let (longest_arr, shortest_arr) = if arr1.len() > arr2.len() {
        (arr1, arr2) // arr1 is the longest, arr2 is the shortest
    } else {
        (arr2, arr1) // arr2 is the longest, arr1 is the shortest
    };

    let mut max_len = 0;
    let mut max_start1 = 0;
    let mut max_end1 = 0;
    let mut max_start2 = 0;
    let mut max_end2 = 0;

    // Find the longest common subsequence
    let mut i = 0;
    while i < longest_arr.len() {
        let mut j = 0;
        while j < shortest_arr.len() {
            let mut len = 0;
            // Compare starting at i in longest_arr and j in shortest_arr
            while i + len < longest_arr.len()
                && j + len < shortest_arr.len()
                && longest_arr[i + len] == shortest_arr[j + len]
            {
                len += 1;
            }

            // Update maximum length and indices if a longer match is found
            if len > max_len {
                max_len = len;
                max_start1 = i;
                max_end1 = i + len; // end is exclusive
                max_start2 = j;
                max_end2 = j + len; // end is exclusive
            }

            j += 1;
        }

        i += 1;
    }

    if max_len > 0 {
        // If common subsequence is found, return the 5 parts
        let longest_start = &longest_arr[0..max_start1]; // part before common subsequence in longest arr
        let longest_end = &longest_arr[max_end1..]; // part after common subsequence in longest arr

        let shortest_start = &shortest_arr[0..max_start2]; // part before common subsequence in shortest arr
        let shortest_end = &shortest_arr[max_end2..]; // part after common subsequence in shortest arr

        let common = &longest_arr[max_start1..max_end1]; // the common subsequence
        let mut combined_start = zip_merge(longest_start, shortest_start);
        let mut combined_end = zip_merge(longest_end, shortest_end);
        let mut combined =
            Vec::with_capacity(shortest_arr.len() + longest_arr.len() + common.len());
        combined.append(&mut combined_start); // Add the shortest array
        combined.extend_from_slice(common); // Add the shortest array
        combined.append(&mut combined_end); // Add the longest array
        combined
    } else {
        // If no common subsequence, concatenate the longest array to the end of the shortest array
        if shortest_arr.len() > 0 || longest_arr.len() > 0 {
            println!("{:=<20}", "*");
            println!("{}", format!("{:?}", shortest_arr).blue());
            println!("{:-<20}", "|");
            println!("{}", format!("{:?}", longest_arr).green());
            println!("{:=<20}", "*");
        }
        let mut combined = Vec::with_capacity(shortest_arr.len() + longest_arr.len());
        combined.extend_from_slice(shortest_arr); // Add the shortest array
        combined.extend_from_slice(longest_arr); // Add the longest array

        combined // Return the combined array as an owned Vec<T>
    }
}

fn deduplicate_patterns<T>(arr: &Vec<T>, thourougness: usize) -> Vec<T>
where
    T: PartialEq + Clone + Send + Sync,
{
    let current_array = arr.clone();
    let n = current_array.len();
    // Wrap the indices_to_remove in a Mutex for safe mutable access across threads
    let indices_to_remove = Mutex::new(Vec::new());

    // Parallelize the outer loop over sizes
    for size in 1..=n / 2 {
        // Parallelize the inner loop over starting positions
        if size * 100 / (n / 2) > 10 && (size * 100 / (n / 2)) % thourougness != 0 {
            continue;
        }
        (0..size).into_par_iter().for_each(|start| {
            let mut i = start;

            // Jump by window size
            while i + size <= n {
                // let current_array = current_array.lock().unwrap();
                let window = current_array[i..i + size].to_vec();

                // Look for another window starting from a different index
                let j = i + size;
                if j + size <= n {
                    let next_window = current_array[j..j + size].to_vec();
                    if window == next_window {
                        // Mark the repeated window's indices for removal
                        let mut indices = indices_to_remove.lock().unwrap();
                        for k in i..i + size {
                            indices.push(k);
                        }
                    }
                }
                // Jump the window by its size
                i += size;
            }
        });
        if !indices_to_remove.lock().unwrap().is_empty() {
            let mut deduplicated_array = Vec::new();
            for (i, item) in current_array.iter().enumerate() {
                if !indices_to_remove.lock().unwrap().contains(&(i)) {
                    deduplicated_array.push(item.clone());
                }
            }
            return deduplicate_patterns(&deduplicated_array, thourougness);
        }
    }
    return current_array;
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: zip_merge <input_file1> <input_file2> <output_file>");
        return Ok(());
    }

    // Assign input files and output file from arguments
    let input_file1 = &args[1];
    let input_file2 = &args[2];
    let output_file = &args[3];

    println!("Reading filepaths");
    let file1_lines = read_lines_to_vec(&input_file1)?;
    let file2_lines = read_lines_to_vec(&input_file2)?;
    println!("Reading done");
    println!("Running Step 1");
    let dedup1 = deduplicate_patterns(&file1_lines, 20);
    println!("Running Step 2");
    let dedup2 = deduplicate_patterns(&file2_lines, 20);
    println!("Running Step 3");
    let merge = &zip_merge(&dedup1[..], &dedup2[..]);
    println!("Running Step 4");
    let merge_dedup = deduplicate_patterns(merge, 1);

    println!(
        "File1 Len: {}, File2 Len: {}, Merged Len: {}, Best Case: {}, Worst Case: {}",
        format!("{}", file1_lines.len()).blue(),
        format!("{}", file2_lines.len()).blue(),
        format!("{}", merge_dedup.len()).green(),
        format!("{}", cmp::max(file1_lines.len(), file2_lines.len())).cyan(),
        format!("{}", file1_lines.len() + file2_lines.len()).red(),
    );

    println!("Data has been saved to: {}", output_file);
    // println!("Input 1: {}", format!("{:?}", file1_lines).red());
    // println!("Input 2: {}", format!("{:?}", file2_lines).yellow());
    // println!("Merge R: {}", format!("{:?}", merge_dedup).cyan());
    save_vec_to_file(merge_dedup, &output_file)?;

    Ok(())
}
