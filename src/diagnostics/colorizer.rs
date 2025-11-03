use std::fmt;

#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub error: AnsiColor,
    pub warning: AnsiColor,
    pub info: AnsiColor,
    pub hint: AnsiColor,
    pub code: AnsiColor,
    pub highlight: AnsiColor,
    pub line_number: AnsiColor,
}

#[derive(Debug, Clone, Copy)]
pub struct AnsiColor {
    pub fg: u8,
    pub bg: Option<u8>,
    pub bold: bool,
    pub underline: bool,
}

pub struct Colorizer {
    enabled: bool,
    scheme: ColorScheme,
}

impl Colorizer {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            scheme: ColorScheme::default(),
        }
    }
    
    pub fn with_scheme(mut self, scheme: ColorScheme) -> Self {
        self.scheme = scheme;
        self
    }
    
    pub fn colorize(&self, text: &str, color: AnsiColor) -> String {
        if !self.enabled {
            return text.to_string();
        }
        
        let mut codes = Vec::new();
        
        if color.bold {
            codes.push("1".to_string());
        }
        
        if color.underline {
            codes.push("4".to_string());
        }
        
        codes.push(format!("38;5;{}", color.fg));
        
        if let Some(bg) = color.bg {
            codes.push(format!("48;5;{}", bg));
        }
        
        format!("\x1b[{}m{}\x1b[0m", codes.join(";"), text)
    }
    
    pub fn error(&self, text: &str) -> String {
        self.colorize(text, self.scheme.error)
    }
    
    pub fn warning(&self, text: &str) -> String {
        self.colorize(text, self.scheme.warning)
    }
    
    pub fn info(&self, text: &str) -> String {
        self.colorize(text, self.scheme.info)
    }
    
    pub fn hint(&self, text: &str) -> String {
        self.colorize(text, self.scheme.hint)
    }
    
    pub fn code(&self, text: &str) -> String {
        self.colorize(text, self.scheme.code)
    }
    
    pub fn highlight(&self, text: &str) -> String {
        self.colorize(text, self.scheme.highlight)
    }
    
    pub fn line_number(&self, num: usize) -> String {
        self.colorize(&num.to_string(), self.scheme.line_number)
    }
    
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    
    pub fn disable(&mut self) {
        self.enabled = false;
    }
    
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl ColorScheme {
    pub fn dark() -> Self {
        Self {
            error: AnsiColor::red().bold(),
            warning: AnsiColor::yellow().bold(),
            info: AnsiColor::blue(),
            hint: AnsiColor::cyan(),
            code: AnsiColor::white(),
            highlight: AnsiColor::magenta().bold(),
            line_number: AnsiColor::dark_gray(),
        }
    }
    
    pub fn light() -> Self {
        Self {
            error: AnsiColor::new(160).bold(),
            warning: AnsiColor::new(172).bold(),
            info: AnsiColor::new(27),
            hint: AnsiColor::new(39),
            code: AnsiColor::new(235),
            highlight: AnsiColor::new(125).bold(),
            line_number: AnsiColor::new(243),
        }
    }
    
    pub fn no_color() -> Self {
        let plain = AnsiColor::white();
        Self {
            error: plain,
            warning: plain,
            info: plain,
            hint: plain,
            code: plain,
            highlight: plain,
            line_number: plain,
        }
    }
}

impl AnsiColor {
    pub fn new(fg: u8) -> Self {
        Self {
            fg,
            bg: None,
            bold: false,
            underline: false,
        }
    }
    
    pub fn with_bg(mut self, bg: u8) -> Self {
        self.bg = Some(bg);
        self
    }
    
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }
    
    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }
    
    pub fn black() -> Self { Self::new(0) }
    pub fn red() -> Self { Self::new(9) }
    pub fn green() -> Self { Self::new(10) }
    pub fn yellow() -> Self { Self::new(11) }
    pub fn blue() -> Self { Self::new(12) }
    pub fn magenta() -> Self { Self::new(13) }
    pub fn cyan() -> Self { Self::new(14) }
    pub fn white() -> Self { Self::new(15) }
    pub fn dark_gray() -> Self { Self::new(8) }
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Default for Colorizer {
    fn default() -> Self {
        Self::new(true)
    }
}
