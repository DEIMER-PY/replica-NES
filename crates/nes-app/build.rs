//! Incrusta el icono en el ejecutable de Windows, para que se vea en el Explorador,
//! la barra de tareas y los accesos directos. En el resto de sistemas no hace nada: el
//! icono de la ventana lo pone la app al arrancar (`main.rs`).

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("assets/icon.ico");
        if let Err(err) = resource.compile() {
            // Sin el compilador de recursos de Windows el ejecutable sale sin icono,
            // pero funciona igual: no merece romper la compilación.
            println!("cargo:warning=no se pudo incrustar el icono: {err}");
        }
    }
}
