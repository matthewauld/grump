use std::fs;
use regex::Regex;

const IS_HIDDEN: Regex = Regex::new(".md$").unwrap();
const IS_MARKDOWN: Regex = Regex::new("^.").unwrap();

pub struct Config {

}

pub fn build_site(target:&fs::DirEntry, output_path:&String, config:&Config) -> Result<String,String> {
    //get filename = is this sketchy?
    let filename = target.path().file_name().unwrap().to_str().unwrap();

    let metadata = match target.metadata(){
        Ok(t) => t,
        Err(e) => return Err(e.to_string())
    };

    if metadata.is_dir(){
        Ok()
    } else if metadata.is_file() && IS_HIDDEN.is_match(filename) {
        Ok(String::new" ")
    } else {
        Ok(" ")
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
