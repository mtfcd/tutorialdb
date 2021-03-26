use std::{io::{Read, Write}, process::{ChildStdout, ChildStdin, Command, Stdio}};


#[test]
fn test_insert() {
    let cmd = Command::new("./target/debug/tutorialdb.exe")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = cmd.stdin.unwrap();
    let mut stdout = cmd.stdout.unwrap();
    for i in 1..3 { // table will full at 1400.
        println!("round {}", i);
        read_child_output(&mut stdout);
        let username = format!("user_{}", i);
        let email = format!("user_{}@mail.com", i);
        let insert = format!("insert {} {} {}\n", i, username, email);
        write_child_stdin(&mut stdin, insert);
    }
    read_child_output(&mut stdout);
    write_child_stdin(&mut stdin, "select\n".into());
    read_child_output(&mut stdout);
    write_child_stdin(&mut stdin, ".exit\n".into());
}

fn write_child_stdin(stdin: &mut ChildStdin, input: String) {
    stdin.write(input.as_bytes()).unwrap();
    println!("{}", input);
}

const PROMPT: &str = "db > "; // this has to be the same with prompt in main.
const PT_LEN: usize = PROMPT.len();

fn read_child_output(stdout: &mut ChildStdout) {
    let mut prev_buf = [0u8; 10];
    let mut buf = [0u8; 10];
    let mut prompt = [0u8; PT_LEN];
    loop {
        let read_len = stdout.read(&mut buf).unwrap();
        print!("{}", String::from_utf8(Vec::from(buf)).unwrap());

        if read_len == 0 {
            break; 
        }
        if read_len >= PT_LEN {
            prompt.copy_from_slice(&buf[read_len-PT_LEN..read_len]);
        } else {
            prompt[..PT_LEN-read_len].copy_from_slice(&prev_buf[(10 - PT_LEN) + read_len..]);
            prompt[PT_LEN-read_len..].copy_from_slice(&buf[..read_len]);
        }
        if String::from_utf8(Vec::from(prompt)).unwrap() == PROMPT {
            break; 
        }
        prev_buf.copy_from_slice(&buf);
    }
}