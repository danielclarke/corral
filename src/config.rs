pub struct Config {
    input_dir: String,
    output_file: String,
}

impl Config {
    pub fn parse(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("Too few arguments, call like: `corral input_dir output_sheet.png`");
        }

        let input_dir = args[1].clone();
        let output_file = args[2].clone();

        Ok(Config {
            input_dir,
            output_file,
        })
    }
}