use std::{fs, io};

const SEPARATOR: &[u8] = "###$$$!!!WHO-KNOWS-WHAT-COMES-NEXT!!!$$$###".as_bytes();

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
fn read_file_bytes(filename: &str) -> Vec<u8> {
    match fs::read(filename) {
        Ok(bytes) => bytes,
        Err(ref err) if err.kind() == io::ErrorKind::NotFound => panic!("File:{} not found.", filename),
        Err(err) => panic!("{}", err),
    }
}
fn hide_secret_file(image_path: &str, secret_file_path: &str, output_path: &str) -> Result<(), io::Error> {
    let image_data = read_file_bytes(image_path);
    let secret_file_path = read_file_bytes(secret_file_path);

    let output_data = concat_bytes!(image_data, SEPARATOR, secret_file_path);
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

fn start_extracting_secret_files(bytes: &Vec<u8>, mut start_index: usize) -> usize {
    let mut i = start_index;
    let length = bytes.len();
    let separator_length = SEPARATOR.len();
    let mut separator_index = 0;
    let mut files_extracted = 0;

    while i < length {
        if separator_index < separator_length {
            if &bytes[i] == &SEPARATOR[separator_index] {
                separator_index += 1;
            } else {
                separator_index = 0;
            }
        } else {
            match extract_secret_file(&bytes, start_index, i - separator_length, files_extracted + 1) {
                Err(err) => {
                    println!("Error ectracting file: {}", err.to_string())
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

        }
        _ => {
            files_extracted += 1;
            println!("File#{} extracted", files_extracted);
        }
    };
    files_extracted
}

fn process_combined_file(filename: &str) -> Result<(), String> {
    let separator_length = SEPARATOR.len();
    match fs::read(filename)  {
        Ok(bytes) => {
            let mut separator_index = 0;
            let mut i = 0;
            let bytes_length = bytes.len();

            while i < bytes_length {
                if separator_index < separator_length {
                    if &bytes[i] == &SEPARATOR[separator_index] {
                        separator_index += 1;
                    } else {
                        separator_index = 0;
                    }
                } else {
                    if start_extracting_secret_files(&bytes, i) == 0 {
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
    beep!();
    loop {
        let mut operation = String::new();
        println!("- - - - - - - - - - - - - - - - - - - - - - - - - - - - -");
        println!("SELECT OPERATION:\n\t[H]IDE\n\t[E]XTRACT\n\t[Q]UIT");
        println!("- - - - - - - - - - - - - - - - - - - - - - - - - - - - -");
        io::stdin().read_line(&mut operation).unwrap();
        match operation.trim() {
            "H" | "h" => {
                let mut image_path = String::new();
                let mut secret_file_path = String::new();

                println!("Image Path: ");
                io::stdin().read_line(&mut image_path).unwrap();
                println!("File Path: ");
                io::stdin().read_line(&mut secret_file_path).unwrap();

                let mut output_path = String::new();
                println!("Output Path: ");
                io::stdin().read_line(&mut output_path).unwrap();  // TODO: It's better to automatically put the image extension at the end of the filename,
                // since user may enter wrong or mismatched extension

                println!("Processing ...");
                match hide_secret_file(&image_path.trim(), &secret_file_path.trim(), &output_path.trim()) {
                    Ok(()) => { beep!(); println!("Successfully hid your requested file inside the image."); },
                    Err(err) => println!("FUCK! Failed to hide your requested file:\tReason:\n{}", err),
                };
            },
            "E" | "e" => {
                let mut combined_file_path = String::new();

                println!("File Path: ");
                io::stdin().read_line(&mut combined_file_path).unwrap(); // TODO: Replace these .unwrap and panic usages too, since they crash app thread
                if let Err(err) = process_combined_file(combined_file_path.trim()) {
                    println!("FUCK! {}", err);
                } else {
                    beep!();
                }
            }
            "Q" | "q" => {
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
