use crate::{LexFile, DFA};

pub struct CodeGenerator {
    file: LexFile,
    dfa: DFA,
}

impl CodeGenerator {
    pub fn new(file: LexFile, dfa: DFA) -> Self {
        CodeGenerator { file, dfa }
    }

    pub fn generate_code(&self) -> String {
        let mut code = String::new();

        // Add user-defined code sections from the lexer file
        code.push_str(&self.generate_header());

        // Generate the transition table for the DFA
        code.push_str(&self.generate_transition_table());

        // Generate the token recognition logic with yytext and yyleng
        code.push_str(&self.generate_token_logic());

        // Add user-defined code from the lexer file
        code.push_str(&self.file.code);

        code
    }

    fn generate_header(&self) -> String {
        // Generate the header part of the lexer code
        // This includes standard includes, types, etc.
        let mut header = String::new();

        println!("{:?}", self.file.definitions_code);
        for line in &self.file.definitions_code {
            header.push_str(line);
            header.push_str("\n");
        }

        header.push_str("#include <string.h>\n");
        header.push_str("#include <stdio.h>\n");
        header.push_str("#include <stdlib.h>\n");
        header.push_str("\n");

        header.push_str("#define YY_BUFFER_SIZE 16384\n");
        header.push_str("\n");

        // Define yytext and yyleng as global variables
        header.push_str("char* yytext;\n");
        header.push_str("int   yyleng;\n");
        header.push_str("int   yylineno = 1;\n");
        header.push_str("int   yycolumn = 0;\n");
        header.push_str("FILE *yyin = NULL;\n");
        header.push_str("\n");

        header
    }

    fn generate_transition_table(&self) -> String {
        // Generate code for the DFA transition table
        let mut table_code = String::new();

        // Define state type
        table_code.push_str("typedef int StateID;\n");
        table_code.push_str("\n");

        // Generate the transition table as a 2D array or switch statement
        table_code.push_str("static StateID transition(StateID state, unsigned char c) {\n");
        table_code.push_str("    switch(state) {\n");

        // For each state, generate its transitions
        for state in &self.dfa.states {
            table_code.push_str(&format!("    case {}:\n", state));
            table_code.push_str("        switch(c) {\n");

            // Find all transitions from this state
            for ((from_state, symbol), to_state) in &self.dfa.transitions {
                if from_state == state {
                    if let crate::Symbol::Char(ch) = symbol {
                        // Use ASCII code instead of character literal
                        let ascii_code = *ch as u8;
                        table_code.push_str(&format!(
                            "            case {}: // {}\n",
                            ascii_code,
                            char_description(*ch)
                        ));
                        table_code.push_str(&format!("                return {};\n", to_state));
                    }
                }
            }

            table_code.push_str("            default:\n");
            table_code.push_str("                return -1; // Error state\n");
            table_code.push_str("        }\n");
        }

        table_code.push_str("    default:\n");
        table_code.push_str("        return -1; // Error state\n");
        table_code.push_str("    }\n");
        table_code.push_str("}\n");
        table_code.push_str("\n");

        // Generate final states check
        table_code.push_str("static int is_accepting(StateID state) {\n");
        table_code.push_str("    switch(state) {\n");

        for state in &self.dfa.final_states {
            table_code.push_str(&format!("    case {}:\n", state));
            table_code.push_str("        return 1;\n");
        }

        table_code.push_str("    default:\n");
        table_code.push_str("        return 0;\n");
        table_code.push_str("    }\n");
        table_code.push_str("}\n");
        table_code.push_str("\n");

        // Generate function to execute the correct action based on state
        table_code.push_str("static void execute_action(StateID state) {\n");
        table_code.push_str("    switch(state) {\n");

        for (state, action) in &self.dfa.actions {
            table_code.push_str(&format!("    case {}:\n", state));
            table_code.push_str(&format!("        {}\n", action));
            table_code.push_str("        break;\n");
        }

        table_code.push_str("    default:\n");
        table_code.push_str("        // No action for this state\n");
        table_code.push_str("        break;\n");
        table_code.push_str("    }\n");
        table_code.push_str("}\n");
        table_code.push_str("\n");

        table_code
    }

