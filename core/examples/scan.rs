use std::error::Error;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let [_, db, root] = args.as_slice() else {
        return Err("usage: scan <db> <root>".into());
    };
    let mut library = quokka_core::Library::open(Path::new(db))?;
    let stats = library.scan(Path::new(root))?;
    println!("{stats:?}");
    println!("items in library: {}", library.count()?);
    Ok(())
}
