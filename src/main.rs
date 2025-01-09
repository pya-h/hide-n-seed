use std::{fs, io};
use dotenv::dotenv;
use std::env;

#[macro_export] macro_rules! concat_bytes {
        ($($x:expr), *) => {
            {
                let mut v: Vec<u8> = Vec::new();
                $(
                    v.extend($x);
                )*
                v
            }
        };
}

#[macro_export] macro_rules! beep {
     () => {
         print!("\x07");
     };
 }

fn try_reading_string(input_message: &str, target: &mut String) -> bool {
    // Simple function which ensures getting valid input from user, and also allow user to cancel input.
    let mut temp_input = String::new();
    loop {
        println!("{}", input_message);
        let reading_result = io::stdin().read_line(&mut temp_input);
        if let Err(err) = reading_result {
            println!("Invalid Path Idiot!: {}\nEnter empty input to cancel ...", err.to_string());
            continue;
        }
        *target = temp_input.to_string();
        if target.trim().len() == 0 {
            break;
        }
        return true;
    }
    false
}
fn read_file_bytes(filename: &str) -> Vec<u8> {
    match fs::read(filename) {
        Ok(bytes) => bytes,
        Err(ref err) if err.kind() == io::ErrorKind::NotFound => panic!("File:{} not found.", filename),
        Err(err) => panic!("{}", err),
    }
}
fn hide_secret_file(image_path: &str, secret_file_path: &str, output_path: &str, file_separator_bytes: &[u8]) -> Result<(), io::Error> {
    let image_data = read_file_bytes(image_path);
    let secret_file_path = read_file_bytes(secret_file_path);

    let output_data = concat_bytes!(image_data, file_separator_bytes, secret_file_path);
    fs::write(output_path, output_data)
}

fn extract_secret_file(bytes: &Vec<u8>, start: usize, end: usize, file_number: usize) -> Result<(), io::Error> {
    let end = if end >= bytes.len() {
        bytes.len()
    } else {
        end
    };
    fs::write(format!("secret_x_{}", file_number), &bytes[start..end])
}

fn start_extracting_secret_files(bytes: &Vec<u8>, mut start_index: usize, file_separator_bytes: &[u8]) -> usize {
    let mut i = start_index;
    let length = bytes.len();
    let separator_length = file_separator_bytes.len();
    let mut separator_index = 0;
    let mut files_extracted = 0;

    while i < length {
        if separator_index < separator_length {
            if &bytes[i] == &file_separator_bytes[separator_index] {
                separator_index += 1;
            } else {
                separator_index = 0;
            }
        } else {
            match extract_secret_file(&bytes, start_index, i - separator_length, files_extracted + 1) {
                Err(err) => {
                    println!("Error extracting a file, skipping its data; Reason: {}", err.to_string())
                }
                _ => {
                    files_extracted += 1;
                    println!("File#{} extracted", files_extracted);
                }
            }
            separator_index = 0;
            start_index = i;
        }
        i += 1;
    }
    match extract_secret_file(&bytes, start_index, length, files_extracted + 1) {
        Err(err) => {
            println!("Error writing extracted data inside a new file; Reason: {}", err.to_string())
        }
        _ => {
            files_extracted += 1;
            println!("File#{} extracted", files_extracted);
        }
    };
    files_extracted
}

fn process_combined_file(filename: &str, file_separator_bytes: &[u8]) -> Result<(), String> {
    let separator_length = file_separator_bytes.len();
    match fs::read(filename)  {
        Ok(bytes) => {
            let mut separator_index = 0;
            let mut i = 0;
            let bytes_length = bytes.len();

            while i < bytes_length {
                if separator_index < separator_length {
                    if &bytes[i] == &file_separator_bytes[separator_index] {
                        separator_index += 1;
                    } else {
                        separator_index = 0;
                    }
                } else {
                    if start_extracting_secret_files(&bytes, i, &file_separator_bytes) == 0 {
                        return Err(String::from("Found some secret files but couldn't extract them."));
                    }
                    break;
                }
                i += 1;
            }
        }
        Err(ref err) if err.kind() == io::ErrorKind::NotFound => {
            return Err(String::from("Can not find the file specified!"));
        }
        Err(err) => {
            return Err(err.to_string());
        }
    };
    Ok(())
}
fn main() {
    dotenv().ok();
    let file_separator_string = env::var("FILE_SEPARATOR_TEXT")
        .expect("FILE_SEPARATOR_TEXT is not set! It must be set and never change.");
    let file_separator_bytes = file_separator_string.as_bytes();
    beep!();
    loop {
        let mut operation = String::new();
        try_reading_string(
            "- - - - - - - - - - - - - - - - - - - - - - - - - - - - -\nSELECT OPERATION:\n\t[H]IDE\n\t[E]XTRACT\n\t[Q]UIT\n- - - - - - - - - - - - - - - - - - - - - - - - - - - - -",
                         &mut operation);

        match operation.trim() {
            "H" | "h" => {
                let mut image_path = String::new();
                let mut secret_file_path = String::new();
                let mut output_path = String::new();

                if !try_reading_string("Image Path: ", &mut image_path)
                    || !try_reading_string("Secret File Path: ", &mut secret_file_path)
                    || !try_reading_string("Output Path: ", &mut output_path) {
                    continue;
                }
                println!("Processing ...");
                match hide_secret_file(&image_path.trim(), &secret_file_path.trim(), &output_path.trim(), &file_separator_bytes) {
                    Ok(()) => { beep!(); println!("Successfully hid your requested file inside the image."); },
                    Err(err) => println!("FUCK! Failed to hide your requested file:\tReason:\n{}", err),
                };
            },
            "E" | "e" => {
                let mut combined_file_path = String::new();

                if !try_reading_string("Combined [By Me] File Path: ", &mut combined_file_path) {
                    continue;
                }
                println!("{}", combined_file_path.trim());
                if let Err(err) = process_combined_file(combined_file_path.trim(), &file_separator_bytes) {
                    println!("FUCK! {}", err);
                } else {
                    beep!();
                }
            }
            "Q" | "q" | "" => {
                beep!();
                println!("FUCK U & HAVE A NICE DAY.");
                break;
            }
            _ => {
                println!("BITE ME.");
            }
        }
    }

}
