use html_escape::encode_safe;

pub fn render(tex: &str, display: bool) -> String {
    let body = render_fragment(tex.trim());
    let mode = if display { "display" } else { "inline" };
    format!("<span class=\"math-render math-{mode}\">{body}</span>")
}

fn render_fragment(input: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut index = 0;

    while index < chars.len() {
        match chars[index] {
            '\\' => {
                let (command, next) = read_command(&chars, index + 1);
                index = next;
                match command.as_str() {
                    "frac" => {
                        if let Some((num, next)) = read_group(&chars, index) {
                            if let Some((den, final_index)) = read_group(&chars, next) {
                                out.push_str("<span class=\"mfrac\"><span>");
                                out.push_str(&render_fragment(&num));
                                out.push_str("</span><span>");
                                out.push_str(&render_fragment(&den));
                                out.push_str("</span></span>");
                                index = final_index;
                                continue;
                            }
                        }
                        out.push_str("\\frac");
                    }
                    "sqrt" => {
                        if let Some((radicand, next)) = read_group(&chars, index) {
                            out.push_str("<span class=\"msqrt\"><span>");
                            out.push_str(&render_fragment(&radicand));
                            out.push_str("</span></span>");
                            index = next;
                            continue;
                        }
                        out.push_str("\\sqrt");
                    }
                    _ => out.push_str(symbol_for(&command).unwrap_or_else(|| command.as_str())),
                }
            }
            '^' | '_' => {
                let class_name = if chars[index] == '^' { "msup" } else { "msub" };
                if let Some((value, next)) = read_atom(&chars, index + 1) {
                    out.push_str("<span class=\"");
                    out.push_str(class_name);
                    out.push_str("\">");
                    out.push_str(&render_fragment(&value));
                    out.push_str("</span>");
                    index = next;
                } else {
                    out.push(chars[index]);
                    index += 1;
                }
            }
            '{' | '}' => {
                index += 1;
            }
            '&' | '<' | '>' | '"' | '\'' => {
                out.push_str(&encode_safe(&chars[index].to_string()));
                index += 1;
            }
            ch => {
                out.push(ch);
                index += 1;
            }
        }
    }

    out
}

fn read_command(chars: &[char], mut index: usize) -> (String, usize) {
    let start = index;
    while index < chars.len() && chars[index].is_ascii_alphabetic() {
        index += 1;
    }

    if start == index && index < chars.len() {
        index += 1;
    }

    (chars[start..index].iter().collect(), index)
}

fn read_atom(chars: &[char], index: usize) -> Option<(String, usize)> {
    if index >= chars.len() {
        return None;
    }
    if chars[index] == '{' {
        return read_group(chars, index);
    }
    Some((chars[index].to_string(), index + 1))
}

fn read_group(chars: &[char], index: usize) -> Option<(String, usize)> {
    if chars.get(index) != Some(&'{') {
        return None;
    }

    let mut depth = 0;
    for cursor in index..chars.len() {
        match chars[cursor] {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some((chars[index + 1..cursor].iter().collect(), cursor + 1));
                }
            }
            _ => {}
        }
    }

    None
}

fn symbol_for(command: &str) -> Option<&'static str> {
    Some(match command {
        "alpha" => "α",
        "beta" => "β",
        "gamma" => "γ",
        "delta" => "δ",
        "epsilon" => "ε",
        "theta" => "θ",
        "lambda" => "λ",
        "mu" => "μ",
        "pi" => "π",
        "sigma" => "σ",
        "phi" => "φ",
        "omega" => "ω",
        "Gamma" => "Γ",
        "Delta" => "Δ",
        "Theta" => "Θ",
        "Lambda" => "Λ",
        "Pi" => "Π",
        "Sigma" => "Σ",
        "Phi" => "Φ",
        "Omega" => "Ω",
        "times" => "×",
        "cdot" => "·",
        "pm" => "±",
        "le" | "leq" => "≤",
        "ge" | "geq" => "≥",
        "neq" => "≠",
        "approx" => "≈",
        "infty" => "∞",
        "sum" => "∑",
        "prod" => "∏",
        "int" => "∫",
        "partial" => "∂",
        "nabla" => "∇",
        "to" | "rightarrow" => "→",
        "leftarrow" => "←",
        "Rightarrow" => "⇒",
        "Leftarrow" => "⇐",
        _ => return None,
    })
}
