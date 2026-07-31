fn main() {
    let acknowledgements = two_face::acknowledgement::listing();
    let mut markdown = String::from(
        "Mdir4 syntax definition acknowledgements\n\n\
         Syntax definitions are supplied by two-face 0.5.1. Their generation and curation\n\
         draw on the [bat project](https://github.com/sharkdp/bat).\n\n\
         # Syntaxes\n\n",
    );
    for license in acknowledgements.for_syntaxes() {
        license.write_md(&mut markdown);
    }
    print!("{markdown}");
}
