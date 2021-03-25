const ID_SIZE: usize = 4;
const USERNAME_SIZE: usize = 32;
const EMAIL_SIZE: usize = 255;
const ID_OFFSET: usize = 0;
const USERNAME_OFFSET: usize = ID_OFFSET + ID_SIZE;
const EMAIL_OFFSET: usize = USERNAME_OFFSET + USERNAME_SIZE;
const ROW_SIZE: usize = ID_SIZE + USERNAME_SIZE + EMAIL_SIZE;

const TABLE_MAX_PAGE: usize = 100;
const PAGE_SIZE: usize = 4019;
const ROWS_PER_PAGE: usize = PAGE_SIZE / ROW_SIZE;
const TABLE_MAX_ROWS: usize = TABLE_MAX_PAGE * ROWS_PER_PAGE;

#[derive(Debug)]
pub struct Row {
    id: u32,
    username: [u8; USERNAME_SIZE], // Rust use Unicode Scaler Value in Strings. but u8 is used. because char in C is a u8.
    email: [u8; EMAIL_SIZE],
}

struct Table {
    num_rows: usize,
    pages: Vec<Vec<u8>>, // ideal implemention would be a fix sized Vec.
}

impl Row {
    pub fn new(input: &str) -> Result<Self, ()> {
        let mut iter = input.split_ascii_whitespace();
        iter.next(); // pop out "insert"

        let id = match iter.next() {
            Some(id_str) => match id_str.parse() {
                Ok(value) => value,
                Err(_) => return Err(()),
            },
            None => return Err(()),
        };
        let mut username = [0; 32];
        match iter.next() {
            Some(name_str) => str2arr(name_str, &mut username),
            None => return Err(()),
        };
        let mut email = [0; 255];
        match iter.next() {
            Some(mail_str) => str2arr(mail_str, &mut email),
            None => return Err(()),
        };

        return Ok(Row {
            id,
            username,
            email,
        });
    }

    pub fn serialize(&self, slot: &mut [u8; ROW_SIZE]) {
        slot[ID_OFFSET..ID_SIZE].copy_from_slice(&self.id.to_le_bytes());
        slot[USERNAME_OFFSET..USERNAME_SIZE].copy_from_slice(&self.username);
        slot[EMAIL_OFFSET..EMAIL_SIZE].copy_from_slice(&self.email);
    }
}

fn str2arr(s: &str, arr: &mut [u8]) {
    s.chars()
        .zip(arr.iter_mut())
        .for_each(|(b, ptr)| *ptr = b as u8);
}

impl Table {
    pub fn new() -> Self {
        Table {
            num_rows: 0,
            pages: Vec::new(),
        }
    }

    pub fn row_slot(&mut self) -> &mut [u8] {
        let page_num = self.num_rows / ROWS_PER_PAGE;
        if let None = self.pages.get(page_num) {
            self.pages.push(Vec::with_capacity(PAGE_SIZE));
        }

        let offset: usize = self.num_rows % ROWS_PER_PAGE;
        &mut self.pages[page_num][offset..ROW_SIZE]
    }
}
