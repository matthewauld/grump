
use std::io::Read;
use std::fs;
use std::fs::File;
use pulldown_cmark::{Options, html, Parser};
use glob;
use std::path;



#[derive(Debug)]
pub struct Config {
    pub cmark_options: Options,
    pub include_style: bool,
    pub style_path: path::PathBuf,
    pub include_script: bool,
    pub script_path: path::PathBuf,
    pub menu: String,
    pub ignore: Vec<glob::Pattern>,
    pub site_name: String,
}

//TODO, use paths isntead of strings
impl Config{

    pub fn new(target: &file_tree::Directory)->Config{

        let menu = format!("<nav><ul>{}</ul></nav>",target.generate_html_menu(&vec![]));
        let mut c = Options::empty();
        c.insert(Options::ENABLE_STRIKETHROUGH);
        c.insert(Options::ENABLE_TABLES);
        c.insert(Options::ENABLE_TASKLISTS);
        c.insert(Options::ENABLE_FOOTNOTES);
        return Config{
            site_name: String::from(target.path.canonicalize().unwrap().file_name().unwrap().to_str().unwrap()),
            cmark_options: c,
            include_style: true,
            style_path: path::PathBuf::from("/style.css"),
            include_script: true,
            script_path: path::PathBuf::from("/script.js"),
            menu: menu,
            ignore: vec![],

        }
    }


}

// Process an indavidual file item. If it is a markdown file, create an html file. If a directory, create an index file.
pub fn build_item(root: file_tree::FileSystemItem, config: &Config){
    match root {
        file_tree::FileSystemItem::FileEntry(item)=>{
            if item.extension == "md"{
                match process_markdown_file(item, config){
                    Ok(_) =>(),
                    Err(e) => eprintln!("Unable to process markdown file: {}",e)
                }
            }
        }
        file_tree::FileSystemItem::DirEntry(item)=>{
            // build an index if there is none! Also add an option in the config to disable
            if !item.hidden && !file_tree::should_ignore(&item.path, &config.ignore) && !&item.path.join("index.md").exists() {
                match build_default_index(&item,config){
                    Ok(_) =>(),
                    Err(e) => eprintln!("{}",e)
                };

            }
            for child in item.children {
                build_item(child, config);
            }
        }
    }
}

pub fn build_default_index(item: &file_tree::Directory, config: &Config) -> std::io::Result<()>{
    let mut output = String::from("<!-- grump --><!DOCTYPE html><html><head><meta charset='utf-8'>");
    output.push_str(&format!("<title>{}</title>",config.site_name));

    //add style
    if config.include_style {
        let p = &config.style_path.to_str().unwrap();
        output.push_str(&format!(" <link rel=\"stylesheet\" type=\"text/css\" href=\"{}\">", p));
    }
    //add script
    if config.include_script {
        let p = &config.script_path.to_str().unwrap();
        output.push_str(&format!("<script type = \"text/javascript\" src = \"{}\" ></script>",p));
    }
     output+=&format!("</head><body><div class=\"main\">{}<h1>{}</h1>{}</div></body></html>",config.menu, item.name, item.generate_directory_menu());
    let p = path::Path::new(&item.path).join("index.html");
    fs::write(&p,&output)
}

pub fn process_markdown_file(file: file_tree::File, config: &Config)->std::io::Result<()>{
    let mut input = String::new();
    let mut output = String::new();
    //create the new path

    let output_path = file.path.with_extension("html");
    //ignore if file hasn't changed
    //TODO: add logic to return ok if file doesn't need to be updated, or force it if config.force == true



    //convert to html
    //add beginning
    output.push_str("<!-- grump --><!DOCTYPE html><html><head><meta charset='utf-8'>");
    //add style
    output.push_str(&format!("<title>{}</title>",config.site_name));
    if config.include_style {
        let p = &config.style_path.to_str().unwrap();
        output.push_str(&format!(" <link rel=\"stylesheet\" type=\"text/css\" href=\"{}\">", p));
    }
    //add script
    if config.include_script {
        let p = &config.script_path.to_str().unwrap();
        output.push_str(&format!("<script type = \"text/javascript\" src = \"{}\" ></script>",p));
    }
    output.push_str("</head><body><div class=\"main\"></body></html>");

    //add body
    let mut f = File::open(file.path)?;
    f.read_to_string(&mut input)?;
    let parser = Parser::new_ext(&input, config.cmark_options);

    //TODO: Add menu logic
    html::push_html(&mut output, parser);
    //need to remove <p> tags around menu
    output = output.replace("{{MENU}}",&config.menu);
    output += "</div></body></html>";
    //add end


    fs::write(&output_path, &output)?;

    Ok(())
}




