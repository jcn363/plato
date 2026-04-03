use epub_edit::{EpubEditorCore, EpubMetadata};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn list_epubs(directory: &str) -> io::Result<Vec<PathBuf>> {
    let mut epubs = Vec::new();
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext.to_str().unwrap_or("") == "epub" {
                epubs.push(path);
            }
        }
    }
    Ok(epubs)
}

fn print_menu() {
    println!("=== EPUB Editor Menu ===");
    println!("1. List EPUB files");
    println!("2. Open EPUB file");
    println!("3. View metadata");
    println!("4. Edit metadata");
    println!("5. List chapters");
    println!("6. Edit chapter");
    println!("7. Preview changes");
    println!("8. Undo last change");
    println!("9. Redo last change");
    println!("10. Save EPUB");
    println!("0. Exit");
    print!("Select option: ");
}

fn preview_metadata(meta: &EpubMetadata) -> String {
    let mut preview = String::from("Metadata Preview:\n");
    preview.push_str("-----------------\n");
    preview.push_str(&format!("Title: {}\n", meta.title));
    preview.push_str(&format!("Author: {}\n", meta.author));
    preview.push_str(&format!("Language: {}\n", meta.language));
    preview.push_str(&format!("Identifier: {}\n", meta.identifier));
    if let Some(ref p) = meta.publisher {
        preview.push_str(&format!("Publisher: {}\n", p));
    }
    if let Some(ref d) = meta.date {
        preview.push_str(&format!("Date: {}\n", d));
    }
    if let Some(ref d) = meta.description {
        preview.push_str(&format!("Description: {}\n", d));
    }
    preview
}

