mod parser;
pub mod renderer;

use anyhow::Result;
use renderer::language::Language;

pub fn generate(lang: Language, generator: &str) -> Result<String> {
    let stub = parser::parse_generator_stub(generator.to_string());

    // eprint!("=======\n{:?}\n======", generator);
    eprint!("=======\n{}\n======\n", renderer::render_stub(lang.clone(), stub.clone(), true)?);
    // eprint!("=======\n{:?}\n======", stub);

    let output_str = renderer::render_stub(lang.clone(), stub, false)?;

    Ok(output_str.as_str().trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_code_generation() {
        let generator = "read m:int n:int\nwrite result";
        let received = generate(String::from("ruby"), generator).unwrap();
        let expected = "m, n = gets.split.map(&:to_i)\nputs \"result\"";

        assert_eq!(received, expected);
    }
}