pub mod file_tree{
    use glob;
    use std::path;
    use std::fs;

    #[derive(Debug)]
    pub struct File {
        pub name: String,
        pub path: path::PathBuf,
        pub hidden: bool,
        pub extension: String,
        pub last_modified: std::time::SystemTime,
    }

    #[derive(Debug)]
    pub struct Directory {
        pub name: String,
        pub path: path::PathBuf,
        pub children: Vec<FileSystemItem>,
        pub hidden: bool,
    }

    #[derive(Debug)]
    pub enum FileSystemItem {
        DirEntry(Directory),
        FileEntry(File),
    }

    impl Directory {
        pub fn new(dir_path: &path::PathBuf)->std::io::Result<Directory> {
            let long_path = dir_path.canonicalize()?;
            let mut filename = long_path.file_name().unwrap_or(std::ffi::OsStr::new(".")).to_str().unwrap();
            let hidden: bool;
            if filename.starts_with(".")&& filename!="." {
                filename = without_first(&filename);
                hidden = true;
            } else {
                hidden = false;
            }

            let mut new_dir = Directory {
                name: String::from(filename),
                path: path::PathBuf::from(dir_path),
                children: Vec::new(),
                hidden: hidden,
            };

            for entry in fs::read_dir(dir_path)?{
                let target = entry?;
                let t = target.file_type()?;
                if t.is_dir(){
                    match Directory::new(&target.path()) {
                        Ok(x)    => new_dir.children.push(FileSystemItem::DirEntry(x)),
                        Err(err) => {
                            eprintln!("Unable to to process {:?}, ignoring: {}", target.path(),err);
                            continue
                        }
                    }

                } else if t.is_file() {
                    match File::new(&target.path()) {
                        Ok(x)    => new_dir.children.push(FileSystemItem::FileEntry(x)),
                        Err(err) => {
                            eprintln!("Unable to to process {:?}, ignoring: {}", target.path(),err);
                            continue
                        }
                    }
                }
            }
        return Ok(new_dir);
        }
        pub fn generate_html_menu(&self, ignore: &Vec<glob::Pattern>) -> String {
            let child_string = self.children.iter().fold(String::new(),|acc, x| acc + &x.generate_submenu(ignore, &self.path).unwrap_or(String::new()));
            format!("</div><nav><div><a href=\"/\">Home</a></div><ul>{}</ul></nav><div class=\"main\">",child_string)
        }

        pub fn generate_directory_menu(&self) -> String {
            let child_string: String = self.children.iter().fold(String::new(), |acc, x| {
                //TODO: right now this adds all files regardless of ignore, I am not sure if I want to change this...

                match x {
                    FileSystemItem::DirEntry(dir) if !dir.hidden =>  acc + &format!("<li><a href=\"{}\">📁 {}</a></li>",&dir.name, dir.path.strip_prefix(&self.path).unwrap().to_str().unwrap()),
                    FileSystemItem::FileEntry(file) if !file.hidden => {

                        match file {
                            file if file.name == "index.html" => acc,
                            file if file.extension == "md" =>{
                                let possible_site = file.path.with_extension("html");
                                if !possible_site.exists(){
                                    acc + &format!("<li><a href=\"{}\">📄 {}</a></li>", possible_site.strip_prefix(&self.path).unwrap().to_str().unwrap(),&file.name[0..file.name.len()-5])
                                } else {
                                    acc
                                }

                            } ,
                            file if file.extension == "html" => acc + &format!("<li><a href=\"{}\">📄 {}</a></li>", file.path.strip_prefix(&self.path).unwrap().to_str().unwrap(),&file.name[0..file.name.len()-5]),
                            file => acc + &format!("<li><a href=\"{}\">📄 {}</a></li>", file.path.strip_prefix(&self.path).unwrap().to_str().unwrap(),file.name)
                        }

                    },
                    _ => acc

                }
            });
            return format!("<ul>{}</ul>",child_string)
        }


    }