    fn generate_token_logic(&self) -> String {
        // Generate the token recognition and handling logic
        let mut logic = String::new();

        // Define yylex function which is the main scanning function
        logic.push_str("int yylex(void) {\n");
        logic.push_str("    static char *current_pos = NULL;\n");
        logic.push_str("    static char *buffer_end = NULL;\n");
        logic.push_str("    static char buffer[YY_BUFFER_SIZE];\n");
        logic.push_str("    char *token_start;\n");
        logic.push_str("\n");

        logic.push_str("    // Initialize buffer if first call\n");
        logic.push_str("    if (current_pos == NULL || current_pos >= buffer_end) {\n");
        logic.push_str("        if ((current_pos = buffer_end = buffer) == NULL)\n");
        logic.push_str("            return 0; // EOF\n");
        logic.push_str("        int n = fread(buffer, 1, YY_BUFFER_SIZE, yyin);\n");
        logic.push_str("        buffer_end = buffer + n;\n");
        logic.push_str("        if (n == 0) return 0; // EOF\n");
        logic.push_str("    }\n");
        logic.push_str("\n");

        logic.push_str("yylex_restart:\n");
        logic.push_str("    token_start = current_pos;\n");
        logic.push_str("    StateID current_state = 0; // Start state\n");
        logic.push_str("    StateID last_accepting_state = -1;\n");
        logic.push_str("    char *last_accepting_pos = NULL;\n");
        logic.push_str("\n");

        logic.push_str("    while (current_pos < buffer_end) {\n");
        logic.push_str("        unsigned char c = (unsigned char)*current_pos;\n");
        logic.push_str("        StateID next_state = transition(current_state, c);\n");
        logic.push_str("\n");

        logic.push_str("        if (next_state == -1) {\n");
        logic.push_str("            break; // No valid transition\n");
        logic.push_str("        }\n");
        logic.push_str("\n");

        logic.push_str("        current_state = next_state;\n");
        logic.push_str("        current_pos++;\n");
        logic.push_str("\n");

        logic.push_str("        // Update line and column counts\n");
        logic.push_str("        if (c == '\\n') {\n");
        logic.push_str("            yylineno++;\n");
        logic.push_str("            yycolumn = 0;\n");
        logic.push_str("        } else {\n");
        logic.push_str("            yycolumn++;\n");
        logic.push_str("        }\n");
        logic.push_str("\n");

        logic.push_str("        if (is_accepting(current_state)) {\n");
        logic.push_str("            last_accepting_state = current_state;\n");
        logic.push_str("            last_accepting_pos = current_pos;\n");
        logic.push_str("        }\n");
        logic.push_str("    }\n");
        logic.push_str("\n");

        logic.push_str("    if (last_accepting_state != -1) {\n");
        logic.push_str("        // Found a match - set up yytext and yyleng\n");
        logic.push_str("        yyleng = last_accepting_pos - token_start;\n");
        logic.push_str("        yytext = (char *)malloc(yyleng + 1);\n");
        logic.push_str("        if (!yytext) {\n");
        logic.push_str("            fprintf(stderr, \"Out of memory allocating yytext\\n\");\n");
        logic.push_str("            exit(1);\n");
        logic.push_str("        }\n");
        logic.push_str("        memcpy(yytext, token_start, yyleng);\n");
        logic.push_str("        yytext[yyleng] = '\\0';\n");
        logic.push_str("\n");

        logic.push_str("        // Reposition the current_pos to where we accepted\n");
        logic.push_str("        current_pos = last_accepting_pos;\n");
        logic.push_str("\n");

        logic.push_str("        // Execute the associated action\n");
        logic.push_str("        execute_action(last_accepting_state);\n");
        logic.push_str("\n");

        logic.push_str(
            "        // Free yytext before returning since the action should have consumed it\n",
        );
        logic.push_str("        free(yytext);\n");
        logic.push_str("        yytext = NULL;\n");
        logic.push_str("\n");

        logic.push_str("        // Return to get the next token\n");
        logic.push_str("        goto yylex_restart;\n");
        logic.push_str("    }\n");
        logic.push_str("\n");

        // Handle error case - skip invalid character
        logic.push_str("\n");
        logic.push_str("    if (current_pos < buffer_end) {\n");
        logic.push_str("        fprintf(stderr, \"Lexer error: Unexpected character '");
        logic.push_str("%c' (0x%02X) at line %d, column %d\\n\",\n");
        logic.push_str(
            "                (*current_pos >= 32 && *current_pos <= 126) ? *current_pos : '?',\n",
        );
        logic.push_str("                (unsigned char)*current_pos, yylineno, yycolumn);\n");
        logic.push_str("        current_pos++; // Skip invalid character\n");
        logic.push_str("        goto yylex_restart;\n");
        logic.push_str("    }\n");

        logic.push_str("    return 0; // EOF\n");
        logic.push_str("}\n");
        logic.push_str("\n");

        // Main function
        logic.push_str("int main(int argc, char *argv[]) {\n");
        logic.push_str("    FILE *input_file = stdin;\n");
        logic.push_str("\n");

        logic.push_str("    if (argc > 1) {\n");
        logic.push_str("        input_file = fopen(argv[1], \"r\");\n");
        logic.push_str("        if (!input_file) {\n");
        logic.push_str("            fprintf(stderr, \"Cannot open file %s\\n\", argv[1]);\n");
        logic.push_str("            return 1;\n");
        logic.push_str("        }\n");
        logic.push_str("    }\n");
        logic.push_str("\n");

        logic.push_str("    yyin = input_file;\n");
        logic.push_str("    yylex();\n");
        logic.push_str("\n");

        logic.push_str("    if (input_file != stdin) {\n");
        logic.push_str("        fclose(input_file);\n");
        logic.push_str("    }\n");
        logic.push_str("\n");

        logic.push_str("    return 0;\n");
        logic.push_str("}\n");
        logic.push_str("\n");

        logic
    }
}

// Helper function to get a readable description of a character
fn char_description(ch: char) -> String {
    match ch {
        '\n' => String::from("\\n (newline)"),
        '\r' => String::from("\\r (carriage return)"),
        '\t' => String::from("\\t (tab)"),
        ' ' => String::from("space"),
        '\x00'..='\x1F' | '\x7F' => format!("ASCII {:?} (control)", ch as u8),
        _ => format!("'{}'", ch),
    }
}
