use std::{fmt, process::exit};

#[derive(Clone, Copy)]
pub enum MetaDataFormat {
    Json,
    Lua,
}

pub struct Config {
    pub padding: u8,
    pub input_dir: String,
    pub output_file: String,
    pub output_file_format: MetaDataFormat,
}

struct NamedArg<'a> {
    name: &'a str,
    value: Option<&'a str>,
}

impl fmt::Display for NamedArg<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self.value {
            Some(value) => {
                write!(
                    f,
                    "name: {name}, value: {value}",
                    name = self.name,
                    value = value
                )
            }
            None => {
                write!(f, "name: {name}", name = self.name,)
            }
        }
    }
}

struct NamedParam<'a> {
    name: &'a str,
    valid_values: Option<&'a [&'a str]>,
}

impl<'a> NamedParam<'a> {
    fn parse(&self, args: &[String]) -> Result<Option<NamedArg>, &'a str> {
        for arg in args {
            if let Some(index) = arg.find(&format!("--{name}", name = self.name)) {
                if index != 0 {
                    return Ok(None);
                }
                let invoked_arg: Vec<&str> = arg.split('=').collect();
                if invoked_arg.len() == 1 {
                    return Ok(Some(NamedArg {
                        name: self.name,
                        value: None,
                    }));
                }
                if invoked_arg.len() != 2 {
                    return Err("incorrect format");
                }
                match self.valid_values {
                    Some(valid_values) => {
                        for value in valid_values {
                            if invoked_arg[1] == *value {
                                return Ok(Some(NamedArg {
                                    name: self.name,
                                    value: Some(value),
                                }));
                            }
                        }
                    }
                    None => {
                        return Ok(Some(NamedArg {
                            name: self.name,
                            value: None,
                        }));
                    }
                }
            }
        }
        Ok(None)
    }
}

impl fmt::Display for NamedParam<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self.valid_values {
            Some(valid_values) => {
                write!(
                    f,
                    "[--{name}={values}]",
                    name = self.name,
                    values = valid_values.join("|")
                )
            }
            None => {
                write!(f, "[--{name}]", name = self.name,)
            }
        }
    }
}

impl Config {
    pub fn parse(args: &[String]) -> Result<Config, &'static str> {
        let named_params = [
            NamedParam {
                name: "data-fmt",
                valid_values: Some(&["json", "lua"]),
            },
            NamedParam {
                name: "help",
                valid_values: None,
            }, // NamedParam {name: "padding"}
        ];

        let mut metadata_format = MetaDataFormat::Json;

        for named_param in &named_params {
            if let Ok(Some(arg)) = named_param.parse(args) {
                match arg {
                    NamedArg {
                        name: "data-fmt",
                        value: Some("json"),
                    } => metadata_format = MetaDataFormat::Json,
                    NamedArg {
                        name: "data-fmt",
                        value: Some("lua"),
                    } => metadata_format = MetaDataFormat::Lua,
                    NamedArg {
                        name: "help",
                        value: None,
                    } => {
                        println!("A basic sprite sheet packer\n\nUsage: `corral input_dir output_sheet.png [--data-fmt=json|lua]");
                        exit(0);
                    }
                    _ => {
                        return Err("Unrecognised argument");
                    }
                }
            };
        }

        if args.len() < 3 {
            return Err("Too few arguments, Usage: `corral input_dir output_sheet.png [--data-fmt=json|lua]`");
        }

        let input_dir = args[1].clone();
        let output_file = args[2].clone();

        Ok(Config {
            padding: 2u8,
            input_dir,
            output_file,
            output_file_format: metadata_format,
        })
    }
}
