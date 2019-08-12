
use super::file_tree;
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
    pub force: bool,
    pub ignore: Vec<glob::Pattern>,
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
            cmark_options: c,
            include_style: true,
            style_path: path::PathBuf::from("/style.css"),
            include_script: true,
            script_path: path::PathBuf::from("/script.css"),
            menu: menu,
            force: false,
            ignore: vec![],

        }
    }


}
pub fn build_item(root: file_tree::FileSystemItem, config: &Config){
    match root {
        file_tree::FileSystemItem::FileEntry(item)=>{
            if item.extension == "md"{
                match process_markdown_file(item, config){
                    Ok(_) =>(),
                    Err(e) => eprintln!("{}",e)
                }
            }
        }
        file_tree::FileSystemItem::DirEntry(item)=>{
            //TODO=> build an index if there is none! Also add an option in the config to disable
            if !item.hidden && !file_tree::should_ignore(&item.path, &config.ignore) && !path::Path::new(&item.path).join("index.md").exists() {
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
    let mut output = String::from("<html><head>");
    if config.include_style {
        let p = &config.style_path.to_str().unwrap();
        output.push_str(&format!(" <link rel=\"stylesheet\" type=\"text/css\" href=\"{}\">", p));
    }
    //add script
    if config.include_script {
        let p = &config.script_path.to_str().unwrap();
        output.push_str(&format!("<script type = \"text/javascript\" src = \"{}\" ></script>",p));
    }
     output+=&format!("</head><body>{}<h1>{}</h1>{}</body></html>",config.menu, item.name, item.generate_directory_menu());
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
    output.push_str("<!DOCTYPE html><html><head>");
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
    output.push_str("</head><body>");
    //add body
    let mut f = File::open(file.path)?;
    f.read_to_string(&mut input)?;
    let parser = Parser::new_ext(&input, config.cmark_options);
    //TODO: Add menu logic

    html::push_html(&mut output, parser);
    let output = output.replace("{{MENU}}",&config.menu);
    //add end


    fs::write(&output_path, &output)?;

    Ok(())
}


pub fn build_site(target:&str){
    let dir = file_tree::Directory::new(target).expect("Target directory not found");
    let config = Config::new(&dir);
    let root = file_tree::FileSystemItem::DirEntry(dir);


    build_item(root,&config);
}
