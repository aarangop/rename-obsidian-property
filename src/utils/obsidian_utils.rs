use std::{error::Error, fs, io, path::{Path, PathBuf}};
use std::io::BufRead;
use regex::Regex;

use super::match_files;


pub struct ObsidianNote {
  path: PathBuf,
  content: Option<Vec<String>>,
}

impl ObsidianNote {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        ObsidianNote {
            path: path.as_ref().to_path_buf(),
            content: None,
        }
    }

    pub fn read(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let file = fs::File::open(&self.path)?;
        let reader = io::BufReader::new(file);
        let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;
        Ok(lines)
    }

    pub fn load_content(&mut self) -> Result<(), Box<dyn Error>> {
        self.content = Some(self.read()?);
        Ok(())
    } 

    // Get a reference to the content if it's loaded
    pub fn content(&self) -> Option<&Vec<String>> {
        self.content.as_ref()
    }
    
    /// Save changes back to the file
    pub fn save(&mut self) -> io::Result<()> {
        let content = self.content().unwrap();
        let content = content.join("\n");
        fs::write(&self.path, content)?;
        Ok(())
    }

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

pub struct ObsidianNoteProcessor {
  notes: Vec<ObsidianNote>,
  modified_count: usize
}

impl ObsidianNoteProcessor {
    pub fn new(notes: Vec<ObsidianNote>) -> Self {
        Self {
            notes,
            modified_count: 0,
        }
    }

    pub fn modified_count(&self) -> usize {
      self.modified_count
    }

    pub fn notes(&self) -> &Vec<ObsidianNote> {
      &self.notes
    }

    pub fn load_content(&mut self) -> Result<&mut Self, Box<dyn Error>> {
        for note in &mut self.notes {
            note.load_content()?;
        }
        Ok(self)
    }

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

    pub fn save(&mut self) -> Result<&mut Self, Box<dyn Error>> {
        for note in &mut self.notes {
            note.save()?;
        }
        Ok(self)
    }
}

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
