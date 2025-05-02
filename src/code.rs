use crate::{LexFile, DFA};

pub struct CodeGenerator {
    file: LexFile,
    dfa: DFA,
}

impl CodeGenerator {
    pub fn new(file: LexFile, dfa: DFA) -> Self {        
        CodeGenerator { file, dfa }
    }

    pub fn code(&self) -> String {
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

        header.push_str("#include \"libl.h\"\n");
        header.push_str("#define YY_BUFFER_SIZE 16384\n");
        header.push_str("#define ECHO printf(\"%s\\n\", yytext)\n");
        header.push_str("#define REJECT do {  \\\n");
        header.push_str("    yy_rejected = 1; \\\n");
        header.push_str("    goto find_rule;  \\\n");
        header.push_str("} while (0)\n");
        header.push_str("\n");
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
                    if let crate::TransitionSymbol::Char(ch) = symbol {
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

        // Define global variables for proper REJECT functionality
        logic.push_str("// Global variables for REJECT and lexer state\n");
        logic.push_str("static int yy_rule_index = 0; // Current rule index\n");
        logic.push_str("static int yy_rule_count = 100; // Number of rules\n");
        logic.push_str("static int yy_rejected = 0; // Flag indicating REJECT was called\n");
        logic.push_str("static int yy_more_len = 0; // Length accumulated by yymore()\n");
        logic.push_str("\n");

        // Define yymore() functionality

        // Define yylex function which is the main scanning function
        logic.push_str("int yylex(void) {\n");
        logic.push_str("    static char *current_pos = NULL;\n");
        logic.push_str("    static char *buffer_end = NULL;\n");
        logic.push_str("    static char buffer[YY_BUFFER_SIZE];\n");
        logic.push_str("    static char *yytext_buffer = NULL;\n");
        logic.push_str("    static int yytext_buffer_size = 0;\n");
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
        logic.push_str("    yy_rule_index = 0; // Start with the first rule\n");
        logic.push_str("    yy_rejected = 0; // Reset REJECT flag\n");
        logic.push_str("    yy_more_len = 0; // Reset yymore() accumulation\n");
        logic.push_str("\n");

        logic.push_str("find_rule:\n");
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
        logic.push_str("\n");
        
        // Improved yytext allocation with support for yymore()
        logic.push_str("        // Allocate or reallocate yytext buffer if needed\n");
        logic.push_str("        int total_len = yy_more_len + yyleng;\n");
        logic.push_str("        if (yytext_buffer == NULL || total_len + 1 > yytext_buffer_size) {\n");
        logic.push_str("            int new_size = total_len + 1 + 128; // Some extra space\n");
        logic.push_str("            if (yytext_buffer) {\n");
        logic.push_str("                yytext_buffer = (char *)realloc(yytext_buffer, new_size);\n");
        logic.push_str("            } else {\n");
        logic.push_str("                yytext_buffer = (char *)malloc(new_size);\n");
        logic.push_str("            }\n");
        logic.push_str("            yytext_buffer_size = new_size;\n");
        logic.push_str("        }\n");
        logic.push_str("\n");
        logic.push_str("        if (!yytext_buffer) {\n");
        logic.push_str("            fprintf(stderr, \"Out of memory allocating yytext\\n\");\n");
        logic.push_str("            exit(1);\n");
        logic.push_str("        }\n");
        logic.push_str("\n");
        logic.push_str("        // Copy new text to yytext (after any text kept by yymore())\n");
        logic.push_str("        memcpy(yytext_buffer + yy_more_len, token_start, yyleng);\n");
        logic.push_str("        yytext_buffer[total_len] = '\\0';\n");
        logic.push_str("        yytext = yytext_buffer;\n");
        logic.push_str("        yyleng = total_len; // Update yyleng to include yymore text\n");
        logic.push_str("\n");

        logic.push_str("        // Reposition the current_pos to where we accepted\n");
        logic.push_str("        current_pos = last_accepting_pos;\n");
        logic.push_str("\n");

        logic.push_str("        // Execute the associated action\n");
        logic.push_str("        execute_action(last_accepting_state);\n");
        logic.push_str("\n");
        
        // Handle REJECT - try other rules
        logic.push_str("        // Handle REJECT macro effects\n");
        logic.push_str("        if (yy_rejected) {\n");
        logic.push_str("            // When REJECT is called, try the next rule\n");
        logic.push_str("            yy_rule_index++; // Try next rule\n");
        logic.push_str("            yy_rejected = 0;\n");
        logic.push_str("\n");
        logic.push_str("            // If we've exhausted all rules\n");
        logic.push_str("            if (yy_rule_index >= yy_rule_count) {\n");
        logic.push_str("                fprintf(stderr, \"REJECT failed - no more rules to try\\n\");\n");
        logic.push_str("                // Reset and continue as if no match\n");
        logic.push_str("                current_pos = token_start + 1;\n");
        logic.push_str("                goto yylex_restart;\n");
        logic.push_str("            }\n");
        logic.push_str("\n");
        logic.push_str("            // Reset position and try next rule\n");
        logic.push_str("            current_pos = token_start;\n");
        logic.push_str("            goto find_rule;\n");
        logic.push_str("        }\n");
        logic.push_str("\n");

        // Reset yytext and more_len if not using REJECT or yymore
        logic.push_str("        // Reset yytext and yymore state for next token\n");
        logic.push_str("        yy_more_len = 0;\n");
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
        logic.push_str("\n");

        logic.push_str("    free(yytext);\n");
        logic.push_str("    yytext = NULL;\n");

        logic.push_str("    return 0; // EOF\n");
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