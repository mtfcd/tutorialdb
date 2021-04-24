use super::btree::*;

use std::{convert::TryInto, io::Read, io::Seek, io::{SeekFrom, Write}, usize};
use std::fs::File;
use std::fs::OpenOptions;
use std::process;


const ID_SIZE: usize = 4;
const USERNAME_SIZE: usize = 32;
const EMAIL_SIZE: usize = 255;
const ID_OFFSET: usize = 0;
const USERNAME_OFFSET: usize = ID_OFFSET + ID_SIZE;
const EMAIL_OFFSET: usize = USERNAME_OFFSET + USERNAME_SIZE;
pub const ROW_SIZE: usize = ID_SIZE + USERNAME_SIZE + EMAIL_SIZE;

const TABLE_MAX_PAGE: usize = 100;
pub const PAGE_SIZE: usize = 4096;

#[derive(Debug)]
pub struct Row {
    pub id: u32,
    username: String,
    email: String,
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

pub struct Table {
    root_page_num: usize,
    pager: Pager,
}

pub enum SyntaxErr {
    StringTooLong,
    WrongArgsNum,
    InvalidID,  // orignal error type in the blog is negtive id, which is will be catch by parsing from str to usize. 
}

impl Table {
    pub fn db_open(file_name: &str) -> Self {
        let mut pager = Pager::open(file_name);
        if pager.num_pages == 0 {
            let page = pager.get_page(0);
            initialize_leaf_node(page);
        }

        Table {
            root_page_num: 0,
            pager,
        }
    }

    pub fn db_close(&mut self) {
        for i in 0..self.pager.num_pages {
            if let Some(_) = self.pager.pages[i] {
                self.pager.flush(i);
            }
        }
    }

    pub fn find(&mut self, key: u32) -> Cursor {
        let root_page_num = self.root_page_num;
        let root_node = self.pager.get_page(root_page_num);
        match get_node_type(root_node) {
            NodeType::Leaf => Cursor::leaf_node_find(self, key),
            NodeType::Iternal => process::exit(2)
        }
    }
}

pub enum ExecuteResult {
    ExecuteSuccess,
    ExecuteTableFull,
    ExecuteDuplicateKey
}

struct Pager {
    pages: Vec<Option<Page>>, // use a Option here to check if a page is in memory.
    fd: File,
    file_length: usize,
    num_pages: usize
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

        if file_len % PAGE_SIZE != 0 {
            println!("Db file is not a whole number of pages. Corrupt file.");
            process::exit(1);
        }

        return Pager {
            pages: vec![None; TABLE_MAX_PAGE],
            fd: file,
            file_length: file_len,
            num_pages: file_len / PAGE_SIZE
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
            self.num_pages += 1;
        }
        self.pages[page_num].as_mut().unwrap()
    }

    fn flush(&mut self, page_num: usize) {
        match self.pages[page_num] {
            None => {
                println!("Tried to flush null page");
                process::exit(2);
            }
            Some(ref page) => {
                seek_file(&mut self.fd, page_num * PAGE_SIZE);
                if let Err(e) = self.fd.write(&page[0..PAGE_SIZE]) {
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

pub struct Cursor<'a> {
    table: &'a mut Table,
    page_num: usize,
    cell_num: u32,
    end_of_table: bool,
}

impl<'a> Cursor<'a> {
    pub fn table_start(table: &'a mut Table) -> Self {
        let root_page_num = table.root_page_num;
        let page = table.pager.get_page(root_page_num);
        let num_cells = get_leaf_node_num_cells(page);
        let end_of_table = num_cells == 0; 
        Cursor {
            table,
            page_num: root_page_num,
            cell_num: 0,
            end_of_table
        }
    }

    pub fn leaf_node_find(table: &'a mut Table, key: u32) -> Self {
        let mut min_index: u32 = 0;
        let page_num = table.root_page_num;
        let page = table.pager.get_page(page_num);
        let mut one_past_max_index: u32 = get_leaf_node_num_cells(page);
        while one_past_max_index != min_index {
            let index: u32 = (one_past_max_index + min_index) / 2;
            let key_at_index = get_leaf_node_key(page, index);
            if key == key_at_index {
                min_index = index;
                break;
            }
            if key_at_index < key {
                min_index = index + 1;
            } else {
                one_past_max_index = index;
            }
        }

        return Cursor {
            table,
            page_num,
            cell_num: min_index,
            end_of_table: false
        }
    }

    fn advance(&mut self) {
        self.cell_num += 1;
        
        let page_num = self.page_num;
        let page = self.table.pager.get_page(page_num);

        if self.cell_num >= get_leaf_node_num_cells(page) {
            self.end_of_table = true;
        }
    }

    fn value(&mut self) -> &mut [u8] {
        let page_num = self.page_num;
        let page = self.table.pager.get_page(page_num);

        leaf_node_value(page, self.cell_num)
    }

    pub fn insert(&mut self, row: Row) -> ExecuteResult {
        let page = self.table.pager.get_page(self.page_num);
        let num_cells = get_leaf_node_num_cells(page);

        if num_cells as usize >= LEAF_NODE_MAX_CELLS {
            return ExecuteResult::ExecuteTableFull;
        }
        if row.id == get_leaf_node_key(page, self.cell_num) {
            return ExecuteResult::ExecuteDuplicateKey
        }

        if self.cell_num < num_cells {
            make_room(page, self.cell_num);
        }

        set_leaf_node_num_cells(page, num_cells + 1);
        set_leaf_node_key(page, self.cell_num, row.id);
        row.serialize(self.value());
        return ExecuteResult::ExecuteSuccess;
    }

    pub fn select(&mut self) {
        while !self.end_of_table {
            let row_slot = self.value();
            let row = Row::deserialize(row_slot);
            println!("({}, {}, {})", row.id, row.username, row.email);
            self.advance();
        }
    }
}