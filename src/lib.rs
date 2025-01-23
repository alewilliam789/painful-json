use std::collections::HashMap;
use std::{env, i32};
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

#[derive(Default, Clone)]
pub struct Booler {
    value : bool,
    current: String,
    correct: String,
    index : usize,
}

#[derive(Default)]
pub enum Member {
    Str(String),
    Num(Number),  
    Bool(Booler),
    Arr(Vec<Member>),
    JSON(JSON),
    #[default]
    Empty
}

pub struct JSON {
    pub map :  HashMap<String, Member>,
    current_object : bool,
    is_json : bool,
}
pub struct Array {
    value : Vec<Member>,
    data : Member,
    is_array : bool,
    is_escaped : bool
}

pub struct JSONPair {
    key: String,
    value : Member,
    current_key : bool, 
    current_value : bool,
    is_escaped: bool
}

impl std::fmt::Display for Member {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Member::Bool(booly) => {
                write!(f,"{}",booly.value)
            }
            Member::Str(string) => {
                write!(f,"{}",string)
            }
            Member::Num(nummer) =>{
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
            Member::Arr(_) => {

                write!(f,"[ Might be something in here ]")
            }
            _ => {
                write!(f,"Not quite done yet")
            }
        }
    }
}


pub fn get_reader() -> Result<BufReader<File>> {
    
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
            value : Member::Empty,
            current_key : false,
            current_value : false,
            is_escaped : false
    };

    let mut arr  = Array {
        value : Vec::new(),
        data : Member::Empty,
        is_array : false,
        is_escaped : false
    };

    let mut curr_u8 : [u8;1] = [65u8];

    loop {
        
        let _len = reader.read(&mut curr_u8)?;

        if _len == 0 {
            break
        }

        let char = curr_u8[0] as char;

        parse_character(char, &mut json_pair, &mut json, &mut arr)?;
    }


    json.map.insert(json_pair.key, json_pair.value);

    
    if !json.is_json && json.map.is_empty() {
        return Err(anyhow!("No JSON found"));
    }

    Ok(json)
}

