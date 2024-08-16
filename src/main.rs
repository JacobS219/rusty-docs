use regex::Regex;
use std::env;
use std::fs;
// use std::path::Path;
use std::collections::HashSet;

struct Function {
    name: String,
    arguments: String,
    line: usize,
    doc: Option<String>,
}

struct Class {
    name: String,
    functions: Vec<Function>,
    line: usize,
    doc: Option<String>,
}

fn extract_classes_and_functions(content: &str) -> (usize, Vec<Class>, Vec<Function>) {
    let mut classes = Vec::new();
    let mut functions = Vec::new();

    let class_re = Regex::new(r"class (\w+)").unwrap(); // for capturing class name
    let func_re = Regex::new(r"(\w+(?:\s*[\*\&]|\s*<\w+>)?\s+\w+)\s*\(([^)]*)\)").unwrap(); // for capturing function name and arguments
    let comment_re = Regex::new(r"/\*([\s\S]*?)\*/").unwrap(); // for capturing block /* */ comments
    let comment_with_slashes = Regex::new(r"^\s*//(.*)$").unwrap(); // for // comments

    let lines: Vec<&str> = content.lines().collect(); // split content into lines

    let line_count = lines.len(); // count the lines

    for (index, line) in lines.iter().enumerate() {
        if let Some(class_match) = class_re.captures(line) {
            let class_name = class_match[1].to_string();

            let mut comments = Vec::new();

            // Iterate backward from current line looking for in-line // comments
            let mut i = index;
            while i > 0 {
                if comment_with_slashes.is_match(lines[i - 1]) {
                    comments.push(
                        comment_with_slashes.captures(lines[i - 1]).unwrap()[1]
                            .to_string()
                            .trim()
                            .to_string(),
                    );
                    i -= 1;
                } else if comment_re.is_match(lines[i - 1]) {
                    comments.push(
                        comment_re.captures(lines[i - 1]).unwrap()[1]
                            .to_string()
                            .trim()
                            .to_string(),
                    );
                    i -= 1;
                } else {
                    break;
                }
            }

            // Reverse the comments, since we collected them in bottom-to-top order, but we want top-to-bottom order in the output
            comments.reverse();
            let doc_comment = if !comments.is_empty() {
                Some(comments.join("<br>")) // line html line break between comments
            } else {
                None
            };

            classes.push(Class {
                name: class_name,
                functions: Vec::new(),
                line: index + 1,
                doc: doc_comment,
            });
        }

        if let Some(func_match) = func_re.captures(line) {
            let func_name = func_match[1].to_string();
            let args = func_match[2].to_string();

            let mut comments = Vec::new();

            // Iterate backward from current line looking for in-line // comments
            let mut i = index;
            while i > 0 {
                if comment_with_slashes.is_match(lines[i - 1]) {
                    comments.push(
                        comment_with_slashes.captures(lines[i - 1]).unwrap()[1]
                            .to_string()
                            .trim()
                            .to_string(),
                    );
                    i -= 1;
                } else if comment_re.is_match(lines[i - 1]) {
                    comments.push(
                        comment_re.captures(lines[i - 1]).unwrap()[1]
                            .to_string()
                            .trim()
                            .to_string(),
                    );
                    i -= 1;
                } else {
                    break;
                }
            }

            // Reverse the comments, since we collected them in bottom-to-top order, but we want top-to-bottom order in the output
            comments.reverse();
            let doc_comment = if !comments.is_empty() {
                Some(comments.join("<br>")) // line html line break between comments
            } else {
                None
            };

            functions.push(Function {
                name: func_name,
                arguments: args,
                line: index + 1,
                doc: doc_comment,
            });
        }
    }

    (line_count, classes, functions)
}

fn count_operators_and_operands(content: &str) -> (usize, usize, usize, usize) {
    let operator_re = Regex::new(r"[+\-*/%=<>&|]+").unwrap();
    let operand_re = Regex::new(r"\b\w+\b").unwrap();

    let mut distinct_operators: HashSet<&str> = HashSet::new();
    let mut distinct_operands: HashSet<&str> = HashSet::new();

    let mut total_operators = 0;
    let mut total_operands = 0;

    for op_match in operator_re.find_iter(content) {
        distinct_operators.insert(op_match.as_str());
        total_operators += 1;
    }

    for opnd_match in operand_re.find_iter(content) {
        distinct_operands.insert(opnd_match.as_str());
        total_operands += 1;
    }

    (
        distinct_operators.len(),
        distinct_operands.len(),
        total_operators,
        total_operands,
    )
}

