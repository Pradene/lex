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
        header.push_str("static int yy_rejected = 0; // Flag indicating REJECT was called\n");
        header.push_str("#define REJECT do {  \\\n");
        header.push_str("    yy_rejected = 1; \\\n");
        header.push_str("    return ;  \\\n");
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

        // Structure to associate state with pattern/rule ID
        table_code.push_str("struct PatternInfo {\n");
        table_code.push_str("    int pattern_id;\n");
        table_code.push_str("    int priority;\n");
        table_code.push_str("};\n\n");

        // Generate mapping from state to rule/pattern ID and priority
        table_code.push_str("static struct PatternInfo get_pattern_info(StateID state) {\n");
        table_code.push_str("    struct PatternInfo info = {-1, -1};\n");
        table_code.push_str("    switch(state) {\n");

        // Assign a pattern ID and priority for each final state
        // Priority should be based on the rule order (earlier rules have higher priority)
        let mut pattern_id = 0;
        for state in &self.dfa.final_states {
            table_code.push_str(&format!("    case {}:\n", state));
            table_code.push_str(&format!("        info.pattern_id = {};\n", pattern_id));
            // Higher priority for earlier patterns (reverse of pattern_id)
            table_code.push_str(&format!("        info.priority = {};\n", self.dfa.final_states.len() - pattern_id));
            table_code.push_str("        break;\n");
            pattern_id += 1;
        }

        table_code.push_str("    }\n");
        table_code.push_str("    return info;\n");
        table_code.push_str("}\n\n");

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
        logic.push_str("static int yy_current_pattern_id = -1;  // Current pattern being matched\n");
        logic.push_str("static int yy_starting_state = 0;      // DFA start state\n");
        logic.push_str("static int yy_more_len = 0;            // Length accumulated by yymore()\n");
        logic.push_str("static char *yy_current_token_start = NULL; // Start of current token\n");
        logic.push_str("\n");

        // Define data structures for tracking matched patterns
        logic.push_str("// Maximum number of patterns that could match a token\n");
        logic.push_str("#define MAX_MATCHES 100\n");
        logic.push_str("\n");
        
        logic.push_str("// Structure to track matched patterns\n");
        logic.push_str("typedef struct {\n");
        logic.push_str("    StateID state;      // The accepting state\n");
        logic.push_str("    int pattern_id;     // Pattern ID for this match\n");
        logic.push_str("    int priority;       // Priority of this pattern\n");
        logic.push_str("    int length;         // Length of this match\n");
        logic.push_str("    char *text_position; // Position in the input where match occurred\n");
        logic.push_str("} Match;\n");
        logic.push_str("\n");
        
        logic.push_str("// Array to hold all matches for the current token\n");
        logic.push_str("static Match yy_matches[MAX_MATCHES];\n");
        logic.push_str("static int yy_match_count = 0;  // Number of patterns that matched\n");
        logic.push_str("static int yy_match_index = 0;  // Current match being processed\n");
        logic.push_str("\n");

        // Define function to add a match to our collection
        logic.push_str("// Function to add a match to our collection\n");
        logic.push_str("static void add_match(StateID state, char *pos) {\n");
        logic.push_str("    if (yy_match_count < MAX_MATCHES) {\n");
        logic.push_str("        struct PatternInfo info = get_pattern_info(state);\n");
        logic.push_str("        if (info.pattern_id != -1) {\n");
        logic.push_str("            yy_matches[yy_match_count].state = state;\n");
        logic.push_str("            yy_matches[yy_match_count].pattern_id = info.pattern_id;\n");
        logic.push_str("            yy_matches[yy_match_count].priority = info.priority;\n");
        logic.push_str("            yy_matches[yy_match_count].length = pos - yy_current_token_start;\n");
        logic.push_str("            yy_matches[yy_match_count].text_position = pos;\n");
        logic.push_str("            yy_match_count++;\n");
        logic.push_str("        }\n");
        logic.push_str("    } else {\n");
        logic.push_str("        fprintf(stderr, \"Too many matches for token, increase MAX_MATCHES\\n\");\n");
        logic.push_str("    }\n");
        logic.push_str("}\n");
        logic.push_str("\n");
        
        // Function to compare matches for sorting
        logic.push_str("// Function to compare matches for sorting by length, then priority\n");
        logic.push_str("static int compare_matches(const void *a, const void *b) {\n");
        logic.push_str("    const Match *m1 = (const Match *)a;\n");
        logic.push_str("    const Match *m2 = (const Match *)b;\n");
        logic.push_str("    // First compare by length (longest match first)\n");
        logic.push_str("    if (m1->length != m2->length) {\n");
        logic.push_str("        return m2->length - m1->length;\n");
        logic.push_str("    }\n");
        logic.push_str("    // Then by priority (highest priority first)\n");
        logic.push_str("    return m2->priority - m1->priority;\n");
        logic.push_str("}\n");
        logic.push_str("\n");

        // Define yymore() functionality
        logic.push_str("// Implementation of yymore() function\n");
        logic.push_str("#define yymore() do { \\\n");
        logic.push_str("    yy_more_len = yyleng; \\\n");
        logic.push_str("} while (0)\n");
        logic.push_str("\n");

        // Define yylex function which is the main scanning function
        logic.push_str("int yylex(void) {\n");
        logic.push_str("    static char *current_pos = NULL;\n");
        logic.push_str("    static char *buffer_end = NULL;\n");
        logic.push_str("    static char buffer[YY_BUFFER_SIZE];\n");
        logic.push_str("    static char *yytext_buffer = NULL;\n");
        logic.push_str("    static int yytext_buffer_size = 0;\n");
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

        logic.push_str("scan_token:\n");
        logic.push_str("    // Reset match tracking for a new token\n");
        logic.push_str("    yy_match_count = 0;\n");
        logic.push_str("    yy_match_index = 0;\n");
        logic.push_str("    yy_current_token_start = current_pos;\n");
        logic.push_str("\n");
        logic.push_str("    // Run the DFA to find all potential matches\n");
        logic.push_str("    char *scan_pos = current_pos;\n");
        logic.push_str("    StateID current_state = yy_starting_state;\n");
        logic.push_str("\n");
        logic.push_str("    while (scan_pos < buffer_end) {\n");
        logic.push_str("        unsigned char c = (unsigned char)*scan_pos;\n");
        logic.push_str("        StateID next_state = transition(current_state, c);\n");
        logic.push_str("\n");
        logic.push_str("        if (next_state == -1) {\n");
        logic.push_str("            break; // No valid transition\n");
        logic.push_str("        }\n");
        logic.push_str("\n");
        logic.push_str("        current_state = next_state;\n");
        logic.push_str("        scan_pos++;\n");
        logic.push_str("\n");
        logic.push_str("        // If we've reached an accepting state, record this match\n");
        logic.push_str("        if (is_accepting(current_state)) {\n");
        logic.push_str("            add_match(current_state, scan_pos);\n");
        logic.push_str("        }\n");
        logic.push_str("    }\n");
        logic.push_str("\n");

        logic.push_str("    // If we found matches, sort them by length and priority\n");
        logic.push_str("    if (yy_match_count > 0) {\n");
        logic.push_str("        qsort(yy_matches, yy_match_count, sizeof(Match), compare_matches);\n");
        logic.push_str("\n");
        logic.push_str("        // Process each match in order until one is not REJECTed\n");
        logic.push_str("process_match:\n");
        logic.push_str("        // If we've tried all matches, move to the next character and try again\n");
        logic.push_str("        if (yy_match_index >= yy_match_count) {\n");
        logic.push_str("            if (current_pos < buffer_end) {\n");
        logic.push_str("                fprintf(stderr, \"All matches REJECTed, skipping character '%c'\\n\", *current_pos);\n");
        logic.push_str("                current_pos++;\n");
        logic.push_str("                goto scan_token;\n");
        logic.push_str("            } else {\n");
        logic.push_str("                // End of buffer, no more tokens\n");
        logic.push_str("                return 0;\n");
        logic.push_str("            }\n");
        logic.push_str("        }\n");
        logic.push_str("\n");

        logic.push_str("        // Get the current match to process\n");
        logic.push_str("        Match *match = &yy_matches[yy_match_index];\n");
        logic.push_str("        yy_current_pattern_id = match->pattern_id;\n");
        logic.push_str("\n");
        logic.push_str("        // Set up yytext and yyleng based on this match\n");
        logic.push_str("        yyleng = match->length;\n");
        logic.push_str("\n");
        logic.push_str("        // Allocate or reallocate yytext buffer if needed\n");
        logic.push_str("        int total_len = yy_more_len + yyleng;\n");
        logic.push_str("        if (yytext_buffer == NULL || total_len + 1 > yytext_buffer_size) {\n");
        logic.push_str("            int new_size = total_len + 1;\n");
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
        logic.push_str("        memcpy(yytext_buffer + yy_more_len, current_pos, yyleng);\n");
        logic.push_str("        yytext_buffer[total_len] = '\\0';\n");
        logic.push_str("        yytext = yytext_buffer;\n");
        logic.push_str("        yyleng = total_len; // Update yyleng to include yymore text\n");
        logic.push_str("\n");

        logic.push_str("        // Execute the associated action\n");
        logic.push_str("        yy_rejected = 0;  // Reset REJECT flag before action\n");
        logic.push_str("        execute_action(match->state);\n");
        logic.push_str("\n");
        
        logic.push_str("        // If action called REJECT, try the next match\n");
        logic.push_str("        if (yy_rejected) {\n");
        logic.push_str("            yy_match_index++;\n");
        logic.push_str("            goto process_match;\n");
        logic.push_str("        }\n");
        logic.push_str("\n");
        
        logic.push_str("        // Update the current position to after the matched text\n");
        logic.push_str("        current_pos = current_pos + match->length;\n");
        logic.push_str("\n");
        
        logic.push_str("        // Reset yymore state for next token (unless yymore() was called)\n");
        logic.push_str("        if (!yy_more_len) {\n");
        logic.push_str("            yy_current_pattern_id = -1;\n");
        logic.push_str("        } else {\n");
        logic.push_str("            // yymore() was called, keep accumulated text\n");
        logic.push_str("            // (yy_more_len is already set by yymore macro)\n");
        logic.push_str("        }\n");
        logic.push_str("\n");
        
        logic.push_str("        // Scan for the next token\n");
        logic.push_str("        goto scan_token;\n");
        logic.push_str("    }\n");
        logic.push_str("\n");

        // Handle case where no match was found
        logic.push_str("    // No match found - either EOF or an error\n");
        logic.push_str("    if (current_pos < buffer_end) {\n");
        logic.push_str("        // Print error for unrecognized character\n");
        logic.push_str("        fprintf(stderr, \"Lexer error: Unexpected character '");
        logic.push_str("%c' (0x%02X) at line %d, column %d\\n\",\n");
        logic.push_str(
            "                (*current_pos >= 32 && *current_pos <= 126) ? *current_pos : '?',\n",
        );
        logic.push_str("                (unsigned char)*current_pos, yylineno, yycolumn);\n");
        logic.push_str("\n");
        
        logic.push_str("        // Update line/column tracking\n");
        logic.push_str("        if (*current_pos == '\\n') {\n");
        logic.push_str("            yylineno++;\n");
        logic.push_str("            yycolumn = 0;\n");
        logic.push_str("        } else {\n");
        logic.push_str("            yycolumn++;\n");
        logic.push_str("        }\n");
        logic.push_str("\n");
        
        logic.push_str("        // Skip invalid character and continue\n");
        logic.push_str("        current_pos++;\n");
        logic.push_str("        goto scan_token;\n");
        logic.push_str("    }\n");
        logic.push_str("\n");

        logic.push_str("    // Clean up at EOF\n");
        logic.push_str("    if (yytext_buffer) {\n");
        logic.push_str("        free(yytext_buffer);\n");
        logic.push_str("        yytext_buffer = NULL;\n");
        logic.push_str("        yytext = NULL;\n");
        logic.push_str("    }\n");
        logic.push_str("\n");
        
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