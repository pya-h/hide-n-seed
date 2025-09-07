use dotenvy_macro::dotenv;
use hide_n_seed::encryptor;
use std::{fs, io};

#[macro_export]
macro_rules! concat_bytes {
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

#[macro_export]
macro_rules! beep {
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
            println!(
                "Invalid Path Idiot!: {}\nEnter empty input to cancel ...",
                err.to_string()
            );
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

fn get_your_password(retry_count: u8) -> String {
    if retry_count > 3 {
        println!("FUCK U! Too many retries!");
        return String::new();
    }
    println!("Password: ");
    match rpassword::read_password() {
        Ok(password) => {
            println!("Confirm: ");
            match rpassword::read_password() {
                Ok(confirm) => {
                    if password != confirm {
                        println!("FUCK U! Passwords don't match!");
                        return get_your_password(retry_count + 1);
                    }
                    return password;
                }
                Err(err) => {
                    println!("FUCK! Failed to read your password:\tReason:\n{}", err);
                }
            }
        }
        Err(err) => {
            println!("FUCK! Failed to read your password:\tReason:\n{}", err);
        }
    };
    get_your_password(retry_count + 1)
}

fn read_file_bytes(filename: &str) -> Vec<u8> {
    match fs::read(filename) {
        Ok(bytes) => bytes,
        Err(ref err) if err.kind() == io::ErrorKind::NotFound => {
            panic!("File:{} not found.", filename)
        }
        Err(err) => panic!("{}", err),
    }
}

fn get_short_filename(filename: &str) -> &str {
    let filename_as_bytes = filename.as_bytes();
    for i in (0..filename.len()).rev() {
        if filename_as_bytes[i] as char == '/' {
            return &filename[i + 1..];
        }
    }
    &filename[..]
}

fn hide_secret_file(
    image_path: &str,
    secret_files_path: &[String],
    output_path: &str,
    file_separator_bytes: &[u8],
    password: String,
    secret_key_bytes: &[u8; 32],
    ignore_troubled_files: bool,
) -> Result<(), io::Error> {
    return match encryptor::encrypt(&password, &secret_key_bytes) {
        Ok((nonce, ciphered_password)) => {
            println!("Processing ...");
            let encrypted_password =
                hex::decode(format!("{}{}", nonce, ciphered_password)).unwrap();
            let mut output_data: Vec<u8> = concat_bytes!(
                read_file_bytes(image_path),
                file_separator_bytes,
                encrypted_password
            );
            let password_as_bytes = encryptor::string_to_fixed_array(&password);
            for path in secret_files_path {
                let short_filename = get_short_filename(path);
                match String::from_utf8(read_file_bytes(path))
                    .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
                {
                    Ok(secret_file) => match encryptor::encrypt(&secret_file, &password_as_bytes) {
                        Ok((file_nonce, encrypted_file_data)) => {
                            let final_encrypted_file_data =
                                hex::decode(format!("{}{}", file_nonce, encrypted_file_data))
                                    .unwrap();
                            output_data = concat_bytes!(
                                output_data,
                                file_separator_bytes,
                                short_filename.as_bytes(),
                                "/".as_bytes(),
                                final_encrypted_file_data
                            );
                        }
                        Err(err) => {
                            if !ignore_troubled_files {
                                return Err(err);
                            }
                            println!("Ignoring file:{} in hide list: {}", short_filename, err);
                        }
                    },
                    Err(err) => {
                        if !ignore_troubled_files {
                            return Err(err);
                        }
                        println!("Ignoring file:{} in hide list: {}", short_filename, err);
                    }
                }
            }

            fs::write(output_path, output_data)
        }
        Err(err) => {
            println!("FUCK! Failed to encrypt your password! Process cancelled.");
            Err(err)
        }
    };
}

fn extract_secret_file(
    bytes: &Vec<u8>,
    start: usize,
    end: usize,
    file_number: usize,
    secret: &[u8; 32],
) -> Result<String, io::Error> {
    let end = if end >= bytes.len() { bytes.len() } else { end };
    let mut file_data_start = start;
    while file_data_start < end && bytes[file_data_start] as char != '/' {
        file_data_start += 1;
    }
    let mut final_file_name: String = String::new();
    let mut encrypted_file_data_with_nonce: &[u8] = &[];
    if file_data_start < end {
        if let Ok(filename) = std::str::from_utf8(&bytes[start..file_data_start]) {
            encrypted_file_data_with_nonce = &bytes[file_data_start + 1..end];
            final_file_name = filename.to_string();
        }
    }

    if final_file_name.is_empty() || encrypted_file_data_with_nonce.is_empty() {
        final_file_name = format!("secret_x_{}", file_number);
        encrypted_file_data_with_nonce = &bytes[start..end];
    }
    let (nonce, ciphered_file_data) =
        encryptor::separate_nonce_n_password(encrypted_file_data_with_nonce);
    match encryptor::decrypt_as_bytes(&nonce, &ciphered_file_data, &secret) {
        Ok(decrypted_file_data) => match fs::write(final_file_name.clone(), decrypted_file_data) {
            Ok(_) => Ok(final_file_name),
            Err(err) => Err(err),
        },
        Err(err) => Err(err),
    }
}

fn start_extracting_secret_files(
    bytes: &Vec<u8>,
    mut start_index: usize,
    file_separator_bytes: &[u8],
    password: &String,
) -> usize {
    let mut i = start_index;
    let length = bytes.len();
    let separator_length = file_separator_bytes.len();
    let mut separator_index = 0;
    let mut files_extracted = 0;
    let file_data_decrypt_secret = encryptor::string_to_fixed_array(&password);
    while i < length {
        if separator_index < separator_length {
            if &bytes[i] == &file_separator_bytes[separator_index] {
                separator_index += 1;
            } else {
                separator_index = 0;
            }
        } else {
            match extract_secret_file(
                &bytes,
                start_index,
                i - separator_length,
                files_extracted + 1,
                &file_data_decrypt_secret,
            ) {
                Err(err) => {
                    println!(
                        "Error extracting a file, skipping its data; Reason: {}",
                        err.to_string()
                    )
                }
                Ok(filename) => {
                    files_extracted += 1;
                    println!("{}. '{}' Extracted.", files_extracted, filename);
                }
            }
            separator_index = 0;
            start_index = i;
        }
        i += 1;
    }
    match extract_secret_file(
        &bytes,
        start_index,
        length,
        files_extracted + 1,
        &file_data_decrypt_secret,
    ) {
        Err(err) => {
            println!(
                "Error writing extracted data inside a new file; Reason: {}",
                err.to_string()
            )
        }
        Ok(filename) => {
            files_extracted += 1;
            println!("{}. '{}' Extracted.", files_extracted, filename);
        }
    };
    files_extracted
}

fn process_combined_file(
    filename: &str,
    file_separator_bytes: &[u8],
    password: &str,
    secret_key: &[u8; 32],
) -> Result<bool, String> {
    let separator_length = file_separator_bytes.len();
    match fs::read(filename) {
        Ok(bytes) => {
            let mut separator_index = 0;
            let mut i = 0;
            let bytes_length = bytes.len();
            let mut prefix_separator_index = 0;
            let mut hidden_data_start_index = 0;

            while i < bytes_length {
                if separator_index < separator_length {
                    if &bytes[i] == &file_separator_bytes[separator_index] {
                        separator_index += 1;
                    } else {
                        separator_index = 0;
                    }
                } else {
                    if hidden_data_start_index == 0 {
                        hidden_data_start_index = i;
                    }
                    if prefix_separator_index < separator_length {
                        if &bytes[i] == &file_separator_bytes[prefix_separator_index] {
                            prefix_separator_index += 1;
                        } else {
                            prefix_separator_index = 0;
                        }
                    } else {
                        let encrypted_nonce_n_pass =
                            &bytes[hidden_data_start_index..(i - separator_length)];

                        let (nonce, ciphered_password) =
                            encryptor::separate_nonce_n_password(encrypted_nonce_n_pass);
                        match encryptor::decrypt(&nonce, &ciphered_password, &secret_key) {
                            Ok(actual_password) => {
                                if password != actual_password.to_string() {
                                    return Ok(false);
                                }
                                if start_extracting_secret_files(
                                    &bytes,
                                    i,
                                    &file_separator_bytes,
                                    &actual_password,
                                ) == 0
                                {
                                    return Err(String::from(
                                        "Found some secret files but couldn't extract them.",
                                    ));
                                }
                            }
                            Err(_) => {
                                return Err(String::from(
                                    "Failed comparing passwords! Seems file data is curropted.",
                                ));
                            }
                        }
                        break;
                    }
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
    Ok(true)
}

fn main() {
    let file_separator_string = dotenv!("FILE_SEPARATOR_TEXT");
    if file_separator_string.is_empty() {
        panic!("FILE_SEPARATOR_TEXT is not set! It must be set and never change.");
    }
    let file_separator_bytes = file_separator_string.as_bytes();
    let secret_key = dotenv!("SECRET_KEY");
    if secret_key.is_empty() {
        panic!("SECRET_KEY is not set! It must be set and never change.");
    }
    let secret_key_bytes = encryptor::string_to_fixed_array(&secret_key);

    beep!();
    loop {
        let mut operation = String::new();
        try_reading_string(
            "- - - - - - - - - - - - - - - - - - - - - - - - - - - - -\nSELECT OPERATION:\n\t[H]IDE\n\t[B]ATCH HIDE\n\t[E]XTRACT\n\t[Q]UIT\n- - - - - - - - - - - - - - - - - - - - - - - - - - - - -\n",
                         &mut operation);

        match operation.trim() {
            "H" | "h" => {
                let mut image_path = String::new();
                let mut secret_file_path = String::new();
                let mut output_path = String::new();

                if !try_reading_string("Image Path: ", &mut image_path)
                    || !try_reading_string("Secret File Path: ", &mut secret_file_path)
                    || !try_reading_string("Output Path: ", &mut output_path)
                {
                    continue;
                }

                let password = get_your_password(0);
                if password.is_empty() {
                    continue;
                }

                match hide_secret_file(
                    &image_path.trim(),
                    &[secret_file_path.trim().to_string()],
                    &output_path.trim(),
                    &file_separator_bytes,
                    password,
                    &secret_key_bytes,
                    false,
                ) {
                    Ok(()) => {
                        beep!();
                        println!("Successfully hid your requested file inside the image.");
                    }
                    Err(err) => println!(
                        "FUCK! Failed to hide your requested file:\tReason:\n{}",
                        err
                    ),
                };
            }
            "B" | "b" => {
                let mut image_path = String::new();
                let mut secret_files_path: Vec<String> = Vec::new();
                let mut output_path = String::new();

                if !try_reading_string("Image Path: ", &mut image_path) {
                    continue;
                }

                let mut temp = String::new();
                let mut i = 1;
                println!("Secret File List: [Empty To End]");
                while try_reading_string(format!("#{}: ", i).as_str(), &mut temp) {
                    secret_files_path.push(temp.trim().to_string());
                    i += 1;
                }

                if !try_reading_string("Output Path: ", &mut output_path) {
                    continue;
                }

                let password = get_your_password(0);
                if password.is_empty() {
                    continue;
                }

                match hide_secret_file(
                    &image_path.trim(),
                    &secret_files_path,
                    &output_path.trim(),
                    &file_separator_bytes,
                    password,
                    &secret_key_bytes,
                    true, // TODO: Make this optional in the menu
                ) {
                    Ok(()) => {
                        beep!();
                        println!("Successfully hid your requested file inside the image.");
                    }
                    Err(err) => println!(
                        "FUCK! Failed to hide your requested file:\tReason:\n{}",
                        err
                    ),
                };
            }
            "E" | "e" => {
                let mut combined_file_path = String::new();
                let mut password = String::from("!");
                if !try_reading_string("Combined [By Me] File Path: ", &mut combined_file_path) {
                    continue;
                }

                let mut successfully_completed = false;
                let mut retry_count = 0;

                while !successfully_completed && !password.is_empty() && retry_count < 5 {
                    println!("Password: ");
                    match rpassword::read_password() {
                        Ok(_password) => {
                            password = _password;
                        }
                        Err(_) => {
                            password = String::new();
                        }
                    }
                    match process_combined_file(
                        combined_file_path.trim(),
                        &file_separator_bytes,
                        &password,
                        &secret_key_bytes,
                    ) {
                        Ok(password_matched) => {
                            successfully_completed = password_matched;
                        }
                        Err(err) => {
                            println!("FUCK! {}", err);
                        }
                    }
                    retry_count += 1;
                }

                if successfully_completed {
                    beep!();
                } else if password.is_empty() {
                    println!("Cancelled!");
                } else if retry_count >= 5 {
                    println!("FUCK U! Too many fuckin retries!");
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
