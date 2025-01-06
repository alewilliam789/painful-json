use std::env;

use anyhow::{self, Result};

use painful_json::*;



fn main() -> Result<()> {
   
    let mut reader = get_reader()?;


    let json = parse_json(&mut reader)?;

    let field = json.map.get("floater").unwrap_or(&JSONField::Empty);

    println!("{}",field);



    Ok(())
}