fn generate_html(
    file_name: &str,
    line_count: usize,
    classes: Vec<Class>,
    functions: Vec<Function>,
    difficulty: f64,
    effort: f64,
) -> String {
    let mut output = String::from("<html><body>");

    output.push_str(&format!(
        "<h1>{}</h1><h2>Line Count: {}</h2>",
        file_name, line_count
    ));

    output.push_str(&format!("<h3>Halstead Difficulty: {:.2}</h3>", difficulty));
    output.push_str(&format!("<h3>Halstead Effort: {:.2}</h3>", effort));

    for class in classes {
        output.push_str(&format!(
            "<h2>Class: {} - Line:{}</h2>",
            class.name, class.line
        ));
        for function in &class.functions {
            output.push_str(&format!(
                "<h3><strong>Function:</strong> {}</h3><p>{:?}</p>",
                function.name, function.doc
            ));
            output.push_str(&format!(
                "<p><strong>Comments:</strong> {}</p>",
                class.doc.as_ref().unwrap_or(&String::from(""))
            ));
        }
    }

    for function in functions {
        output.push_str(&format!(
            "<h3>Function: {} - Line:{}</h3>",
            function.name, function.line
        ));
        output.push_str(&format!(
            "<p><strong>Arguments:</strong> {}</p>",
            function.arguments
        ));
        output.push_str(&format!(
            "<p><strong>Comments:</strong> {}</p>",
            function.doc.as_ref().unwrap_or(&String::from(""))
        ));
    }

    output.push_str("</body></html>");

    output
}

fn read_files_recursively(directory_path: &str, html_output: &mut String) {
    for entry in fs::read_dir(directory_path).unwrap() {
        let entry_path = entry.unwrap().path();

        if entry_path.is_dir() {
            read_files_recursively(&entry_path.display().to_string(), html_output);
        } else {
            // Attempt to read the file. If it fails, skip this file.
            let file_name = entry_path.file_name().unwrap().to_str().unwrap();

            let maybe_content = fs::read_to_string(&entry_path);
            if let Ok(content) = maybe_content {
                let (line_count, classes, functions) = extract_classes_and_functions(&content);

                let (n1, n2, N1, N2) = count_operators_and_operands(&content);

                let difficulty = (n1 as f64 / 2.0) * (N2 as f64 / n2 as f64);

                let N = N1 + N2; // program length
                let volume = N as f64 * ((n1 + n2) as f64).log2(); // program volume


                let effort = (difficulty * volume) / 48.11; // effort

                *html_output +=
                    &generate_html(file_name, line_count, classes, functions, difficulty, effort);
            } else {
                println!("Warning: Unable to read file {:?}", entry_path);
            }
        }
    }
}

fn main() {
    let exe_path = env::current_exe().expect("Failed to get current exe path");
    let exe_dir = exe_path
        .parent()
        .expect("Failed to get directory of exe")
        .display()
        .to_string();

    let mut html_output = String::from("<html><body>");

    read_files_recursively(&exe_dir, &mut html_output);

    html_output.push_str("</body></html>");

    // Construct path for the output file in the same directory as the executable
    let output_path = exe_dir + "/output.html";

    fs::write(output_path, html_output).unwrap();
}

// fn read_files_recursively<P: AsRef<Path>>(path: P, html_output: &mut String) {
//     for entry in fs::read_dir(path).unwrap() {
//         let entry = entry.unwrap();
//         let path = entry.path();

//         if path.is_dir() {
//             read_files_recursively(&path, html_output);
//         } else if path.is_file() {
//             let content = fs::read_to_string(&path).unwrap();

//             let file_name = path.file_name().unwrap().to_str().unwrap();

//             let (line_count, classes, functions) = extract_classes_and_functions(&content);

//             let (n1, n2, N1, N2) = count_operators_and_operands(&content);

//             let difficulty = (n1 as f64 / 2.0) * (N2 as f64 / n2 as f64);

//             *html_output += &generate_html(file_name, line_count, classes, functions, difficulty);
//         }
//     }
// }

// fn main() {
//     let path = "C:/Users/jschi/source/repos/CStuff";
//     let mut html_output = String::from("<html><body>");

//     read_files_recursively(path, &mut html_output);

//     html_output.push_str("</body></html>");
//     fs::write("output.html", html_output).unwrap();
// }

// if let Some(func_match) = func_re.captures(line) {
//     let func_name = func_match[1].to_string();
//     let args = func_match[2].to_string();

//     // capture the previous comment (if any) as documentation
//     let doc_comment = if index > 0 && comment_re.is_match(lines[index - 1]) {
//         Some(comment_re.captures(lines[index - 1]).unwrap()[1].to_string())
//     } else if index > 0 && comment_with_slashes.is_match(lines[index - 1]) {
//         Some(comment_with_slashes.captures(lines[index - 1]).unwrap()[1].to_string().trim().to_string())
//     } else {
//         None
//     };

//     functions.push(Function {
//         name: func_name,
//         arguments: args,
//         line: index + 1,
//         doc: doc_comment,
//     });
// }