fn main() -> io::Result<()> {
    env_logger::init();

    let library_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/mnt/onboard".to_string());

    println!("EPUB Editor for Plato (using shared core)");
    println!("Library path: {}", library_path);
    println!();

    let mut current_editor: Option<EpubEditorCore> = None;
    let mut running = true;

    while running {
        print_menu();
        io::stdout().flush()?;

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        let choice = choice.trim();

        match choice {
            "1" => {
                println!("\nEPUB files in {}:", library_path);
                match list_epubs(&library_path) {
                    Ok(epubs) => {
                        for (i, epub) in epubs.iter().enumerate() {
                            println!("{}. {}", i + 1, epub.file_name().unwrap_or_default().to_string_lossy());
                        }
                        if epubs.is_empty() { println!("No EPUB files found."); }
                    }
                    Err(e) => println!("Error listing EPUBs: {}", e),
                }
                println!();
            }
            "2" => {
                print!("Enter EPUB file path: ");
                io::stdout().flush()?;
                let mut path = String::new();
                io::stdin().read_line(&mut path)?;
                let path = path.trim();

                if !path.is_empty() && Path::new(path).exists() {
                    match EpubEditorCore::new(path) {
                        Ok(editor) => {
                            current_editor = Some(editor);
                            println!("Opened: {}", path);
                        }
                        Err(e) => println!("Error opening EPUB: {}", e),
                    }
                } else {
                    println!("Error: Invalid file path.");
                }
                println!();
            }
            "3" => {
                if let Some(ref editor) = current_editor {
                    println!("\n{}", preview_metadata(&editor.metadata));
                } else {
                    println!("Error: No EPUB file open.");
                }
                println!();
            }
            "4" => {
                if let Some(ref mut editor) = current_editor {
                    let meta = &editor.metadata;
                    print!("Title [{}]: ", meta.title);
                    io::stdout().flush()?;
                    let mut input = String::new();
                    io::stdin().read_line(&mut input)?;
                    let title = input.trim().to_string();

                    print!("Author [{}]: ", meta.author);
                    io::stdout().flush()?;
                    input.clear();
                    io::stdin().read_line(&mut input)?;
                    let author = input.trim().to_string();

                    print!("Language [{}]: ", meta.language);
                    io::stdout().flush()?;
                    input.clear();
                    io::stdin().read_line(&mut input)?;
                    let language = input.trim().to_string();

                    let mut new_meta = meta.clone();
                    if !title.is_empty() { new_meta.title = title; }
                    if !author.is_empty() { new_meta.author = author; }
                    if !language.is_empty() { new_meta.language = language; }

                    println!("\n--- Metadata Preview ---");
                    println!("New Title: {}", new_meta.title);
                    println!("New Author: {}", new_meta.author);
                    println!("New Language: {}", new_meta.language);
                    
                    print!("Apply these changes? (y/n): ");
                    io::stdout().flush()?;
                    input.clear();
                    io::stdin().read_line(&mut input)?;
                    
                    if input.trim().to_lowercase() == "y" {
                        editor.set_metadata(new_meta);
                        println!("Metadata updated.");
                    }
                } else {
                    println!("Error: No EPUB file open.");
                }
                println!();
            }
            "5" => {
                if let Some(ref editor) = current_editor {
                    println!("\n=== Chapters ===");
                    for (i, chapter) in editor.chapters.iter().enumerate() {
                        println!("{}. {}", i + 1, chapter.title);
                    }
                } else {
                    println!("Error: No EPUB file open.");
                }
                println!();
            }
            "6" => {
                if let Some(ref mut editor) = current_editor {
                    print!("Chapter number: ");
                    io::stdout().flush()?;
                    let mut input = String::new();
                    io::stdin().read_line(&mut input)?;

                    if let Ok(index) = input.trim().parse::<usize>() {
                        if index > 0 && index <= editor.chapters.len() {
                            let chapter = &editor.chapters[index - 1];
                            println!("\n=== Chapter {}: {} ===", index, chapter.title);
                            let preview = chapter.content.chars().take(500).collect::<String>();
                            println!("{}\n...[truncated]", preview);

                            print!("Edit content? (y/n): ");
                            io::stdout().flush()?;
                            input.clear();
                            io::stdin().read_line(&mut input)?;

                            if input.trim().to_lowercase() == "y" {
                                println!("Enter new content (Type 'END' on a new line to finish):");
                                let mut new_content = String::new();
                                loop {
                                    let mut line = String::new();
                                    io::stdin().read_line(&mut line)?;
                                    if line.trim() == "END" { break; }
                                    new_content.push_str(&line);
                                }

                                if !new_content.trim().is_empty() {
                                    print!("Apply changes? (y/n): ");
                                    io::stdout().flush()?;
                                    input.clear();
                                    io::stdin().read_line(&mut input)?;
                                    if input.trim().to_lowercase() == "y" {
                                        if let Err(e) = editor.update_chapter(index - 1, new_content) {
                                            println!("Error updating chapter: {}", e);
                                        } else {
                                            println!("Chapter updated.");
                                        }
                                    }
                                }
                            }
                        } else {
                            println!("Invalid chapter number.");
                        }
                    }
                } else {
                    println!("Error: No EPUB file open.");
                }
                println!();
            }
            "7" => {
                if let Some(ref editor) = current_editor {
                    println!("\n=== Preview of Current State ===");
                    println!("{}", preview_metadata(&editor.metadata));
                    println!("\nChapters summary:");
                    for (i, chapter) in editor.chapters.iter().enumerate() {
                        println!("{}. {} ({} chars)", i + 1, chapter.title, chapter.content.len());
                    }
                } else {
                    println!("Error: No EPUB file open.");
                }
                println!();
            }
            "8" => {
                if let Some(ref mut editor) = current_editor {
                    match editor.undo() {
                        Ok(true) => println!("Undo successful."),
                        Ok(false) => println!("Nothing to undo."),
                        Err(e) => println!("Error during undo: {}", e),
                    }
                } else {
                    println!("Error: No EPUB file open.");
                }
                println!();
            }
            "9" => {
                if let Some(ref mut editor) = current_editor {
                    match editor.redo() {
                        Ok(true) => println!("Redo successful."),
                        Ok(false) => println!("Nothing to redo."),
                        Err(e) => println!("Error during redo: {}", e),
                    }
                } else {
                    println!("Error: No EPUB file open.");
                }
                println!();
            }
            "10" => {
                if let Some(ref mut editor) = current_editor {
                    match editor.save() {
                        Ok(_) => println!("EPUB saved successfully."),
                        Err(e) => println!("Error saving EPUB: {}", e),
                    }
                } else {
                    println!("Error: No EPUB file open.");
                }
                println!();
            }
            "0" => {
                running = false;
                println!("Goodbye!");
            }
            _ => println!("Invalid option."),
        }
    }
    Ok(())
}
