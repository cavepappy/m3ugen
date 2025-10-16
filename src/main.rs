// Description: Given a directory path of child dirs, create a .m3u file based on the contents of
//              the current child dirs files.
//              This should include creating a sub-dir to hold the original .chd or .cue files,
//              moving those files into that sub-dir, and creating a new file that contains the
//              new paths to the original files.
// Author: Wilson (cavepappy) Miller
// Date: 10/15/2025

use std::{
    env, ffi, fs,
    io::{self, BufWriter, Write},
};

/// Verify that a path exists and is valid
fn verify_path(path: &str) -> Result<bool, io::Error> {
    match fs::exists(path) {
        Ok(d) => Ok(d),
        Err(e) => Err(e),
    }
}

/// Get the name of the last chunk of a path
fn get_path_dir_name(path: &str) -> String {
    let dir: Vec<&str> = path
        .split(match env::consts::OS {
            "windows" => "\\",
            _ => "/",
        })
        .collect();
    dir.last().unwrap().to_string()
}

/// Combine the provided string into a valid path format
fn build_path_from_parts(parts: &Vec<&str>) -> ffi::OsString {
    let mut ret = ffi::OsString::new();
    let ret_size = parts.iter().fold(0, |acc, s| acc + s.len());
    ret.reserve(ret_size);

    let total = parts.len();
    let mut count = 0;
    parts.iter().for_each(|part| {
        count += 1;

        ret.push(part);

        // never put a trailing / or \
        if count != total {
            ret.push(match env::consts::OS {
                "windows" => "\\",
                _ => "/",
            });
        }
    });
    ret
}

// TODO skip directories that already have a sub directory containing a .m3u file OR only have one chd/set of bin/cue files
// TODO create a log file that contains any directories that have both chd and bin/cue files
// TODO integrate ratatui to create an optional interface (by passing --tui maybe?)
fn main() -> io::Result<()> {
    // step 1: get input from the user
    let args: Vec<String> = env::args().collect();

    // step 2a: set the parent directory
    let path_to_parent: &str = &args[1];

    // step 3a: verify that the path is valid
    let _ = verify_path(path_to_parent).or_else(|e| Err(e));

    // step 3b: get a list of child dirs in the parent dir
    let child_dirs = fs::read_dir(path_to_parent)?;

    // step 5a: write the path (sub-dir/file_name) to a .m3u file and move the files into the
    //         sub-dir
    child_dirs.for_each(|dir| {
        let curr = dir.unwrap();
        let curr_name = get_path_dir_name(curr.path().to_str().unwrap());

        // Create a hidden subdirectory string
        let mut hidden_name: String = String::new();
        hidden_name.push_str(".");
        hidden_name.push_str(get_path_dir_name(curr.path().to_str().unwrap()).as_str());

        // step 5b: make a path to the sub_directory
        let sub_dir = build_path_from_parts(&vec![path_to_parent, &curr_name, &hidden_name]);

        // step 5c: verify the parent dir still exists
        let _ = verify_path(path_to_parent).or_else(|e| Err(e));

        // step 5d: create a sub-dir for this file (if it doesn't already exist)
        let _ = fs::create_dir(&sub_dir).or_else(|e| Err(e));

        // Build path to the output .m3u file
        let mut file_name = String::new();
        file_name.push_str(curr_name.as_str());
        file_name.push_str(".m3u");

        // Create the output file
        let outfile = fs::File::create(build_path_from_parts(&vec![
            &path_to_parent,
            curr_name.as_str(),
            file_name.as_str(),
        ]))
        .unwrap();

        // step 5f: move the .cue or .chd files to the sub_dir and write to our .m3u file
        let _ = verify_path(sub_dir.to_str().unwrap()).or_else(|e| Err(e));
        let files = fs::read_dir(&curr.path()).unwrap();

        // loop through the files in the current directory
        for file in files {
            let curr_file = file.unwrap().path().to_str().unwrap().to_string();
            let curr_file_name = get_path_dir_name(&curr_file);

            // Skip the file if it's anything other than our data files
            if !["chd", "cue", "bin"]
                .iter()
                .any(|ext| curr_file.ends_with(ext))
            {
                continue;
            }

            // build the path that we want to move our data files to
            let new_file = build_path_from_parts(&vec![
                path_to_parent,
                curr_name.as_str(),
                hidden_name.as_str(),
                get_path_dir_name(curr_file.as_str()).as_str(),
            ]);

            // write to m3u_file
            let file_m3u_line =
                build_path_from_parts(&vec![hidden_name.as_str(), curr_file_name.as_str()]);
            let mut buf = BufWriter::new(&outfile);
            let _ = buf.write(file_m3u_line.to_str().unwrap().as_bytes());
            let _ = buf.write(b"\n");

            // move file
            match fs::rename(&curr_file, &new_file) {
                Ok(_) => (),
                Err(e) => println!(
                    "ERROR ({e}): Unable to move {} to {}",
                    curr_file,
                    new_file.to_str().unwrap()
                ),
            }
        }
    });
    Ok(())
}
