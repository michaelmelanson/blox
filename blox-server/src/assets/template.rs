use std::collections::HashMap;

use handlebars::Handlebars;

use blox_interpreter::Scope;

#[derive(Clone)]
pub enum Template {
    Handlebars(handlebars::Template),
}

impl Template {
    pub fn render(&self, scope: &Scope) -> Result<String, anyhow::Error> {
        match self {
            Template::Handlebars(template) => {
                let mut handlebars = Handlebars::new();
                handlebars.register_template("template", template.clone());

                let mut data = HashMap::new();
                for (identifier, value) in scope.bindings.read().unwrap().iter() {
                    data.insert(identifier.0.clone(), value.to_display_string());
                }

                handlebars
                    .render("template", &data)
                    .map_err(|err| Box::new(err).into())
            }
        }
    }
}

impl blox_assets::Asset for Template {
    const EXTENSIONS: &'static [&'static str] = &[".html.hbs"];
    type Loader = TemplateLoader;
}

pub struct TemplateLoader;

impl blox_assets::Loader<Template> for TemplateLoader {
    fn load(content: &[u8], filename: &str) -> Result<Template, anyhow::Error> {
        let input = String::from_utf8(content.to_vec())?;

        if filename.ends_with(".html.hbs") {
            let template = handlebars::Template::compile(&input)
                .map_err(|err| TemplateRenderError(format!("{}", err)))?;

            Ok(Template::Handlebars(template))
        } else {
            unimplemented!()
        }
    }
}

#[derive(Debug)]
pub struct TemplateRenderError(String);

impl std::fmt::Display for TemplateRenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("template render error: {}", self.0))
    }
}

impl std::error::Error for TemplateRenderError {}
