use std::convert::TryInto;

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
    username: String, // Rust use Unicode Scaler Value in Strings. but u8 is used. because char in C is a u8.
    email: String,
}

pub struct Table {
    num_rows: usize,
    pages: Vec<Vec<u8>>, // ideal implemention would be a fix sized Vec.
}

pub enum SyntaxErr {
    StringTooLong,
    WrongArgsNum,
    InvalidID,  // orignal error type in the blog is negtive id, which is will be catch by parsing from str to usize. 
}

impl Row {
    pub fn new(input: &str) -> Result<Self, SyntaxErr> {
        let mut iter = input.split_ascii_whitespace();
        iter.next(); // pop out "insert"

        let id = match iter.next() {
            Some(id_str) => match id_str.parse() {
                Ok(value) => value,
                Err(_) => return Err(SyntaxErr::InvalidID),
            },
            None => return Err(SyntaxErr::WrongArgsNum),
        };
        let username = match iter.next() {
            Some(name_str) => {
                if name_str.len() > USERNAME_SIZE {
                    return Err(SyntaxErr::StringTooLong)
                }
                name_str.to_string()
            }
            None => return Err(SyntaxErr::WrongArgsNum),
        };
        let email = match iter.next() {
            Some(mail_str) => {
                if mail_str.len() > EMAIL_SIZE {
                    return Err(SyntaxErr::StringTooLong)
                }
                mail_str.to_string()
            }
            None => return Err(SyntaxErr::WrongArgsNum),
        };

        return Ok(Row {
            id,
            username,
            email,
        });
    }

    pub fn serialize(&self, slot: &mut [u8]) {
        slot[ID_OFFSET..USERNAME_OFFSET].copy_from_slice(&self.id.to_le_bytes());
        string2arr(&self.username, &mut slot[USERNAME_OFFSET..EMAIL_OFFSET]);
        string2arr(&self.email, &mut slot[EMAIL_OFFSET..]);
    }

    pub fn deserialize(slot: &mut [u8]) -> Self {
        let id = u32::from_le_bytes(slot[ID_OFFSET..USERNAME_OFFSET].try_into().unwrap());
        let username = String::from_utf8(slot[USERNAME_OFFSET..EMAIL_OFFSET].to_owned()).unwrap();
        let email = String::from_utf8(slot[EMAIL_OFFSET..].to_owned()).unwrap();
        Row {id, username, email}
    }
}

fn string2arr(s: &String, arr: &mut [u8]) {
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

    pub fn row_slot(&mut self, row_num: usize) -> &mut [u8] {
        let page_num = row_num / ROWS_PER_PAGE;
        if let None = self.pages.get(page_num) {
            self.pages.push(vec![0; PAGE_SIZE]);
        }

        let offset: usize = row_num % ROWS_PER_PAGE;
        &mut self.pages[page_num][(offset * ROW_SIZE)..((offset + 1) * ROW_SIZE)]
    }

    pub fn is_full(&self) -> bool {
        self.num_rows >= TABLE_MAX_ROWS
    }

    pub fn insert(&mut self, row: Row) -> ExecuteResult {
        if self.is_full() {
            return ExecuteResult::ExecuteTableFull;
        }
        row.serialize(self.row_slot(self.num_rows));
        self.num_rows += 1;
        return ExecuteResult::ExecuteSuccess;
    }

    pub fn select(&mut self) {
        for i in 0..self.num_rows {
            let row_slot = self.row_slot(i);
            let row = Row::deserialize(row_slot);
            println!("({}, {}, {})", row.id, row.username, row.email);
        }
    }
}

pub enum ExecuteResult {
    ExecuteSuccess,
    ExecuteTableFull,
}