fn parse_character(current_char : char, json_pair : &mut JSONPair, json : &mut JSON, array : &mut Array) -> Result<()> {

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
        '[' => {
            if json_pair.current_value {
                array.is_array = true;
            }
        }
        ']' => {
            if json_pair.current_value {
                array.is_array = false;
                json_pair.current_value = false;

                match array.data {
                    Member::Empty => (),
                    _ => {
                        array.value.push(mem::take(&mut array.data));
                    }
                }

                json_pair.value = Member::Arr(mem::take(&mut array.value));
            }
        }
        '"' => {
            if array.is_array {
                match array.data {
                    Member::Empty => {
                        (array.data, array.is_escaped) = checkset_string(&mut array.data, array.is_escaped, current_char)?;
                    }
                    Member::Str(_)=>{
                        if array.is_escaped {
                            (array.data, array.is_escaped) = checkset_string(&mut array.data, array.is_escaped, current_char)?;
                        }
                    }
                    _ =>()
                }
            }
            else if json_pair.current_value {
                match json_pair.value {
                    Member::Empty => {
                        (json_pair.value, json_pair.is_escaped) = checkset_string(&mut json_pair.value, json_pair.is_escaped, current_char)?;
                    }
                    Member::Str(_)=>{
                        if json_pair.is_escaped {
                            (json_pair.value, json_pair.is_escaped) = checkset_string(&mut json_pair.value, json_pair.is_escaped, current_char)?;
                        }
                        else {
                            json_pair.current_value = false;
                        }
                    }
                    _ =>()
                }
            }
            else {
                check_key(json_pair)?;
            }
        }
        ':' => {
            if json_pair.key.len() > 0 {
                json_pair.current_value = true;
            }    
        }
        ',' => {
            if array.is_array {
                array.value.push(mem::replace(&mut array.data,Member::Empty));
            }
            else {
                json.map.insert(mem::replace(&mut json_pair.key, String::new()), mem::replace(&mut json_pair.value, Member::Empty));
            
                json_pair.current_value = false;
            };
        }
        '.' => {
            if array.is_array {
               array.data = make_flt(&mut array.data)?;
            }
            else if json_pair.current_value {
                json_pair.value =  make_flt(&mut json_pair.value)?;
            }
        }
        _ => {
            if current_char.is_ascii_whitespace() && ((!json_pair.current_value && !json_pair.current_key)) {
                return Ok(());
            }

            if json.current_object {

                if json_pair.current_value  || array.is_array {

                    let mut current_mem : Member;

                    if array.is_array {
                        current_mem = mem::take(&mut array.data);
                    }
                    else {
                        current_mem = mem::take(&mut json_pair.value);
                    }

                    match current_mem {
                        Member::Empty => {
                            if array.is_array {
                               array.data = create_value(current_char)?;
                            }
                            else {
                                json_pair.value = create_value(current_char)?;
                            }
                        }
                        _ => {
                            if array.is_array {
                                (array.data, array.is_escaped) = fill_value(current_char, &mut current_mem, array.is_escaped)?;
                            }
                            else {
                                (json_pair.value, json_pair.is_escaped) = fill_value(current_char, &mut current_mem, json_pair.is_escaped)?;
                            }
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

fn create_value(current_char : char) -> Result<Member> {

    let mut new_data : Member = Member::Empty;
    
    if current_char == 't' || current_char == 'f' {
        new_data = match current_char {
            't' => {
                let truer = Booler {
                    value: true,
                    current : String::from("t"),
                    correct : String::from("true"),
                    index : 1
                };
                Member::Bool(truer)
            }
            _ => {
                let falser = Booler {
                    value : false,
                    current : String::from("f"),
                    correct : String::from("false"),
                    index : 1
                };

                Member::Bool(falser)
            }
        };
    }
    else if current_char.is_digit(10) || current_char == '-' {

        let mut num = Number {
            value : Num::Uint(0),
            is_negative : false,
            digit: 0,
        };

        new_data = match current_char {
            '-' => {

                let int = Number {
                    value : Num::Int(0),
                    is_negative : true,
                    digit: 0,
                };

                Member::Num(int)
            }
            _ => {
                num.value = Num::Uint(current_char.to_digit(10).expect("This needs to be a number"));
                num.digit += 1;
                Member::Num(num)
            }
        };
    } 

    Ok(new_data)
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

fn fill_value(current_char : char, data : &mut Member, escaped : bool) -> Result<(Member,bool)> {
    let is_escaped = escaped;

    match data {
               Member::Str(str) => {

                    let (new_string,is_escaped ) = append_character(current_char, str, escaped)?;
                    return Ok((Member::Str(new_string),is_escaped));
                }
               Member::Num(num) => {
                    if current_char.is_ascii_whitespace() {
                        return Ok((Member::Num(mem::take(num)),is_escaped));
                    }
                    
                   let base : i32 = 10;

                   let sign : i32 = if num.is_negative {-1} else {1};

                   match num.value {
                        Num::Uint(uint) => {
                            num.value = Num::Uint(uint*(base.pow(num.digit)as u32)  + current_char.to_digit(10).expect("This should be a number"));
                            num.digit += 1;
                        }
                        Num::Int(int) => {
                            if num.digit == 0 {
                                num.value = Num::Int(int+(current_char.to_digit(10).expect("This  should be a number") as i32)*sign);
                                num.digit += 1;
                            }
                            else {
                                num.value = Num::Int(int*base.pow(num.digit) + (current_char.to_digit(10).expect("This should be a number") as i32)*sign);
                                num.digit += 1;
                            }
                        }
                        Num::Flt(flt) => {
                            if num.digit == 0 {
                                num.value = Num::Flt(flt+((current_char.to_digit(10).expect("This  should be a number") as f32)/10.00)*sign as f32);
                                num.digit += 1;
                            }
                            else {
                                num.value = Num::Flt(flt + ((current_char.to_digit(10).expect("This should be a number") as f32)/(base.pow(num.digit) as f32))*sign as f32);
                                num.digit += 1;
                            }
                        }
                        _ => ()
                    };

                   return Ok((Member::Num(mem::take(num)),is_escaped));
               }
               Member::Bool(booler) =>{

                    if current_char.is_ascii_whitespace() {
                        return Ok((Member::Bool(mem::take(booler)),is_escaped));
                    }

                    let correct_char = booler.correct.as_bytes()[booler.index] as char;

                    if current_char == correct_char && !current_char.is_ascii_whitespace() {
                        booler.current.push(current_char);
                        if booler.index < booler.correct.len()-1 {
                            booler.index += 1;
                        }
                    }
                    else {
                        return Err(anyhow!("Incorrectly spelled boolean value"));
                    }

                    return Ok((Member::Bool(mem::take(booler)),is_escaped));
               }

                _ => {
                   return Ok((Member::Empty,is_escaped));
                }
    };
}

fn check_key(json_pair : &mut JSONPair) -> Result<()>{
    
    if json_pair.current_key {
        json_pair.current_key = false;
    }
    else {
        json_pair.current_key = true;
    }

    Ok(())
}

fn checkset_string (data : &mut Member, is_escaped: bool, current_char : char) -> Result<(Member,bool)>{
    let new_data : Member;
    let mut new_escaped : bool = is_escaped; 

    new_data = match data {
        Member::Str(str) => {
            let (new_string, escaped) = append_character(current_char, &str,is_escaped)?;
            new_escaped = escaped;

            Member::Str(new_string)
        }
        _ => {
            Member::Str(String::new())
        }
    };

    Ok((new_data,new_escaped))
}

fn make_flt(data : &mut Member) -> Result<Member> {

    match *data{
        Member::Num(mut nummer) => {

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

            return Ok(Member::Num(nummer));
        }
        _=> {
            return Ok(mem::take(data));
        }
    };
}

fn append_character(current_char : char, stringer : &String, mut is_escaped : bool) -> Result<(String,bool)>{

    let mut new_string = stringer.to_string();

    if current_char.is_alphanumeric() && !is_escaped {
        new_string.push(current_char);
    }
    else if current_char == '\\' && !is_escaped {
        is_escaped = true;
    }
    else {
        match current_char {
            '\\' => {
                new_string.push('\\');
            }
            '"' => {
                new_string.push('"');
            }
            'n' => {
                new_string.push('\n');
            }
            't' => {
                new_string.push('\t');
            }
            'r' => {
                new_string.push('\r');
            }
            _=> {
                new_string.push(current_char);
            }
        }

        is_escaped = false;
    }
    
    return Ok((new_string,is_escaped))


}


#[cfg(test)]
mod tests {
    use super::*;

    fn passed_file(file_path : &str) -> Result<BufReader<File>>{
    
        let reader = read_file(file_path)?;
        
        Ok(reader)
    }

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
            value : Member::Empty,
            current_key : false,
            current_value : false,
            is_escaped : false
        };

        let mut array = Array {
            value : Vec::new(),
            data : Member::Empty,
            is_array : false,
            is_escaped : false
        };

        for char in fake_file.chars() {
            parse_character(char, &mut json_pair, &mut json,&mut array)?;
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
 
        let mut reader = passed_file(file_path)?;

        let json = parse_json(&mut reader)?;

        let field = json.map.get("bool").unwrap_or(&Member::Empty);

       match field {
            Member::Bool(bool)=>{
                assert!(bool.value);
            }
            _=> { panic!("Not a boolean");}
        };

        Ok(())
    }

    #[test]
    fn uint_json() -> Result<()> {
        let file_path: &str = "./json/number.json";

        let mut reader: BufReader<File> = passed_file(file_path)?;

        let json: JSON = parse_json(&mut reader)?;

        let field: &Member = json.map.get("uinter").unwrap_or( &Member::Empty);

        let value :Num = match *field {
            Member::Num(num)=>{
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

        let field: &Member = json.map.get("inter").unwrap_or( &Member::Empty);

        let value :Num = match *field {
            Member::Num(num)=>{
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

        let field: &Member = json.map.get("floater").unwrap_or( &Member::Empty);

        let value :Num = match *field {
            Member::Num(num)=>{
                num.value
            }
            _ => {Num::Empty}
        };

        match value {
            Num::Flt(flt) => {
                assert!(flt == -22.1);
            }
            _ => {
                assert!(false)
            }
        };

        Ok(())
    }

    #[test]
    fn string_json() -> Result<()> {
        let file_path: &str = "./json/string.json";

        let mut reader: BufReader<File> = passed_file(file_path)?;

        let json: JSON = parse_json(&mut reader)?;

        let field: &Member = json.map.get("stringer").unwrap_or( &Member::Empty);

        let value : String = match field {
            Member::Str(str)=>{
                str.to_string()
            }
            _ => {String::from("")}
        };

        assert!(value == "hello");

        Ok(())
    }

    #[test]
    fn escaped_quote_json() -> Result<()> {
        let file_path: &str = "./json/escaped_string.json";

        let mut reader: BufReader<File> = passed_file(file_path)?;

        let json: JSON = parse_json(&mut reader)?;

        let field: &Member = json.map.get("stringer").unwrap_or( &Member::Empty);

        let value : String = match field {
            Member::Str(str)=>{
                str.to_string()
            }
            _ => {String::from("")}
        };

        assert!(value == "hello\"");

        Ok(())
    }

    #[test]
    fn escaped_slash_json() -> Result<()> {
        let file_path: &str = "./json/escaped_string.json";

        let mut reader: BufReader<File> = passed_file(file_path)?;

        let json: JSON = parse_json(&mut reader)?;

        let field: &Member = json.map.get("slasher").unwrap_or( &Member::Empty);

        let value : String = match field {
            Member::Str(str)=>{
                str.to_string()
            }
            _ => {String::from("")}
        };

        assert!(value == "hello\\");

        Ok(())
    }

    #[test]
    fn empty_array_json() -> Result<()> {
        let file_path: &str = "./json/empty_array.json";

        let mut reader: BufReader<File> = passed_file(file_path)?;

        let json: JSON = parse_json(&mut reader)?;

        let field: &Member = json.map.get("arr").unwrap_or( &Member::Empty);

        match field {
            Member::Arr(arr)=>{
                assert!(arr.len() == 0);
            }
            _=>{
               return Err(anyhow!("No array found")); 
            }
        };

        Ok(())
    }

    #[test]
    fn array_json() -> Result<()> {
        let file_path: &str = "./json/array.json";
        let mut reader: BufReader<File> = passed_file(file_path)?;

        let json: JSON = parse_json(&mut reader)?;

        let field: &Member = json.map.get("arr").unwrap_or( &Member::Empty);

        match field {
            Member::Arr(arr)=>{
                let uint = arr.get(0).unwrap_or(&Member::Empty);
                match uint {
                    Member::Num(num)=>{
                        match num.value {
                            Num::Uint(uint)=>{
                                assert!(uint == 1);
                            }
                            _=>()
                        }
                    }
                    _ =>()
                };

                let int = arr.get(1).unwrap_or(&Member::Empty);
                match int {
                    Member::Num(num)=>{
                        match num.value {
                            Num::Int(int)=>{
                                assert!(int == -1);
                            }
                            _=>()
                        }
                    }
                    _ =>()
                };

                let flt = arr.get(2).unwrap_or(&Member::Empty);
                match flt {
                    Member::Num(num)=>{
                        match num.value {
                            Num::Flt(fltr)=>{
                                assert!(fltr == 22.0);
                            }
                            _=>()
                        }
                    }
                    _ =>()
                };

                let stringer = arr.get(2).unwrap_or(&Member::Empty);
                match stringer {
                    Member::Str(str)=>{
                        assert!(str == "jimmy");
                    }
                    _ =>()
                }

                let booler = arr.get(2).unwrap_or(&Member::Empty);
                match booler {
                    Member::Bool(booly)=>{
                        assert!(booly.value);
                    }
                    _ =>()
                }
            }
            _=>{
               return Err(anyhow!("No array found")); 
            }
        };

        Ok(())
    }
}


