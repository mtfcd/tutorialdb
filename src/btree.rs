use std::convert::TryInto;

use super::table::*;


const NODE_TYPE_SIZE: usize = 1;
const NODE_TYPE_OFFSET: usize = 0;
const IS_ROOT_SIZE: usize = 1;
const IS_ROOT_OFFSET: usize = NODE_TYPE_SIZE;
const PARENT_POINTER_SIZE: usize = 4;
const PARENT_POINTER_OFFSET: usize = IS_ROOT_OFFSET + IS_ROOT_SIZE;
const COMMON_NODE_HEADER_SIZE: usize = NODE_TYPE_SIZE + IS_ROOT_SIZE + PARENT_POINTER_SIZE;

const LEAF_NODE_NUM_CELLS_SIZE: usize = 4;
const LEAF_NODE_NUM_CELLS_OFFSET: usize = COMMON_NODE_HEADER_SIZE;
const LEAF_NODE_HEADER_SIZE: usize = COMMON_NODE_HEADER_SIZE + LEAF_NODE_NUM_CELLS_SIZE;

const LEAF_NODE_KEY_SIZE: usize = 4;
const LEAF_NODE_KEY_OFFSET: usize = 0;
const LEAF_NODE_VALUE_SIZE: usize = ROW_SIZE;
const LEAF_NODE_VALUE_OFFSET: usize = LEAF_NODE_KEY_OFFSET + LEAF_NODE_KEY_SIZE;
const LEAF_NODE_CELL_SIZE: usize = LEAF_NODE_KEY_SIZE + LEAF_NODE_VALUE_SIZE;
const LEAF_NODE_SPACE_FOR_CELLS : usize = PAGE_SIZE - LEAF_NODE_HEADER_SIZE;
pub const LEAF_NODE_MAX_CELLS : usize = LEAF_NODE_SPACE_FOR_CELLS / LEAF_NODE_CELL_SIZE;

pub type Page = Vec<u8>; // Rust use Unicode Scaler Value in Strings. but u8 is used. because char in C is a u8.

pub fn get_leaf_node_num_cells(page: &Page) -> u32 {
    let idx_1 = LEAF_NODE_NUM_CELLS_OFFSET;
    let idx_2 = idx_1 + LEAF_NODE_NUM_CELLS_SIZE;
    u32::from_le_bytes(page[idx_1..idx_2].try_into().unwrap())
}
pub fn set_leaf_node_num_cells(page: &mut Page, cell_num: u32) {
    let idx_1 = LEAF_NODE_NUM_CELLS_OFFSET;
    let idx_2 = idx_1 + LEAF_NODE_NUM_CELLS_SIZE;

    page[idx_1..idx_2].copy_from_slice(&cell_num.to_le_bytes());
}

pub fn get_leaf_node_key(page: &Page, cell_num: u32) -> u32 {
    let idx_1 = leaf_node_offset(cell_num) + LEAF_NODE_KEY_OFFSET;
    let idx_2 = idx_1 + LEAF_NODE_KEY_SIZE;

    u32::from_le_bytes(page[idx_1..idx_2].try_into().unwrap())
}
pub fn set_leaf_node_key(page: &mut Page, cell_num: u32, key: u32) {
    let idx_1 = leaf_node_offset(cell_num) + LEAF_NODE_KEY_OFFSET;
    let idx_2 = idx_1 + LEAF_NODE_KEY_SIZE;

    page[idx_1..idx_2].copy_from_slice(&key.to_le_bytes());
}
// pub fn leaf_node_cell(page: &Page, cell_num: u32) -> u32 {
//     let idx_1 = LEAF_NODE_HEADER_SIZE + cell_num as usize * LEAF_NODE_CELL_SIZE;
//     let idx_2 = idx_1 + LEAF_NODE_CELL_SIZE;
//     u32::from_le_bytes(page[idx_1..idx_2].try_into().unwrap())
// }

fn leaf_node_offset(cell_num: u32) -> usize {
    LEAF_NODE_HEADER_SIZE + cell_num as usize * LEAF_NODE_CELL_SIZE
}

pub fn leaf_node_value(page: &mut Page, cell_num: u32) -> &mut [u8] {
    let idx_1 = leaf_node_offset(cell_num) + LEAF_NODE_VALUE_OFFSET;
    let idx_2 = idx_1 + LEAF_NODE_VALUE_SIZE;
    return &mut page[idx_1..idx_2]
}

pub fn initialize_leaf_node(page: &mut Page) {
    page.fill(0);
    set_node_type(page, NodeType::Leaf);
}

pub fn make_room(page: &mut Page, cell_num: u32) {
    let num_cell = get_leaf_node_num_cells(page);
    let idx_1 = leaf_node_offset(cell_num);
    let idx_2 = leaf_node_offset(num_cell + 1);
    page[idx_1..idx_2].rotate_right(LEAF_NODE_CELL_SIZE);
}

pub fn print_constants() {
    println!("ROW_SIZE: {}", ROW_SIZE);
    println!("COMMON_NODE_HEADER_SIZE: {}", COMMON_NODE_HEADER_SIZE);
    println!("LEAF_NODE_HEADER_SIZE: {}", LEAF_NODE_HEADER_SIZE);
    println!("LEAF_NODE_CELL_SIZE: {}", LEAF_NODE_CELL_SIZE);
    println!("LEAF_NODE_SPACE_FOR_CELLS: {}", LEAF_NODE_SPACE_FOR_CELLS);
    println!("LEAF_NODE_MAX_CELLS: {}", LEAF_NODE_MAX_CELLS);
}

pub enum NodeType {
    Iternal = 1,
    Leaf = 2
}
impl NodeType {
    fn from_u8(num: u8) -> Self {
        match num {
            1 => NodeType::Iternal,
            2  => NodeType::Leaf,
            _ => panic!("unknown node type")
        }
    }

    fn to_u8(&self) -> u8 {
        match self {
            Self::Iternal => 1,
            Self::Leaf => 2
        }
    }
}
pub fn get_node_type(page: &Page) -> NodeType {
    let idx_1 = NODE_TYPE_OFFSET;
    let idx_2 = idx_1 + NODE_TYPE_SIZE;
    let type_value = u8::from_le_bytes(page[idx_1..idx_2].try_into().unwrap());
    NodeType::from_u8(type_value)
}
pub fn set_node_type(page: &mut Page, node_type: NodeType) {
    let idx_1 = NODE_TYPE_OFFSET;
    let idx_2 = idx_1 + NODE_TYPE_SIZE;
    page[idx_1..idx_2].copy_from_slice(&node_type.to_u8().to_le_bytes());
}