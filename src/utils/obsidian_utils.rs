//! Utilities for working with Obsidian notes.
//! 
//! This module provides functionality for reading, modifying, and saving
//! Obsidian notes, with a focus on modifying frontmatter properties.

use std::{error::Error, fs, io, path::{Path, PathBuf}};
use std::io::BufRead;
use regex::Regex;

use super::match_files;


/// Represents a single Obsidian note with its path and content.
/// 
/// This struct provides methods to load, modify, and save Obsidian notes.
pub struct ObsidianNote {
  /// Path to the Obsidian note file
  path: PathBuf,
  /// Optional content of the note, loaded on demand
  content: Option<Vec<String>>,
}

impl ObsidianNote {
    /// Creates a new ObsidianNote instance for the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - A path reference to the Obsidian note
    ///
    /// # Returns
    ///
    /// A new ObsidianNote instance without loaded content
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        ObsidianNote {
            path: path.as_ref().to_path_buf(),
            content: None,
        }
    }

    /// Reads the content of the note from disk.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<String>, Box<dyn Error>>` - The note content as lines or an error
    pub fn read(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let file = fs::File::open(&self.path)?;
        let reader = io::BufReader::new(file);
        let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;
        Ok(lines)
    }

    /// Loads the note content into memory.
    ///
    /// # Returns
    ///
    /// * `Result<(), Box<dyn Error>>` - Success or an error
    pub fn load_content(&mut self) -> Result<(), Box<dyn Error>> {
        self.content = Some(self.read()?);
        Ok(())
    } 

    /// Gets a reference to the note content if it's loaded.
    ///
    /// # Returns
    ///
    /// Optional reference to the content lines
    pub fn content(&self) -> Option<&Vec<String>> {
        self.content.as_ref()
    }
    
    /// Saves the note content back to disk.
    ///
    /// # Returns
    ///
    /// * `io::Result<()>` - Success or an IO error
    pub fn save(&mut self) -> io::Result<()> {
        let content = self.content().unwrap();
        let content = content.join("\n");
        fs::write(&self.path, content)?;
        Ok(())
    }

    /// Renames a property in the note's frontmatter.
    ///
    /// This method searches for a property with the given name in the frontmatter
    /// and renames it to the new name.
    ///
    /// # Arguments
    ///
    /// * `property` - The property name to find and replace
    /// * `new_name` - The new name for the property
    ///
    /// # Returns
    ///
    /// * `Result<&ObsidianNote, Box<dyn Error>>` - Reference to self or an error
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Note content couldn't be loaded
    /// - Note doesn't have frontmatter (starting with `---`)
    /// - The property wasn't found in the note
    pub fn rename_property(&mut self, property: &str, new_name: &str) -> Result<&ObsidianNote, Box<dyn Error>> {
      // Load note contents.
      self.load_content()?;
      let mut modified = false;

      let content = match self.content(){
        Some(c) => c,
        None => return Err("Failed to load note content".into())
      };

      // Check if the file has frontmatter.
      let re = Regex::new(r"^---$")?;
      let mut new_content : Vec<String>= Vec::new();

      if !re.is_match(content[0].as_str()) {
        return Err("Note does not have frontmatter".into());
      }

      // Search for property and rename it.
      let re = Regex::new(&format!(r"{}:", property))?;
      for line in content {
        if re.is_match(line.as_str()) {
          println!("Renaming property in {}", self.path.display());
          let new_line = re.replace(line.as_str(), format!("{}:", new_name).as_str());
          new_content.push(new_line.to_string());
          modified = true;
        } else {
          new_content.push(line.to_string());
        }
      }

      if !modified {
        return Err("Property not found".into());
      }

      // Update note content.
      self.content = Some(new_content);
      self.save()?;

      Ok(self)
    }
}

/// Processor for batch operations on multiple Obsidian notes.
///
/// This struct provides functionality to process multiple notes at once,
/// tracking how many notes were modified during operations.
pub struct ObsidianNoteProcessor {
  /// Collection of Obsidian notes to process
  notes: Vec<ObsidianNote>,
  /// Counter of how many notes were modified in the last operation
  modified_count: usize
}

impl ObsidianNoteProcessor {
    /// Creates a new processor with the given collection of notes.
    ///
    /// # Arguments
    ///
    /// * `notes` - Vector of ObsidianNote objects to process
    ///
    /// # Returns
    ///
    /// A new ObsidianNoteProcessor instance
    pub fn new(notes: Vec<ObsidianNote>) -> Self {
        Self {
            notes,
            modified_count: 0,
        }
    }

    /// Returns the number of notes modified in the last operation.
    ///
    /// # Returns
    ///
    /// Count of modified notes
    pub fn modified_count(&self) -> usize {
      self.modified_count
    }

    /// Gets a reference to the collection of notes being processed.
    ///
    /// # Returns
    ///
    /// Reference to the vector of notes
    pub fn notes(&self) -> &Vec<ObsidianNote> {
      &self.notes
    }

    /// Loads content for all notes in the collection.
    ///
    /// # Returns
    ///
    /// * `Result<&mut Self, Box<dyn Error>>` - Reference to self or an error
    pub fn load_content(&mut self) -> Result<&mut Self, Box<dyn Error>> {
        for note in &mut self.notes {
            note.load_content()?;
        }
        Ok(self)
    }

    /// Renames a property in all notes where it exists.
    ///
    /// # Arguments
    ///
    /// * `old_name` - The property name to find and replace
    /// * `new_name` - The new name for the property
    ///
    /// # Returns
    ///
    /// * `Result<&mut Self, Box<dyn Error>>` - Reference to self or an error
    pub fn rename_property(&mut self, old_name: &str, new_name: &str) 
        -> Result<&mut Self, Box<dyn Error>> {
        let modified_notes: Vec<&mut ObsidianNote> = self.notes
          .iter_mut()
          .filter_map(|note| match note.rename_property(old_name, new_name) {
              Ok(_) => {
                  Some(note)
              }
              Err(_) => {
                  None
              }
          })
          .collect();
        self.modified_count = modified_notes.len();  
        Ok(self)
    }

    /// Saves all notes back to disk.
    ///
    /// # Returns
    ///
    /// * `Result<&mut Self, Box<dyn Error>>` - Reference to self or an error
    pub fn save(&mut self) -> Result<&mut Self, Box<dyn Error>> {
        for note in &mut self.notes {
            note.save()?;
        }
        Ok(self)
    }
}

/// Loads Obsidian notes matching the given pattern.
///
/// # Arguments
///
/// * `pattern` - A glob pattern string to match against file paths
///
/// # Returns
///
/// * `Result<Vec<ObsidianNote>, Box<dyn Error>>` - Collection of loaded notes or an error
///
/// # Examples
///
/// ```
/// let notes = load_obsidian_notes(&"./vault/**/*.md".to_string())?;
/// println!("Found {} Obsidian notes", notes.len());
/// ```
pub fn load_obsidian_notes(pattern: &String) -> Result<Vec<ObsidianNote>, Box<dyn Error>>{
  let paths = match_files(pattern)?;

  // Obsidian notes are always markdown format, compile a regex that checks that the file path ends with .md.
  let re = Regex::new(r"\.md$")?;
  let obsidian_notes: Vec<ObsidianNote> = paths
    .iter()
    .filter_map(|path| {
      path.to_str().and_then(|p| {
        if re.is_match(p) {
          Some(ObsidianNote::new(path))
        } else {
          None
        }
      })
    })
    .collect();

  Ok(obsidian_notes)
}
