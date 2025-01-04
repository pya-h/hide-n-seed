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

    let output_dta = concat_bytes!(image_data, SEPARATOR, secret_file_path);
    fs::write(output_path, output_dta)
}

fn extract_secret_bytes(bytes: &Vec<u8>, start_index: usize) -> usize {

    0
}

fn extract_secret_file(filename: &str) {
    let separator_length = SEPARATOR.len();
  match fs::read(filename)  {
    Ok(bytes) => {
        let mut separator_index = 0;
        let mut i = 0;
        let bytes_length = bytes.len();

        while i <= bytes_length {
            if i != separator_length {

                if &bytes[i] == &SEPARATOR[separator_index] {
                    separator_index += 1;
                } else {
                    separator_index = 0;
                }
            } else {
                i += extract_secret_bytes(&bytes, i + 1);
                separator_index = 0;
            }
            i += 1;
        }
    }
    Err(ref err) if err.kind() == io::ErrorKind::NotFound => {

    }
    Err(err) => {

    }
  } 
}

fn main() {
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
        Ok(()) => println!("Successfully hid your requested file inside the image."),
        Err(err) => println!("Failed to hide your requested file:\tReason:\n{}", err),
    };

    // TODO: Add extracting ,,,
}
