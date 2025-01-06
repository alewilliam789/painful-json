use std::collections::HashMap;
use std::{default, env, i32};
use std::fs::{self, File};
use std::io::{self, BufReader, Read};
use std::fmt;
use std::mem;

use anyhow::{anyhow, Result};

#[derive(Default, Copy,Clone)]
pub enum Num {
    
    Uint(u32),
    Int(i32),
    Flt(f32),
    #[default]
    Empty,
}


#[derive(Default, Copy, Clone)]
pub struct Number {
    value : Num,
    digit : u32,
    is_negative : bool
}

pub enum ArrayMember {
    Str(String),
    Num(Number), 
    Bool(bool),
    JSON(JSON),
    Empty
}

#[derive(Default)]
pub enum JSONField {
    Str(String),
    Num(Number),  
    Bool(bool),
    Arr(Vec<ArrayMember>),
    JSON(JSON),
    #[default]
    Empty
}

pub struct JSON {
    pub map :  HashMap<String, JSONField>,
    current_object : bool,
    is_json : bool,
}

pub struct JSONPair {
    key: String,
    value: JSONField,
    current_key : bool, 
    current_value : bool,
}

impl std::fmt::Display for JSONField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JSONField::Bool(booly) => {
                write!(f,"{}",booly)
            }
            JSONField::Str(string) => {
                write!(f,"{}",string)
            }
            JSONField::Num(nummer) =>{
                match nummer.value {
                    Num::Uint(uint) => {
                        write!(f, "{}", uint)
                    }
                    Num::Int(int) => {
                        write!(f, "{}", int)
                    }
                    Num::Flt(flt) => {
                        write!(f,"{}", flt)
                    }
                    _ =>{
                        write!(f,"Empty number")
                    }
                }
            }
            _ => {
                write!(f,"Not quite done yet")
            }
        }
    }
}


pub fn get_reader() -> Result<BufReader<File>> {

    let path = env::current_dir()?;
    
    let args : Vec<String> = env::args().collect();

    let file_path = parse_args(&args);

    let reader = read_file(&file_path);

    reader
}


fn parse_args(input_args : &[String]) -> &str {

    let file_path = match input_args.get(1) {
        Some(file_path) => {
            file_path
        },
        None => {
            panic!("No JSON file path provided");
        }
    };

    match file_path.split('.').last() {
        Some(file_type) => {
            if file_type != "json" {
                panic!("This is not a valid JSON file");
            }
        }
        None => {
            panic!("There's an error in the file path provided");
        }
    };

    

    file_path
}

fn read_file(file_path : &str) -> Result<BufReader<File>>  {
    
    let file = fs::File::open(file_path)?;

    let reader = io::BufReader::new(file);

    Ok(reader)
}

pub fn parse_json<'a>(reader : &'a mut BufReader<File>) -> Result<JSON> {
   

    let mut json = JSON {
        map : HashMap::new(),
        current_object : false,
        is_json : false,
    };

    let mut json_pair = JSONPair {
            key : String::new(),
            value : JSONField::Empty,
            current_key : false,
            current_value : false,
    };

    let mut curr_u8 : [u8;1] = [65u8];

    loop {
        
        let _len = reader.read(&mut curr_u8)?;

        if _len == 0 {
            break
        }

        let char = curr_u8[0] as char;

        parse_character(char, &mut json_pair, &mut json)?;
    }

    match json_pair.value {
        JSONField::Empty => (),
        _ => {
            json.map.insert(json_pair.key, json_pair.value);
        }
    };

    
    if !json.is_json && json.map.is_empty() {
        return Err(anyhow!("No JSON found"));
    }

    Ok(json)
}

