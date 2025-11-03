use crate::lexer::Token;
use std::collections::HashMap;

pub struct MacroExpander {
    macros: HashMap<String, Macro>,
    expansion_depth: usize,
    max_depth: usize,
}

#[derive(Debug, Clone)]
pub struct Macro {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Vec<Token>,
    pub is_variadic: bool,
}

impl MacroExpander {
    pub fn new() -> Self {
        Self {
            macros: HashMap::new(),
            expansion_depth: 0,
            max_depth: 100,
        }
    }
    
    pub fn define_macro(&mut self, name: String, params: Vec<String>, body: Vec<Token>, is_variadic: bool) {
        let macro_def = Macro {
            name: name.clone(),
            parameters: params,
            body,
            is_variadic,
        };
        
        self.macros.insert(name, macro_def);
    }
    
    pub fn expand(&mut self, tokens: Vec<Token>) -> Result<Vec<Token>, String> {
        self.expansion_depth = 0;
        self.expand_recursive(tokens)
    }
    
    fn expand_recursive(&mut self, tokens: Vec<Token>) -> Result<Vec<Token>, String> {
        if self.expansion_depth > self.max_depth {
            return Err("Macro expansion depth exceeded".to_string());
        }
        
        let mut result = Vec::new();
        let mut i = 0;
        
        while i < tokens.len() {
            if let Some(macro_name) = self.get_macro_invocation(&tokens[i]) {
                if let Some(macro_def) = self.macros.get(&macro_name).cloned() {
                    let (args, consumed) = self.extract_arguments(&tokens[i+1..], &macro_def)?;
                    
                    self.expansion_depth += 1;
                    let expanded = self.substitute_parameters(&macro_def, &args)?;
                    let fully_expanded = self.expand_recursive(expanded)?;
                    self.expansion_depth -= 1;
                    
                    result.extend(fully_expanded);
                    i += consumed + 1;
                } else {
                    result.push(tokens[i].clone());
                    i += 1;
                }
            } else {
                result.push(tokens[i].clone());
                i += 1;
            }
        }
        
        Ok(result)
    }
    
    fn get_macro_invocation(&self, token: &Token) -> Option<String> {
        match &token.token_type {
            crate::lexer::TokenType::Ident(name) if self.macros.contains_key(name) => {
                Some(name.clone())
            }
            _ => None,
        }
    }
    
    fn extract_arguments(&self, tokens: &[Token], macro_def: &Macro) -> Result<(Vec<Vec<Token>>, usize), String> {
        if tokens.is_empty() || !matches!(tokens[0].token_type, crate::lexer::TokenType::LeftParen) {
            return Ok((Vec::new(), 0));
        }
        
        let mut args = Vec::new();
        let mut current_arg = Vec::new();
        let mut depth = 0;
        let mut i = 1;
        
        while i < tokens.len() {
            match &tokens[i].token_type {
                crate::lexer::TokenType::LeftParen => {
                    depth += 1;
                    current_arg.push(tokens[i].clone());
                }
                
                crate::lexer::TokenType::RightParen => {
                    if depth == 0 {
                        if !current_arg.is_empty() {
                            args.push(current_arg);
                        }
                        return Ok((args, i + 1));
                    }
                    depth -= 1;
                    current_arg.push(tokens[i].clone());
                }
                
                crate::lexer::TokenType::Comma if depth == 0 => {
                    args.push(current_arg);
                    current_arg = Vec::new();
                }
                
                _ => {
                    current_arg.push(tokens[i].clone());
                }
            }
            
            i += 1;
        }
        
        Err("Unclosed macro invocation".to_string())
    }
    
