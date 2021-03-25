

#[derive(Debug)]
pub struct Row {
    id: i32,
    username: [u8; 32], // Rust use Unicode Scaler Value in Strings. but u8 is used. because char in C is a u8.
    email: [u8; 255]
}


impl Row {
    pub fn new(input: &str) -> Result<Self, ()> {
        let mut iter = input.split_ascii_whitespace();
        iter.next(); // pop out "insert"

        let id = match iter.next() {
            Some(id_str) => match id_str.parse() {
                Ok(value) => value,
                Err(_) => return Err(())
            },
            None => return Err(())
        };
        let mut username = [0; 32];
        match iter.next() {
            Some(name_str) => str2arr(name_str, &mut username),
            None => return Err(())
        };
        let mut email = [0; 255];
        match iter.next() {
            Some(mail_str) => str2arr(mail_str, &mut email),
            None => return Err(())
        };

        return Ok(Row{id, username, email})
    }
}

fn str2arr(s: &str, arr: &mut [u8]) {
    s.chars()
    .zip(arr.iter_mut())
    .for_each(|(b, ptr)| *ptr = b as u8);
} 
