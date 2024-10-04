use std::error::Error;
use std::str::Split;
use std::path::PathBuf;

#[derive(Debug)]
#[allow(unused)]
struct UserEntry {
    user: String,
    pass: Option<String>,
    gid: u16,
    uid: u16,
    description: String,
    home_dir: PathBuf,
    shell: PathBuf,
}

fn build_entry(mut elems: Split<&str>) -> Option<UserEntry> {
    let user = elems.next()?.to_string();
    let pass = elems.next()?;
    let pass = match pass {
        "x" => None,
        _ => Some(pass.to_string()),
    };
    let gid = elems.next()?.parse::<u16>().ok()?;
    let uid = elems.next()?.parse::<u16>().ok()?;
    let description = elems.next()?.to_string();
    let home_dir = PathBuf::from(elems.next()?.to_string());
    let shell = PathBuf::from(elems.next()?.to_string());

    Some(UserEntry {
        user,
        pass,
        gid,
        uid,
        description,
        home_dir,
        shell,
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let contents = std::fs::read_to_string("/etc/passwd")?;
    
    let elems_iter = contents
        .lines()
        .map(|line| line.split(":"));
    let mut entry_collector = Vec::new();

    for elems in elems_iter {
        let entry = match build_entry(elems) {
            Some(entry) => entry,
            None => return Err("Failed to parse passwd entry".into()),
        };

        println!("Entry: {:?}", entry);

        entry_collector.push(entry);
    }

    println!("All users: {:?}", entry_collector.iter().map(|entry| entry.user.clone()).collect::<Vec<String>>().join(", "));

    Ok(())
}
