#[catch(404)]
pub(crate) fn not_found(req: &rocket::Request) -> String {
    format!("Sorry, '{}' is not a valid path.", req.uri())
}