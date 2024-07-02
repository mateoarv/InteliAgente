use std::path::PathBuf;
use firestore::*;
use serde::{Deserialize, Serialize};
use chrono::prelude::*;
use anyhow::Result;

#[derive(Serialize, Deserialize)]
struct UserStruct {
    created_on: FirestoreTimestamp,
}

#[derive(Serialize, Deserialize)]
struct LogStruct {
    date: FirestoreTimestamp,
    duration: u32,
}

pub struct CloudManager {
    db: FirestoreDb,
}

impl CloudManager {
    pub async fn new(project_id: String, key_path: PathBuf) -> Result<Self> {
        let db = FirestoreDb::with_options_service_account_key_file(
            FirestoreDbOptions::new(project_id),
            key_path,
        ).await?;

        Ok(Self {
            db,
        })
    }

    pub async fn create_user(&self, email: &String) {
        let user = UserStruct {
            created_on: FirestoreTimestamp::from(Utc::now()),
        };

        let existing: Option<UserStruct> = self.db.fluent()
            .select()
            .by_id_in("users")
            .obj()
            .one(&email)
            .await
            .unwrap();

        if existing.is_some() {
            println!("User exists");
            return;
        }

        let _: UserStruct = self.db.fluent()
            .insert()
            .into("users")
            .document_id(&email)
            .object(&user)
            .execute()
            .await
            .unwrap();
        println!("User created");
    }

    pub async fn add_log(&self, email: &String, duration: u32) {
        let parent_path = self.db.parent_path("users", &email).unwrap();

        let now = Utc::now();
        let log_struct = LogStruct {
            date: FirestoreTimestamp::from(now),
            duration,
        };

        let _: LogStruct = self.db.fluent()
            .insert()
            .into("logs")
            //.document_id(now.to_string())
            .generate_document_id()
            .parent(&parent_path)
            .object(&log_struct)
            .execute()
            .await
            .unwrap();
    }
}


// #[tokio::main]
// async fn main() {
//     println!("Hello, world!");
//     //let db = FirestoreDb::new(FirestoreDbOptions::from_path("conf_2.conf")).await.unwrap();
//     let db = FirestoreDb::with_options_service_account_key_file(
//         FirestoreDbOptions::new("inteliagente-8728d".to_string()),
//         "conf_2.conf".into(),
//     ).await.unwrap();
//
//     create_user(&db, "mateo2@gmail.com".to_string()).await;
//     add_log(&db, "mateo2@gmail.com".to_string(), 69).await;
// }





