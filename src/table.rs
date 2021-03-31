use std::{convert::TryInto, io::Read, io::Seek, io::{SeekFrom, Write}};
use std::fs::File;
use std::fs::OpenOptions;
use std::process;


const ID_SIZE: usize = 4;
const USERNAME_SIZE: usize = 32;
const EMAIL_SIZE: usize = 255;
const ID_OFFSET: usize = 0;
const USERNAME_OFFSET: usize = ID_OFFSET + ID_SIZE;
const EMAIL_OFFSET: usize = USERNAME_OFFSET + USERNAME_SIZE;
const ROW_SIZE: usize = ID_SIZE + USERNAME_SIZE + EMAIL_SIZE;

const TABLE_MAX_PAGE: usize = 100;
const PAGE_SIZE: usize = 4096;
const ROWS_PER_PAGE: usize = PAGE_SIZE / ROW_SIZE;
const TABLE_MAX_ROWS: usize = TABLE_MAX_PAGE * ROWS_PER_PAGE;

#[derive(Debug)]
pub struct Row {
    id: u32,
    username: String,
    email: String,
}

pub struct Table {
    num_rows: usize,
    pager: Pager,
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
    pub fn db_open(file_name: &str) -> Self {
        let pager = Pager::open(file_name);
        Table {
            num_rows: pager.file_length / ROW_SIZE,
            pager,
        }
    }

    pub fn row_slot(&mut self, row_num: usize) -> &mut [u8] {
        let page_num = row_num / ROWS_PER_PAGE;
        let page = self.pager.get_page(page_num);

        let offset: usize = row_num % ROWS_PER_PAGE;
        &mut page[(offset * ROW_SIZE)..((offset + 1) * ROW_SIZE)]
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

    pub fn db_close(&mut self) {
        let num_full_pages = self.num_rows / ROWS_PER_PAGE;
        for i in 0..num_full_pages {
            if let Some(_) = self.pager.pages[i] {
                self.pager.flush(i, PAGE_SIZE);
            }
        }
        let num_additional_rows = self.num_rows % ROWS_PER_PAGE;
        if num_additional_rows > 0 {
            if let Some(_) = self.pager.pages[num_full_pages] {
                self.pager.flush(num_full_pages, num_additional_rows * ROW_SIZE);
            }
        }
    }
}

pub enum ExecuteResult {
    ExecuteSuccess,
    ExecuteTableFull,
}

type Page = Vec<u8>; // Rust use Unicode Scaler Value in Strings. but u8 is used. because char in C is a u8.

struct Pager {
    pages: Vec<Option<Page>>, // use a Option here to check if a page is in memory.
    fd: File,
    file_length: usize,
}

impl Pager {
    fn open(file_name: &str) -> Self {
        let file = match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(file_name) {
                Ok(file) => file,
                Err(_) => process::exit(1)
            };
        
        let file_len: usize;
        match file.metadata() {
            Ok(meta) => file_len = meta.len() as usize,
            Err(_) => process::exit(1),
        }

        return Pager {
            pages: vec![None; TABLE_MAX_PAGE],
            fd: file,
            file_length: file_len,
        }
    }

    fn get_page<'a>(&'a mut self, page_num: usize) -> &'a mut Page {
        if page_num > TABLE_MAX_PAGE {
            println!("Tried to fetch page number out of bounds. {} > {}", page_num, TABLE_MAX_PAGE);
            process::exit(2);
        }
        
        // inorder to check if a page is exists and return a &mut
        // it has to make a immutable borrow for check and then a mut borrow for return.
        // or it will hava a problem https://rust-lang.github.io/rfcs/2094-nll.html#problem-case-3-conditional-control-flow-across-functions
        let page_opt = &self.pages[page_num]; 
        if page_opt.is_none() {
            let mut new_page = vec![0; PAGE_SIZE];
            let mut num_pages = self.file_length / PAGE_SIZE;
            if self.file_length % PAGE_SIZE > 0 {
                num_pages += 1;
            }
            if page_num < num_pages {
                file_read(&mut self.fd, page_num * PAGE_SIZE, &mut new_page);
            }
            self.pages[page_num] = Some(new_page);
        }
        self.pages[page_num].as_mut().unwrap()
    }

    fn flush(&mut self, page_num: usize, size: usize) {
        match self.pages[page_num] {
            None => {
                println!("Tried to flush null page");
                process::exit(2);
            }
            Some(ref page) => {
                seek_file(&mut self.fd, page_num * PAGE_SIZE);
                if let Err(e) = self.fd.write(&page[0..size]) {
                    println!("Error writing: {}", e);
                    process::exit(2);
                }
            }
        }

    }
}

fn seek_file(file: &mut File, pos: usize) {
    match file.seek(SeekFrom::Start(pos as u64)) {
        Ok(_) => {},
        Err(e) => {
            println!("Error reading file {}", e);
            process::exit(2)
        }
    };
}

fn file_read(file: &mut File, pos: usize, buf: &mut Vec<u8>) {
    seek_file(file, pos);
    match file.read(buf) {
        Ok(_) => {},
        Err(e) => {
            println!("Error reading file {}", e);
            process::exit(2)
        }
    }
}