pub fn render_template<T: rocket::serde::Serialize>(
    name: &'static str,
    context: T,
) -> rocket_dyn_templates::Template {
    rocket_dyn_templates::Template::render(name, context)
}
