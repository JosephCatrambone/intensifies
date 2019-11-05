
use actix_web::{web, App, HttpRequest, HttpServer, Responder};

fn index(_req: HttpRequest) -> impl Responder {
	"Hello from the index page!"
}

fn hello(path: web::Path<String>) -> impl Responder {
	format!("Hello {}!", &path)
}

pub fn start_web_service() {
	App::new()
		.route("/", web::get().to(index))
		.route("/{name}", web::get().to(hello));
}
