use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;

#[pyfunction]
fn lexer(code: &str) -> PyResult<Vec<(String, String)>> {
    let mut tokens = Vec::new();
    let mut chars = code.char_indices().peekable();

    while let Some((_, c)) = chars.next() {
        match c {
            ' ' | '\t' | '\n' => continue,
            '=' => tokens.push(("ASSIGN".into(), "=".into())),
            '+' => tokens.push(("OP".into(), "+".into())),
            '"' => {
                let mut s = String::from("\"");
                loop {
                    match chars.next() {
                        Some((_, '"')) => { s.push('"'); break; }
                        Some((_, ch)) => s.push(ch),
                        None => return Err(PyRuntimeError::new_err("Unterminated string")),
                    }
                }
                tokens.push(("STRING".into(), s));
            }
            c if c.is_alphabetic() || c == '_' => {
                let mut word = String::from(c);
                while let Some(&(_, nc)) = chars.peek() {
                    if nc.is_alphanumeric() || nc == '_' {
                        word.push(nc);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let kind = match word.as_str() {
                    "print" => "PRINT",
                    "let"   => "LET",
                    _       => "ID",
                };
                tokens.push((kind.into(), word));
            }
            other => {
                return Err(PyRuntimeError::new_err(format!("Unexpected character: {other}")));
            }
        }
    }

    Ok(tokens)
}



#[pymodule]
fn phplus(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(lexer, m)?)?;
    m.add_function(wrap_pyfunction!(transpile, m)?)?;
    Ok(())
}