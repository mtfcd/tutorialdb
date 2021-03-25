use std::{io::{BufRead, BufReader, BufWriter, Cursor, Read, Write}, process::{Command, Stdio}, time::Duration};
use std::thread;


#[test]
fn test_insert() {
    let mut cmd = Command::new("./target/debug/tutorialdb.exe")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = cmd.stdin.unwrap();
    let mut wrter = BufWriter::new(&mut stdin);


    let mut stdout = cmd.stdout.as_mut().unwrap();
    let mut prompt = vec![0; 10000];
    // let mut buf_reader = BufReader::new(stdout);
    for i in 1..2 {
        println!("round {}", i);
        (&mut stdout).read(&mut prompt).unwrap();
        println!("{}, {}", String::from_utf8(prompt.clone()).unwrap(), prompt.len());
        // prompt.clear();
        thread::sleep(Duration::from_secs(1));
        let username = format!("user_{}", i);
        let email = format!("user_{}@mail.com", i);
        let insert = format!("insert {} {} {}\n", i, username, email);
        println!("input1 {}", insert);
        wrter.write(insert.as_bytes()).unwrap();
        println!("input1 {}", insert);
    }
    thread::sleep(Duration::from_secs(1));
}