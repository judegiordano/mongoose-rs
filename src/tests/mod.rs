pub mod create_tests;
pub mod delete_tests;
pub mod read_tests;
pub mod update_tests;

#[cfg(test)]
mod mock {
    use serde::{Deserialize, Serialize};

    use crate::{
        bson::{doc, DateTime},
        Model,
    };

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Address {
        pub address: u32,
        pub street: String,
        pub city: String,
        pub state: String,
        pub zip: String,
        pub country: String,
        pub apt_number: Option<String>,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct User {
        #[serde(rename = "_id")]
        pub id: String,
        pub username: String,
        pub email: String,
        pub avatar_hash: String,
        pub slug: String,
        pub password: String,
        pub age: u32,
        pub address: Address,
        pub example_array: Vec<u32>,
        pub created_at: DateTime,
        pub updated_at: DateTime,
    }

    impl Default for User {
        fn default() -> Self {
            let now = chrono::Utc::now();
            Self {
                id: Self::generate_nanoid(),
                username: String::new(),
                email: String::new(),
                avatar_hash: String::new(),
                slug: String::new(),
                password: String::new(),
                example_array: Vec::new(),
                address: Address {
                    address: u32::default(),
                    street: String::new(),
                    city: String::new(),
                    state: String::new(),
                    zip: String::new(),
                    country: String::new(),
                    apt_number: None,
                },
                age: u32::default(),
                created_at: now.into(),
                updated_at: now.into(),
            }
        }
    }

    impl Model for User {}

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Post {
        #[serde(rename = "_id")]
        pub id: String,
        pub user: String,
        pub content: String,
        pub created_at: DateTime,
        pub updated_at: DateTime,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct PopulatedPost {
        #[serde(rename = "_id")]
        pub id: String,
        pub user: User,
        pub content: String,
        pub created_at: DateTime,
        pub updated_at: DateTime,
    }

    impl Default for Post {
        fn default() -> Self {
            let now = chrono::Utc::now();
            Self {
                id: Self::generate_nanoid(),
                user: String::new(),
                content: String::new(),
                created_at: now.into(),
                updated_at: now.into(),
            }
        }
    }

    impl Model for Post {}

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Log {
        #[serde(rename = "_id")]
        pub id: String,
        pub message: String,
        pub created_at: DateTime,
        pub updated_at: DateTime,
    }

    impl Default for Log {
        fn default() -> Self {
            let now = chrono::Utc::now();
            Self {
                id: Self::generate_nanoid(),
                message: String::new(),
                created_at: now.into(),
                updated_at: now.into(),
            }
        }
    }

    impl Model for Log {}

    pub fn nanoid() -> String {
        use nanoid::nanoid;
        nanoid!(
            20,
            &[
                'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p',
                'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '1', '2', '3', '4', '5', '6',
                '7', '8', '9', '0',
            ]
        )
    }

    pub fn number() -> u32 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen_range(0..99999)
    }

    fn hash_password(password: &str) -> String {
        use argon2::Config;
        use rand::{rngs::OsRng, RngCore};
        let mut salt = [0u8; 32];
        OsRng.fill_bytes(&mut salt);
        let config = Config::default();
        match argon2::hash_encoded(password.as_bytes(), &salt, &config) {
            Ok(hash) => hash,
            Err(err) => {
                tracing::error!("error hashing password {:?}", err);
                std::process::exit(1)
            }
        }
    }

    pub fn user() -> User {
        let bool = number() % 2 == 0;
        let username = format!("username_{}", nanoid());
        let email = format!("email+{}@mail.com", nanoid());
        let email = email.trim().to_lowercase();
        let digest = md5::compute(email.as_bytes());
        let avatar_hash = format!("{digest:?}");
        User {
            username: username.to_string(),
            slug: slug::slugify(username),
            password: hash_password("password"),
            email,
            avatar_hash,
            age: number(),
            example_array: (0..=2).map(|_| number()).collect::<Vec<_>>(),
            address: Address {
                address: number(),
                street: "Fake Street Name".to_string(),
                city: "Fake City".to_string(),
                state: "CA".to_string(),
                zip: "F1256".to_string(),
                country: "US".to_string(),
                apt_number: if bool { Some("F35".to_string()) } else { None },
            },
            ..Default::default()
        }
    }

    pub fn post(user_id: String) -> Post {
        Post {
            user: user_id,
            content: format!("here's my post: {}", nanoid()),
            ..Default::default()
        }
    }

    pub fn log() -> Log {
        Log {
            message: format!("[LOG_MESSAGE]: {}", nanoid()),
            ..Default::default()
        }
    }
}