    impl File {
        pub fn new(file_path: &path::PathBuf)->std::io::Result<File> {
            let long_path = file_path.canonicalize()?;
            let mut filename  = long_path.file_name().unwrap_or(std::ffi::OsStr::new("")).to_str().unwrap();
            let extension = file_path.extension().unwrap_or(std::ffi::OsStr::new("")).to_str().unwrap();
            let hidden: bool;
            if filename.starts_with("."){
                filename = without_first(&filename);
                hidden = true;
            } else {
                hidden = false;
            }
            let new_file = File {
                name: String::from(filename),
                path: path::PathBuf::from(file_path),
                hidden: hidden,
                extension: String::from(extension),
                last_modified: fs::metadata(file_path)?.modified()?,

            };
            Ok(new_file)
        }
    }

    impl FileSystemItem {
        // this is horribly inefficient and needs to get cleaned up. change from if to guard
        pub fn generate_submenu(&self, ignore:&Vec<glob::Pattern>, root: &path::PathBuf)->Option<String>{
            match self {
                FileSystemItem::FileEntry(x)=> {
                    if x.extension == "md"  {
                        let possible_site = x.path.with_extension("html");
                        if !possible_site.exists() && !should_ignore(&possible_site,&ignore) {
                            // get the name of the file
                            let i:usize  = x.name.rfind('.').unwrap_or(x.path.to_str()?.len());
                            let name = String::from(&x.name[0..i]);

                            //remove the first folder from
                            let link = String::from("/") + x.path.strip_prefix(root).unwrap().with_extension("html").to_str().unwrap_or("");
                            Some(format!("<li class=\"file\"><a href=\"{}\">{}</a></li>",&link, name))
                        } else {
                            None
                        }


                    }
                     else if !x.hidden && !should_ignore(&x.path,&ignore) && x.name != "index.html" && x.extension == "html" {

                        // get the name of the file
                        let i:usize  = x.name.rfind('.').unwrap_or(x.path.to_str()?.len());
                        let name = String::from(&x.name[0..i]);

                        //remove the first folder from
                        let link = String::from("/") + x.path.strip_prefix(root).unwrap().to_str().unwrap_or("");
                        Some(format!("<li class=\"file\"><a href=\"{}\">{}</a></li>",&link, name))

                    } else {
                        None
                    }
                }

                FileSystemItem::DirEntry(x)=>{
                    if !x.hidden && !should_ignore(&x.path,&ignore) {
                        //remove the root path
                        let link = String::from("/") + x.path.strip_prefix(root).unwrap().to_str().unwrap_or("/");
                        Some(format!("<li class=\"dir\"><a href=\"{}\">{}</a><ul>{}</ul></li>",&link,&x.name,x.children.iter().fold(String::new(),|acc, x| acc + &x.generate_submenu(ignore, root).unwrap_or(String::new()))))
                    } else {
                        None
                    }
                },

            }
        }
    }

    fn without_first(string: &str) -> &str {
        string
            .char_indices()
            .nth(1)
            .and_then(|(i, _)| string.get(i..))
            .unwrap_or("")
    }

    pub fn should_ignore(path: &path::PathBuf, ignore: &Vec<glob::Pattern>)-> bool{
        for pattern in ignore {
            if pattern.matches(path.to_str().unwrap()){
                return true;
            }
        }
        return false;
    }
}
