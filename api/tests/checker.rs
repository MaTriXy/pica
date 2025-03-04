use std::{
    fs::File,
    io::{BufReader, Read},
};

use serde::{de::DeserializeOwned, Serialize};

pub enum CheckType {
    Json,
    Bson,
}

pub trait JsonChecker {
    /// Check if the Struct passed can be serialized and deserialized using a file
    /// as a reference
    fn check<T: Serialize + DeserializeOwned + PartialEq>(
        &self,
        value: &T,
        r#type: CheckType,
    ) -> bool;

    /// The location where the checker will look for the file. If the folder doesn't exist, it will
    /// be created. It is always in relation to the current directory.
    fn location(&self) -> String {
        let current_dir = std::env::current_dir().expect("Failed to get current directory");

        let json_checks_dir = current_dir.join("tests").join("resource");

        if !json_checks_dir.exists() {
            std::fs::create_dir(&json_checks_dir).expect("Failed to create json_checks directory");
        }

        json_checks_dir
            .to_str()
            .expect("Failed to convert path to string")
            .to_string()
    }
}

#[derive(Debug, Clone, Default)]
pub struct JsonCheckerImpl {
    /// If true, the checker will override the file if it exists
    /// or create a new file if it doesn't exist
    r#override: bool,
}

impl JsonCheckerImpl {
    pub fn r#override(mut self) -> Self {
        self.r#override = true;
        self
    }
}

impl JsonChecker for JsonCheckerImpl {
    fn check<T: Serialize + DeserializeOwned + PartialEq>(
        &self,
        value: &T,
        r#type: CheckType,
    ) -> bool {
        let type_name = std::any::type_name::<T>().to_string().replace("::", "_");

        let serialized_json =
            serde_json::to_string_pretty(value).expect("Failed to serialize value");
        let serialized_bson = bson::to_vec(value).expect("Failed to serialize value");

        if self.r#override {
            let bson_file_path = self.location() + &format!("/{}.bson", type_name);
            let json_file_path = self.location() + &format!("/{}.json", type_name);

            std::fs::write(json_file_path, serialized_json).expect("Failed to write JSON file");
            std::fs::write(bson_file_path, serialized_bson).expect("Failed to write BSON file");
            panic!("Override flag is enabled, remember to disable and commit the changes");
        }

        let file_path = match r#type {
            CheckType::Json => self.location() + &format!("/{}.json", type_name),
            CheckType::Bson => self.location() + &format!("/{}.bson", type_name),
        };

        match r#type {
            CheckType::Json => {
                if self.r#override {
                    std::fs::write(file_path, serialized_json).expect("Failed to write to file");
                    panic!("Override flag is enabled, remember to disable and commit the changes");
                }

                let file = File::open(file_path.clone());

                if file.is_err() {
                    return false;
                }

                let file = file.expect("Failed to open file");
                let mut file = BufReader::new(file);

                let mut contents = String::new();
                file.read_to_string(&mut contents)
                    .expect("Failed to read file contents");

                let expected = serde_json::from_str::<T>(&contents)
                    .expect("Failed to deserialize expect value");

                let actual = serde_json::from_str::<T>(&serialized_json)
                    .expect("Failed to deserialize actual value");

                expected == actual
            }
            CheckType::Bson => {
                let file = File::open(file_path.clone());

                if file.is_err() {
                    return false;
                }

                let file = file.expect("Failed to open file");
                let mut file = BufReader::new(file);

                let mut contents: Vec<u8> = vec![];
                file.read_to_end(&mut contents)
                    .expect("Failed to read file");

                let expected =
                    bson::from_slice::<T>(&contents).expect("Failed to deserialize expect value");

                let actual = bson::from_slice::<T>(&serialized_bson)
                    .expect("Failed to deserialize actual value");

                expected == actual
            }
        }
    }
}