    fn substitute_parameters(&self, macro_def: &Macro, args: &[Vec<Token>]) -> Result<Vec<Token>, String> {
        if !macro_def.is_variadic && args.len() != macro_def.parameters.len() {
            return Err(format!(
                "Macro '{}' expects {} arguments, got {}",
                macro_def.name,
                macro_def.parameters.len(),
                args.len()
            ));
        }
        
        let mut param_map: HashMap<String, Vec<Token>> = HashMap::new();
        
        for (param, arg) in macro_def.parameters.iter().zip(args.iter()) {
            param_map.insert(param.clone(), arg.clone());
        }
        
        let mut result = Vec::new();
        
        for token in &macro_def.body {
            if let crate::lexer::TokenType::Ident(name) = &token.token_type {
                if let Some(replacement) = param_map.get(name) {
                    result.extend(replacement.clone());
                    continue;
                }
            }
            
            result.push(token.clone());
        }
        
        Ok(result)
    }
    
    pub fn has_macro(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }
    
    pub fn remove_macro(&mut self, name: &str) -> bool {
        self.macros.remove(name).is_some()
    }
    
    pub fn clear(&mut self) {
        self.macros.clear();
    }
}

impl Default for MacroExpander {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MacroParser {
    expander: MacroExpander,
}

impl MacroParser {
    pub fn new() -> Self {
        Self {
            expander: MacroExpander::new(),
        }
    }
    
    pub fn parse_macro_definition(&mut self, tokens: &[Token]) -> Result<(String, Macro), String> {
        let mut i = 0;
        
        if !matches!(tokens[i].token_type, crate::lexer::TokenType::Ident(ref s) if s == "macro") {
            return Err("Expected 'macro' keyword".to_string());
        }
        i += 1;
        
        let name = if let crate::lexer::TokenType::Ident(ref n) = tokens[i].token_type {
            n.clone()
        } else {
            return Err("Expected macro name".to_string());
        };
        i += 1;
        
        if !matches!(tokens[i].token_type, crate::lexer::TokenType::LeftParen) {
            return Err("Expected '(' after macro name".to_string());
        }
        i += 1;
        
        let mut params = Vec::new();
        let mut is_variadic = false;
        
        while i < tokens.len() && !matches!(tokens[i].token_type, crate::lexer::TokenType::RightParen) {
            if let crate::lexer::TokenType::Ident(ref param) = tokens[i].token_type {
                params.push(param.clone());
                i += 1;
                
                if matches!(tokens[i].token_type, crate::lexer::TokenType::Comma) {
                    i += 1;
                }
            } else {
                return Err("Expected parameter name".to_string());
            }
        }
        
        i += 1;
        
        if !matches!(tokens[i].token_type, crate::lexer::TokenType::LeftBrace) {
            return Err("Expected '{' after macro parameters".to_string());
        }
        i += 1;
        
        let mut body = Vec::new();
        let mut depth = 1;
        
        while i < tokens.len() && depth > 0 {
            match &tokens[i].token_type {
                crate::lexer::TokenType::LeftBrace => depth += 1,
                crate::lexer::TokenType::RightBrace => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            
            body.push(tokens[i].clone());
            i += 1;
        }
        
        let macro_def = Macro {
            name: name.clone(),
            parameters: params,
            body,
            is_variadic,
        };
        
        Ok((name, macro_def))
    }
    
    pub fn preprocess(&mut self, tokens: Vec<Token>) -> Result<Vec<Token>, String> {
        let mut result = Vec::new();
        let mut i = 0;
        
        while i < tokens.len() {
            if matches!(tokens[i].token_type, crate::lexer::TokenType::Ident(ref s) if s == "macro") {
                let (name, macro_def) = self.parse_macro_definition(&tokens[i..])?;
                self.expander.define_macro(
                    name,
                    macro_def.parameters,
                    macro_def.body,
                    macro_def.is_variadic,
                );
                
                while i < tokens.len() && !matches!(tokens[i].token_type, crate::lexer::TokenType::RightBrace) {
                    i += 1;
                }
                i += 1;
            } else {
                result.push(tokens[i].clone());
                i += 1;
            }
        }
        
        self.expander.expand(result)
    }
}

impl Default for MacroParser {
    fn default() -> Self {
        Self::new()
    }
}
