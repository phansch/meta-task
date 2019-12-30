use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Database {
    tasks: Vec<String>,
}

impl Database {
    /// Get a new instance of `Database`
    pub fn from_disk() -> Self {
        Self::ensure_file_exists();

        let contents = fs::read_to_string(Self::database_file())
            .expect("Something went wrong reading the file");
        toml::from_str(&contents).expect("Could not parse database file")
    }

    pub fn add_task(&mut self, task_name: &str) {
        // TODO: Handle duplicate tasks
        self.tasks.push(task_name.to_string());
    }

    pub fn task_exists(&self, task_name: &str) -> bool {
        self.tasks.contains(&task_name.to_string())
    }

    pub fn remove_task(&mut self, task_name: &str) {
        // TODO: Handle unknown task_name
        self.tasks.retain(|x| x != &task_name.to_string());
    }

    pub fn list_tasks(&self) -> Vec<String> {
        self.tasks.iter().map(|t| {
            format!("Task: {}", t)
        }).collect()
    }

    pub fn save(&self) {
        let toml = toml::to_string(&self)
            .expect("Could not serialize database");
        fs::write(Self::database_file(), toml)
            .expect("Could not save database file");
    }

    fn database_file() -> PathBuf {
        Self::data_dir().join("meta-task").join("database.toml")
    }

    fn data_dir() -> PathBuf {
        dirs::data_local_dir()
            .expect("Could not determine data dir")
    }

    fn ensure_file_exists() {
        if !Self::database_file().exists() {
            let db = Database { tasks: vec![] };
            db.save();
        }
    }
}
