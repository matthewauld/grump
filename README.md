# Grump
Grump is a static site generator for when you just need a website.

While there are many (many) static site generators that use template engines, config files templates and optimizers to generate beautiful, maintainable websites, this is not one of those.

Sometimes, you just need a website to share some files. Or list some information. You may want a bit of css styling or a script, but you don't need anything fancy.

Grump is for you. Just call `grump <directory>` and you are done.

## Grump does three things

1. For every file with a `<name>.md` suffix it generates an html file called `<name>.html`.

2.  For every visible directory (doesn't start with `.` or doesn't match a pattern passed with with `--ignore`). It generates an `index.html` file with a menu with a `menu`(see below) and a list of files and sites that directory contains.

3. The one little bit of magic - Grump will replace `{{MENU}}` with a  file tree of all the visible directories and .html files in a `<nav>` element made of nested lists. Nothing too fancy.

## Scripts and css
Grump will look for `style.css` and `script.js` files in the top of the directory, and will add them to the top of every website it generates.

## Usage
Grump does have some simple options:
```USAGE:
    grump [OPTIONS] [target]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --ignore <ignore>    Patterns to ignore [default: ]
    -j, --js <js>            Javascript file to include
    -s, --style <style>      css file to include

ARGS:
    <target>    Target directory [default: .]
  ```

Tested on linux and mac.

## Grump plays well with everything.
Grump is so simple that any static site server will work. `python -m SimpleHTTPServer` works fine for local servers.
Also is a great way to share files via the [Dat Protocol](https://dat.foundation/) and [Beaker Browser](https://beakerbrowser.com/)

## More to come?
Honestly, this project was a good way to learn rust, however, in the name of completeness, there are a few things I wouldn't mind adding at some point:
- Default CSS and JS files, just something simple.
- a `grump scrub` to remove files grump creates. all generate files start with `<!--grump-->` So this wouldn't be too hard.
- A better menu setup - maybe with some default styling.
