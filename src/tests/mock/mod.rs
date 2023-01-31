use self::{
    post_model::Post,
    user_model::{Address, User},
};

pub mod post_model;
pub mod user_model;

pub fn nanoid() -> String {
    use nanoid::nanoid;
    nanoid!(
        20,
        &[
            'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q',
            'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '1', '2', '3', '4', '5', '6', '7', '8',
            '9', '0',
        ]
    )
}

pub fn number() -> u32 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen_range(0..99999)
}

pub fn user() -> User {
    let bool = number() % 2 == 0;
    User {
        username: format!("username_{}", nanoid()),
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