fn parse_character(current_char : char, json_pair : &mut JSONPair, json : &mut JSON) -> Result<()> {

    if !current_char.is_ascii_whitespace() && !json.current_object && current_char != '{' {
        return Err(anyhow!("No JSON object has been started"))
    }

    match current_char {
        '{' => {
            json.current_object = true;
        }
        '}' => {
            if json.current_object {
                json.current_object = false;
                json.is_json = true;
            }
            else {
                return Err(anyhow!("No JSON object to close")) 
            }
        }
        '"' => {
            if json_pair.current_value {
                checkset_string(json_pair);
            }
            else {
                check_key(json_pair);
            }
        }
        ':' => {
            if json_pair.key.len() > 0 {
                json_pair.current_value = true;
            }    
        }
        ',' => {
            json.map.insert(mem::replace(&mut json_pair.key, String::new()), mem::replace(&mut json_pair.value, JSONField::Empty));
        }
        '.' => {
            if json_pair.current_value {
                make_flt(json_pair)?;
            }
        }
        _ => {
            if current_char.is_ascii_whitespace() && !json_pair.current_value && !json_pair.current_key {
                return Ok(());
            }

            if json.current_object {

                if json_pair.current_value {
                    match json_pair.value {
                        JSONField::Empty => {
                            create_value(current_char, json_pair)?;
                        }
                        _ => {
                            fill_value(current_char, json_pair)?;
                        }
                    }
                }
                else if json_pair.current_key {
                    fill_key(current_char, json_pair)?;
                }
            }
        }
    }

    Ok(())
}

fn create_value(current_char : char, json_pair : &mut JSONPair) -> Result<()> {
    
    if current_char == 't' || current_char == 'f' {
        json_pair.value  = match current_char {
            't' => {
                json_pair.current_value = false;
                JSONField::Bool(true)
            }
            _ => {
                json_pair.current_value = false;
                JSONField::Bool(false)
            }
        };
    }
    else if current_char.is_digit(10) || current_char == '-' {

        let mut num = Number {
            value : Num::Uint(0),
            is_negative : false,
            digit: 0,
        };

        json_pair.value = match current_char {
            '-' => {
                let mut int = Number {
                    value : Num::Int(0),
                    is_negative : true,
                    digit: 0,
                };

                JSONField::Num(int)
            }
            _ => {
                num.value = Num::Uint(current_char.to_digit(10).expect("This needs to be a number"));
                num.digit += 1;
                JSONField::Num(num)
            }
        };
    } 

    Ok(())
}

fn fill_key(current_char : char, json_pair : &mut JSONPair) -> Result<()> {
    
    if json_pair.current_key {
        json_pair.key.push(current_char);
   }
    else {
        return Err(anyhow!("Problem with key"));
    }

    Ok(())
}

fn fill_value(current_char : char, json_pair : &mut JSONPair) -> Result<()> {


    let current_value = &mut json_pair.value;
    
    match current_value {
               JSONField::Str(str) => {
                    str.push(current_char);
                    json_pair.value = JSONField::Str(mem::take(str));
                }
               JSONField::Num(num) => {
                    
                   let base : i32 = 10;
                   
                   if current_char.is_ascii_whitespace() {
                        return Ok(());
                   }

                   match num.value {

                        Num::Uint(uint) => {

                            if num.digit == 0 {

                                num.value = Num::Uint(uint+current_char.to_digit(10).expect("This  should be a number"));
                                num.digit += 1;
                            }
                            else {
                                num.value = Num::Uint(uint*(base.pow(num.digit)as u32)  + current_char.to_digit(10).expect("This should be a number"));
                                num.digit += 1;
                            }
                        }
                        Num::Int(int) => {
                            if num.digit == 0 {
                                num.value = Num::Int(int+current_char.to_digit(10).expect("This  should be a number") as i32);
                                num.digit += 1;
                            }
                            else {
                                num.value = Num::Int(int*base.pow(num.digit) + current_char.to_digit(10).expect("This should be a number") as i32);
                                num.digit += 1;
                            }
                        }
                        Num::Flt(flt) => {
                            if num.digit == 0 {
                                num.value = Num::Flt(flt+(current_char.to_digit(10).expect("This  should be a number") as f32)/10.00);
                                num.digit += 1;
                            }
                            else {
                                num.value = Num::Flt(flt + (current_char.to_digit(10).expect("This should be a number") as f32)/(base.pow(num.digit) as f32));
                                num.digit += 1;
                            }
                        }
                        _ => ()
                    };

                   json_pair.value = JSONField::Num(mem::take(num));
               }
                _ => {
                   json_pair.value = JSONField::Empty;
                }
    };

    Ok(())
}

fn check_key(json_pair : &mut JSONPair) {
    
    if json_pair.current_key {
        json_pair.current_key = false;
    }
    else {
        json_pair.current_key = true;
    }
}

fn checkset_string (json_pair : &mut JSONPair) {

    let current_value = &json_pair.value;

    match current_value {
        JSONField::Empty => {
           json_pair.value = JSONField::Str(String::new());

        }
        JSONField::Str(str) => {
            if str.len() > 0 {
                json_pair.current_value = false;
            }
        }
        _ => ()
    };
}

