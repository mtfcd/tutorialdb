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
    for i in 1..3 {
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

fn read_child_output(stdout: &mut ChildStdout) {
    let mut prompt = [0u8; 100];
    loop {
        let read_len = stdout.read(&mut prompt).unwrap();
        print!("{}", String::from_utf8(Vec::from(&prompt as &[u8])).unwrap());
        if read_len == 0 || prompt[read_len - 1] == b'>' {
            break;
        }
    }
}