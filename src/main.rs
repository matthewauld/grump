
extern crate structopt;
extern crate glob;
use structopt::StructOpt;
use pulldown_cmark::Options;
use std::path::PathBuf;
mod lib;
use lib::file_tree;


#[derive(Debug, StructOpt)]
#[structopt(name="grump", about ="A static site generator for when you just need a website.")]
struct Opt {
    /// css file to include
    #[structopt(parse(from_os_str), name = "style", long, short)]
    style: Option<PathBuf>,
    /// Javascript file to include
    #[structopt(parse(from_os_str),name = "js", long, short)]
    script: Option<PathBuf>,

    /// Patterns to ignore
    #[structopt(name = "ignore", default_value="",long, short)]
    ignore: String,

    /// Target directory
    #[structopt(parse(from_os_str),name = "target", default_value=".")]
    target: PathBuf
}



pub fn main(){
    // get command line arguments
    let opt = Opt::from_args();
    // create file tree
    let root = file_tree::Directory::new(&opt.target).expect("Target directory not found");
    let mut ignore = Vec::new();
    for elem in opt.ignore.split(' '){
        ignore.push(glob::Pattern::new(elem).unwrap());
    }


    let menu = root.generate_html_menu(&ignore);

    // TODO: Check to ensure script files and style files are valid, and tell user. Also clean up this mess
    let mut use_style = false;
    let mut use_script = false;
    let style: PathBuf = match opt.style {
        None => {
            let possible_file = opt.target.clone().join("style.css");
            if possible_file.exists() && possible_file.is_file() {
                use_style = true;
            }
            PathBuf::from("/").join(possible_file.strip_prefix(&opt.target).unwrap())
        },
        Some(y) =>{
            use_style = true;
            y
        }

    };
    let script = match opt.script {
        None => {
            let possible_file = opt.target.clone().join("script.js");
            println!("{:?}",possible_file);
            if possible_file.exists() && possible_file.is_file(){
                use_script = true;
            }
            PathBuf::from("/").join(possible_file.strip_prefix(&opt.target).unwrap())

        },
        Some(y) =>{
            use_script = true;
            y
        }

    };

    //generate markdown options
    let mut c = Options::empty();
    c.insert(Options::ENABLE_STRIKETHROUGH);
    c.insert(Options::ENABLE_TABLES);
    c.insert(Options::ENABLE_TASKLISTS);
    c.insert(Options::ENABLE_FOOTNOTES);

    let config =  lib::Config {
        site_name: String::from(opt.target.canonicalize().unwrap().file_name().unwrap().to_str().unwrap()),
        cmark_options: c,
        include_style: use_style,
        style_path: style,
        include_script: use_script,
        script_path: script,
        menu: menu,
        ignore:ignore,
    };
    println!("{:?}",config);
    lib::build_item(file_tree::FileSystemItem::DirEntry(root),&config);


}