fn make_flt(json_pair : &mut JSONPair) -> Result<()> {
    let field = &json_pair.value;

    json_pair.value = match *field {
        JSONField::Num(mut nummer) => {

            nummer.value = match nummer.value {
                Num::Uint(uint) => {

                    Num::Flt(uint as f32)
                }
                Num::Int(int) => {
                    Num::Flt(int as f32)
                }
                _=> {
                   Num::Flt(0.00)
                }
            };

            nummer.digit = 0;

            JSONField::Num(nummer)
        }
        _=> {
            mem::take(&mut json_pair.value)
        }
    };

    Ok(())
}



fn passed_file(file_path : &str) -> Result<BufReader<File>>{
    
    let reader = read_file(file_path)?;
    
    Ok(reader)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_file_path() -> () {
        let args = vec!["".to_string(), "file.json".to_string()];
        
        assert_eq!(parse_args(&args), "file.json");
    }


    #[test]
    #[should_panic]
    fn no_file_path() -> () {
        let args = vec!["".to_string()];

        parse_args(&args);
    }

    #[test]
    #[should_panic]
    fn no_file_type() -> () {
        let args = vec!["this.".to_string()];

        parse_args(&args);
    }

    #[test]
    fn no_json() -> Result<()> {
        let fake_file = "       ";

        let mut json = JSON {
            map : HashMap::new(),
            current_object : false,
            is_json : false,
        };

        let mut json_pair = JSONPair {
            key : String::new(),
            value : JSONField::Empty,
            current_key : false,
            current_value : false,
        };

        for char in fake_file.chars() {
            parse_character(char, &mut json_pair, &mut json)?;
        }

        assert!(!json.is_json);

        Ok(())
    }

    #[test]
    fn json() -> Result<()> {
        
        let file_path = "./json/empty.json"; 
        let mut reader = passed_file(file_path)?;

        let json = parse_json(&mut reader)?;

        assert!(json.is_json);
        Ok(())
    }

    #[test]
    fn bool_json() -> Result<()> {
        let file_path = "./json/bool.json";

        let path = env::current_dir()?;

        println!("The path is {}", path.display());
 
        let mut reader = passed_file(file_path)?;

        let json = parse_json(&mut reader)?;

        let field = json.map.get("bool").unwrap_or(&JSONField::Empty);

        let value = match *field {
            JSONField::Bool(bool)=>{
                bool
            }
            _=> { false}
        };

        assert!(value);

        Ok(())
    }

    #[test]
    fn uint_json() -> Result<()> {
        let file_path: &str = "./json/number.json";

        let mut reader: BufReader<File> = passed_file(file_path)?;

        let json: JSON = parse_json(&mut reader)?;

        let field: &JSONField = json.map.get("uinter").unwrap_or( &JSONField::Empty);

        let value :Num = match *field {
            JSONField::Num(num)=>{
                num.value
            }
            _ => {Num::Empty}
        };

        match value {
            Num::Uint(uint) => {
                assert!(uint == 22);
            }
            _ => {
                assert!(false)
            }
        };

        Ok(())
    }

    #[test]
    fn int_json() -> Result<()> {
        let file_path: &str = "./json/number.json";

        let mut reader: BufReader<File> = passed_file(file_path)?;

        let json: JSON = parse_json(&mut reader)?;

        let field: &JSONField = json.map.get("inter").unwrap_or( &JSONField::Empty);

        let value :Num = match *field {
            JSONField::Num(num)=>{
                num.value
            }
            _ => {Num::Empty}
        };

        match value {
            Num::Int(int) => {
                assert!(int == -22);
            }
            _ => {
                assert!(false)
            }
        };

        Ok(())
    }

    #[test]
    fn flt_json() -> Result<()> {
        let file_path: &str = "./json/number.json";

        let mut reader: BufReader<File> = passed_file(file_path)?;

        let json: JSON = parse_json(&mut reader)?;

        let field: &JSONField = json.map.get("floater").unwrap_or( &JSONField::Empty);

        let value :Num = match *field {
            JSONField::Num(num)=>{
                num.value
            }
            _ => {Num::Empty}
        };

        match value {
            Num::Flt(flt) => {
                assert!(flt == 22.1);
            }
            _ => {
                assert!(false)
            }
        };

        Ok(())
    }
}


