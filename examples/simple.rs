use route::Routes;

#[derive(Debug, Routes)]
pub enum MyRoutes {
    #[route("/")]
    Home,
    #[route("/about")]
    About,
    #[route("/users/{id}")]
    User { id: i32 },
    #[route("/posts/{year}/{month}/{day}/")]
    Posts { year: i32, month: u8, day: u8 },
}

fn main() {
    println!("home: {}", MyRoutes::Home.url());
    println!("about: {}", MyRoutes::About.url());
    println!("user: {}", MyRoutes::User { id: 32 }.url());
    println!(
        "posts: {}",
        MyRoutes::Posts {
            year: 2016,
            month: 1,
            day: 12,
        }
        .url()
    );
}
