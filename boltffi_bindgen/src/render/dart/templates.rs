use askama::Template;

#[derive(Template)]
#[template(path = "render_dart/enum.txt", escape = "none")]
pub struct EnhancedEnumTemplate<'a> {
    pub dart_enum: &'a super::DartEnum,
}
