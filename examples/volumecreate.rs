use shiplift::{Docker, VolumeCreateOptions};
use std::env;

#[tokio::main]
async fn main() {
    let docker = Docker::new();
    let volumes = docker.volumes();

    let volume_name = env::args()
        .nth(1)
        .expect("You need to specify an volume name");

    match volumes
        .create(
            VolumeCreateOptions::default()
                .name(&volume_name)
                .label("com.github.softprops", "shiplift"),
        )
        .await
    {
        Ok(info) => println!("{:?}", info),
        Err(e) => eprintln!("Error: {}", e),
    }
}
