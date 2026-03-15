use std::{io, path::PathBuf};

use uuid::Uuid;

#[derive(Clone)]
struct EditorInstance {
  id: String,
  content: String,
  path: String,
  saved: bool
}

pub struct Editor {
  pub auto_save: bool,
  editor_instances: Vec<EditorInstance>,
  active_instance_id: Option<String>,
  pub current_content: String
}

impl Editor {
  pub fn new() -> Self {
    Self {
      auto_save: false,
      editor_instances: Vec::new(),
      active_instance_id: None,
      current_content: String::new()
    }
  }

  pub fn new_instance(&mut self, path: String) -> Result<(), io::Error> {
    match self.instance_by_path(path.clone()) {
      Ok(i) => { self.set_active_instance(i.id).unwrap(); },
      Err(_) => {
        let mut instance = EditorInstance::new(path.clone());
        instance.parse(path.clone())?;
        self.active_instance_id = Some(instance.id());
        self.editor_instances.push(instance);
        self.current_content = self.current_instance()?.clone().content;
      }
    };
    Ok(())
  }

  fn set_active_instance(&mut self, instace_id: String) -> Result<(), io::Error> {
    self.active_instance_id = Some(instace_id);
    self.current_content = self.current_instance()?.clone().content;
    Ok(())
  }

  pub fn instances_data(&mut self) -> Result<Vec<(String, String)>, io::Error> {
    let instances = self.editor_instances.clone();
    let mut data = Vec::new();
    for i in instances {
      let path = PathBuf::from(i.path.clone());
      let name = path.file_name()
        .and_then(|os_str| os_str.to_str())
        .map(|s| s.to_string()).unwrap_or("No name".to_string());
      data.push((name, i.path));
    }
    Ok(data)
  }

  fn instance_by_path(&self, path: String) -> Result<EditorInstance, io::Error> {
    match self.editor_instances.iter().find(|i| i.path == path) {
      Some(i) => return Ok(i.clone()),
      None => return Err(io::Error::new(io::ErrorKind::NotFound, "Instance with such path was not found!"))
    }
  }

  pub fn update_instance_content(&mut self, new_content: String) -> Result<(), io::Error> {
    self.current_content = new_content.clone();
    let auto_save = self.auto_save;
    let current_instance = self.current_instance()?;
    current_instance.content = new_content;
    match auto_save {
      true => { current_instance.save()?; },
      false => { current_instance.set_unsaved(); },
    }
    Ok(())
  }

  pub fn save_current_instance(&mut self) ->Result<(), io::Error> {
    let current_instance = self.current_instance()?;
    current_instance.save()?;
    Ok(())
  }

  fn current_instance(&mut self) -> Result<&mut EditorInstance, io::Error> {
    let instance = self.editor_instances.iter_mut().find(|i| i.id == self.active_instance_id.as_deref().unwrap());
    match instance {
      Some(i) => Ok(i),
      None =>  return Err(io::Error::new(io::ErrorKind::NotFound, "No current instance found!"))
    }
  }

  pub fn current_content(&mut self) -> Result<String, io::Error> {
    if self.active_instance_id.is_none() {
      return Ok(String::new())
    }
    let mut instances_copy = self.editor_instances.clone();
    instances_copy.retain(|e| e.id == self.active_instance_id.as_deref().unwrap());
    Ok(instances_copy[0].content.clone())
  }
}

impl EditorInstance {
  pub fn new(path: String) -> Self {
    let instance_id = format!("{}-{}", path, Uuid::new_v4());
    Self {
      id: instance_id,
      content: String::new(),
      path,
      saved: true
    }
  }

  pub fn parse(&mut self, path: String) -> Result<&Self, io::Error> {
    println!("Parsing file: {}", path);
    self.content = std::fs::read_to_string(path)?;
    Ok(self)
  }

  pub fn save(&mut self) -> Result<&mut Self, io::Error> {
    std::fs::write(self.path.clone(), self.content.clone())?;
    self.saved = true;
    Ok(self)
  }

  pub fn set_unsaved(&mut self) {
    self.saved = false;
  }

  pub fn id(&self) -> String {
    self.id.clone()
  }
}