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
