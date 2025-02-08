use rayon::prelude::*;
use rprompt::prompt_reply;
use std::fmt::Display;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

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
    T: PartialEq + Clone, // T needs to implement PartialEq for comparison, and Clone for copying elements
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
        let mut combined = Vec::with_capacity(shortest_arr.len() + longest_arr.len());
        combined.extend_from_slice(shortest_arr); // Add the shortest array
        combined.extend_from_slice(longest_arr); // Add the longest array

        combined // Return the combined array as an owned Vec<T>
    }
}
fn main() -> io::Result<()> {
    // Prompt for 3 file paths
    let input_file1 = prompt_reply("Enter the first input file path: ")?;
    let input_file2 = prompt_reply("Enter the second input file path: ")?;
    let output_file = prompt_reply("Enter the output file path: ")?;

    println!("Input File 1: {}", input_file1);
    println!("Input File 2: {}", input_file2);
    println!("Output File: {}", output_file);

    let file1_lines = read_lines_to_vec(&input_file1)?;
    let file2_lines = read_lines_to_vec(&input_file2)?;
    let merge_result = deduplicate_patterns(zip_merge(&file1_lines[..], &file2_lines[..]));

    println!(
        "F1Len: {}, F2Len: {}, MergedLen: {}, SimpleSum: {}",
        file1_lines.len(),
        file2_lines.len(),
        merge_result.len(),
        file1_lines.len() + file2_lines.len()
    );

    save_vec_to_file(merge_result, &output_file)?;

    println!("Data has been saved to: {}", output_file);

    Ok(())
}

fn deduplicate_patterns<T>(arr: Vec<T>) -> Vec<T>
where
    T: PartialEq + Clone + Send + Sync,
{
    let current_array = Arc::new(Mutex::new(arr.clone()));
    let n = current_array.lock().unwrap().len();
    let indices_to_remove = Arc::new(Mutex::new(Vec::new()));
    let progress = Arc::new(AtomicUsize::new(0)); // Progress counter

    // Parallelize the outer loop over sizes
    (1..=n / 2).into_par_iter().for_each(|size| {
        // Parallelize the inner loop over starting positions
        (0..size).into_par_iter().for_each(|start| {
            let mut i = start;
            let total_work = (n / 2) * (n / 2);

            // Jump by window size
            while i + size <= n {
                let current_array = current_array.lock().unwrap();
                let window = current_array[i..i + size].to_vec();

                // Look for another window starting from a different index
                let j = i + size;
                if j + size <= n {
                    let next_window = current_array[j..j + size].to_vec();
                    if window == next_window {
                        // Mark the repeated window's indices for removal
                        let mut remove_guard = indices_to_remove.lock().unwrap();
                        for k in i..i + size {
                            remove_guard.push(k);
                        }
                    }
                }
                progress.fetch_add(1, Ordering::SeqCst);

                // Jump the window by its size
                i += size;
                if progress.load(Ordering::SeqCst) % (total_work / 200) == 0 {
                    println!(
                        "Progress: {}% done",
                        progress.load(Ordering::SeqCst) * 100 / total_work
                    );
                }
            }
        });
    });

    // Deduplicate the array based on the collected indices
    let indices_to_remove = indices_to_remove.lock().unwrap();
    let current_array = current_array.lock().unwrap();
    let mut deduplicated_array = Vec::new();
    for (i, item) in current_array.iter().enumerate() {
        if !indices_to_remove.contains(&(i)) {
            deduplicated_array.push(item.clone());
        }
    }

    deduplicated_array
}
