
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
        pub fn new(dir_path: &str)->std::io::Result<Directory> {
            let mut filename = path::Path::new(dir_path).file_name().unwrap_or(std::ffi::OsStr::new(".")).to_str().unwrap();
            let hidden: bool;
            if filename.starts_with(".")&& filename!="."{
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
                let target = entry.unwrap();
                let t = target.file_type()?;
                if t.is_dir(){
                    new_dir.children.push(FileSystemItem::DirEntry(Directory::new(target.path().to_str().unwrap())?));
                } else if t.is_file() {
                    new_dir.children.push(FileSystemItem::FileEntry(File::new(target.path().to_str().unwrap())?));
                }
            }
        return Ok(new_dir);
        }
        pub fn generate_html_menu(&self, ignore: &Vec<glob::Pattern>) -> String {
            let child_string = self.children.iter().fold(String::new(),|acc, x| acc + &x.generate_submenu(ignore, &self.path).unwrap_or(String::new()));
            format!("<nav><div><a href=\"/\">{}</a></div><ul>{}</ul></nav>",self.name,child_string)
        }

        pub fn generate_directory_menu(&self) -> String {
            let child_string: String = self.children.iter().fold(String::new(), |acc, x| {
                match x {
                    FileSystemItem::DirEntry(dir) => acc + &format!("<li><a href=\"{}\">{}</li>",&dir.name, dir.path.strip_prefix(&self.path).unwrap().to_str().unwrap()),
                    FileSystemItem::FileEntry(file) => acc + &format!("<li><a href=\"{}\">{}</li>",file.name, file.path.strip_prefix(&self.path).unwrap().to_str().unwrap())
                }
            });
            return format!("<ul>{}</ul>",child_string)
        }


    }

    impl File {
        pub fn new(file_path: &str)->std::io::Result<File> {
            let filename_path = path::Path::new(file_path);
            let mut filename  = filename_path.file_name().unwrap_or(std::ffi::OsStr::new("")).to_str().unwrap();
            let extension = filename_path.extension().unwrap_or(std::ffi::OsStr::new("")).to_str().unwrap();
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

        pub fn generate_submenu(&self, ignore:&Vec<glob::Pattern>, root: &path::PathBuf)->Option<String>{
            match self {
                FileSystemItem::FileEntry(x)=> {
                    if !x.hidden && !should_ignore(&x.path,&ignore) && x.name != "index.html" && x.extension == "html" {

                        // get the name of the file
                        let i:usize  = x.name.rfind('.').unwrap_or(x.path.to_str()?.len());
                        let name = String::from(&x.name[0..i]);

                        //remove the first folder from  TODO: replace with strip prefex
                        let link = String::from("/") + x.path.strip_prefix(root).unwrap().to_str().unwrap_or("/");
                        Some(format!("<li><a href=\"{}\">{}</a></li>",&link, name))
                    } else {
                        None
                    }
                }

                FileSystemItem::DirEntry(x)=>{
                    if !x.hidden && !should_ignore(&x.path,&ignore) {
                        //remove the first folder
                        let link = String::from("/") + x.path.strip_prefix(root).unwrap().to_str().unwrap_or("/");
                        Some(format!("<li><a href=\"{}\">{}</a><ul>{}</ul></li>",&link,&x.name,x.children.iter().fold(String::new(),|acc, x| acc + &x.generate_submenu(ignore, root).unwrap_or(String::new()))))
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
