use std::cell::Cell;
use std::fmt;
use std::str::FromStr;

pub type Value = f64;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub position: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error at {}: {}", self.position, self.message)
    }
}

impl std::error::Error for ParseError {}

#[derive(Clone, Copy, Debug)]
pub struct Variable<'a> {
    pub name: &'a str,
    pub value: &'a Cell<Value>,
}

pub struct Expr<'a> {
    root: Node,
    variables: Vec<&'a Cell<Value>>,
}

impl<'a> Expr<'a> {
    pub fn eval(&self) -> Value {
        self.root.evaluate(&self.variables)
    }

    pub fn print(&self) -> String {
        self.root.format(0)
    }
}

pub fn te_interp(expression: &str) -> Result<Value, ParseError> {
    let expr = te_compile(expression, &[])?;
    Ok(expr.eval())
}

pub fn te_compile<'a>(expression: &'a str, variables: &'a [Variable<'a>]) -> Result<Expr<'a>, ParseError> {
    let mut parser = Parser::new(expression, variables);
    let node = parser.parse_list()?;
    if parser.token.kind != TokenKind::End {
        return Err(parser.error("unexpected characters after expression"));
    }
    Ok(Expr { root: node, variables: parser.variables })
}

#[derive(Clone)]
enum Node {
    Constant(Value),
    Variable(usize),
    UnaryOp(UnaryOp, Box<Node>),
    BinaryOp(BinaryOp, Box<Node>, Box<Node>),
    Function(Function, Vec<Node>),
    CommaList(Vec<Node>),
}

#[derive(Clone, Copy, Debug)]
enum UnaryOp {
    Plus,
    Minus,
}

#[derive(Clone, Copy, Debug)]
enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
}

#[derive(Clone)]
struct Function {
    name: &'static str,
    arity: usize,
    callback: fn(&[Value]) -> Value,
    pure: bool,
}

impl Function {
    fn apply(&self, args: &[Value]) -> Value {
        (self.callback)(args)
    }
}

impl Node {
    fn evaluate(&self, variables: &[&Cell<Value>]) -> Value {
        match self {
            Node::Constant(value) => *value,
            Node::Variable(index) => variables[*index].get(),
            Node::UnaryOp(op, child) => match op {
                UnaryOp::Plus => child.evaluate(variables),
                UnaryOp::Minus => -child.evaluate(variables),
            },
            Node::BinaryOp(op, left, right) => {
                let a = left.evaluate(variables);
                let b = right.evaluate(variables);
                match op {
                    BinaryOp::Add => a + b,
                    BinaryOp::Sub => a - b,
                    BinaryOp::Mul => a * b,
                    BinaryOp::Div => a / b,
                    BinaryOp::Mod => a % b,
                    BinaryOp::Pow => a.powf(b),
                }
            }
            Node::Function(function, args) => {
                let values: Vec<Value> = args.iter().map(|child| child.evaluate(variables)).collect();
                function.apply(&values)
            }
            Node::CommaList(elements) => {
                let mut result = 0.0;
                for element in elements {
                    result = element.evaluate(variables);
                }
                result
            }
        }
    }

    fn format(&self, indent: usize) -> String {
        let prefix = "  ".repeat(indent);
        match self {
            Node::Constant(value) => format!("{}Constant({})", prefix, value),
            Node::Variable(index) => format!("{}Variable({})", prefix, index),
            Node::UnaryOp(op, child) => format!("{}Unary({:?})\n{}", prefix, op, child.format(indent + 1)),
            Node::BinaryOp(op, left, right) => format!("{}Binary({:?})\n{}\n{}", prefix, op, left.format(indent + 1), right.format(indent + 1)),
            Node::Function(function, args) => {
                let mut output = format!("{}Function({})\n", prefix, function.name);
                for arg in args {
                    output.push_str(&format!("{}\n", arg.format(indent + 1)));
                }
                output
            }
            Node::CommaList(elements) => {
                let mut output = format!("{}CommaList\n", prefix);
                for element in elements {
                    output.push_str(&format!("{}\n", element.format(indent + 1)));
                }
                output
            }
        }
    }

    fn optimize(self) -> Node {
        match self {
            Node::BinaryOp(op, left, right) => {
                let left = left.optimize();
                let right = right.optimize();
                if let (Node::Constant(a), Node::Constant(b)) = (&left, &right) {
                    let value = match op {
                        BinaryOp::Add => a + b,
                        BinaryOp::Sub => a - b,
                        BinaryOp::Mul => a * b,
                        BinaryOp::Div => a / b,
                        BinaryOp::Mod => a % b,
                        BinaryOp::Pow => a.powf(*b),
                    };
                    Node::Constant(value)
                } else {
                    Node::BinaryOp(op, Box::new(left), Box::new(right))
                }
            }
            Node::UnaryOp(op, child) => {
                let child = child.optimize();
                if let Node::Constant(value) = child {
                    let value = match op {
                        UnaryOp::Plus => value,
                        UnaryOp::Minus => -value,
                    };
                    Node::Constant(value)
                } else {
                    Node::UnaryOp(op, Box::new(child))
                }
            }
            Node::Function(function, args) => {
                let args: Vec<_> = args.into_iter().map(|arg| arg.optimize()).collect();
                if function.pure && args.iter().all(|arg| matches!(arg, Node::Constant(_))) {
                    let values: Vec<Value> = args.iter().map(|arg| match arg { Node::Constant(value) => *value, _ => unreachable!() }).collect();
                    Node::Constant(function.apply(&values))
                } else {
                    Node::Function(function, args)
                }
            }
            Node::CommaList(elements) => {
                let flattened: Vec<_> = elements.into_iter().map(|child| child.optimize()).collect();
                if flattened.len() == 1 {
                    flattened.into_iter().next().unwrap()
                } else {
                    Node::CommaList(flattened)
                }
            }
            other => other,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TokenKind {
    End,
    Number,
    Identifier,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
    Caret,
    LParen,
    RParen,
    Comma,
    Unknown,
}

struct Token {
    kind: TokenKind,
    text: String,
    value: Value,
    position: usize,
}

struct Parser<'a> {
    input: &'a str,
    position: usize,
    token: Token,
    variables: Vec<&'a Cell<Value>>,
    variable_names: Vec<String>,
    lookup: &'a [Variable<'a>],
}

impl<'a> Parser<'a> {
    fn new(input: &'a str, lookup: &'a [Variable<'a>]) -> Self {
        let mut parser = Parser {
            input,
            position: 0,
            token: Token { kind: TokenKind::End, text: String::new(), value: 0.0, position: 0 },
            variables: Vec::new(),
            variable_names: Vec::new(),
            lookup,
        };
        parser.advance_token();
        parser
    }

    fn parse_list(&mut self) -> Result<Node, ParseError> {
        let mut elements = vec![self.parse_expr()?];
        while self.token.kind == TokenKind::Comma {
            self.advance_token();
            elements.push(self.parse_expr()?);
        }
        let node = if elements.len() == 1 { elements.remove(0) } else { Node::CommaList(elements) };
        Ok(node.optimize())
    }

    fn parse_expr(&mut self) -> Result<Node, ParseError> {
        let mut node = self.parse_term()?;
        while self.token.kind == TokenKind::Plus || self.token.kind == TokenKind::Minus {
            let op = if self.token.kind == TokenKind::Plus { BinaryOp::Add } else { BinaryOp::Sub };
            self.advance_token();
            let rhs = self.parse_term()?;
            node = Node::BinaryOp(op, Box::new(node), Box::new(rhs));
        }
        Ok(node)
    }

    fn parse_term(&mut self) -> Result<Node, ParseError> {
        let mut node = self.parse_factor()?;
        while self.token.kind == TokenKind::Asterisk || self.token.kind == TokenKind::Slash || self.token.kind == TokenKind::Percent {
            let op = match self.token.kind {
                TokenKind::Asterisk => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                TokenKind::Percent => BinaryOp::Mod,
                _ => unreachable!(),
            };
            self.advance_token();
            let rhs = self.parse_factor()?;
            node = Node::BinaryOp(op, Box::new(node), Box::new(rhs));
        }
        Ok(node)
    }

    fn parse_factor(&mut self) -> Result<Node, ParseError> {
        let mut node = self.parse_power()?;
        while self.token.kind == TokenKind::Caret {
            self.advance_token();
            let rhs = self.parse_power()?;
            node = Node::BinaryOp(BinaryOp::Pow, Box::new(node), Box::new(rhs));
        }
        Ok(node)
    }

    fn parse_power(&mut self) -> Result<Node, ParseError> {
        let sign = if self.token.kind == TokenKind::Plus || self.token.kind == TokenKind::Minus {
            let op = self.token.kind;
            self.advance_token();
            Some(op)
        } else {
            None
        };
        let mut node = self.parse_base()?;
        if let Some(op) = sign {
            let unary = if op == TokenKind::Plus { UnaryOp::Plus } else { UnaryOp::Minus };
            node = Node::UnaryOp(unary, Box::new(node));
        }
        Ok(node)
    }

    fn parse_base(&mut self) -> Result<Node, ParseError> {
        match self.token.kind {
            TokenKind::Number => {
                let value = self.token.value;
                self.advance_token();
                Ok(Node::Constant(value))
            }
            TokenKind::Identifier => {
                let name = self.token.text.clone();
                self.advance_token();
                if self.token.kind == TokenKind::LParen {
                    self.advance_token();
                    let args = self.parse_argument_list()?;
                    let function = lookup_function(&name, args.len())
                        .ok_or_else(|| self.error(&format!("unknown function '{}' with {} arguments", name, args.len())))?;
                    Ok(Node::Function(function, args))
                } else if let Some(function) = lookup_function(&name, 1) {
                    let arg = self.parse_power()?;
                    Ok(Node::Function(function, vec![arg]))
                } else if let Some(value) = lookup_constant(&name) {
                    Ok(Node::Constant(value))
                } else if let Some(index) = self.find_variable(&name) {
                    Ok(Node::Variable(index))
                } else {
                    Err(self.error(&format!("unknown symbol '{}'", name)))
                }
            }
            TokenKind::LParen => {
                self.advance_token();
                let node = self.parse_list()?;
                self.expect(TokenKind::RParen)?;
                Ok(node)
            }
            _ => Err(self.error("expected number, variable, function, or parenthesis")),
        }
    }

    fn parse_argument_list(&mut self) -> Result<Vec<Node>, ParseError> {
        if self.token.kind == TokenKind::RParen {
            self.advance_token();
            return Ok(Vec::new());
        }
        let mut args = vec![self.parse_expr()?];
        while self.token.kind == TokenKind::Comma {
            self.advance_token();
            args.push(self.parse_expr()?);
        }
        self.expect(TokenKind::RParen)?;
        Ok(args)
    }

    fn find_variable(&mut self, name: &str) -> Option<usize> {
        if let Some(index) = self.variable_names.iter().position(|candidate| candidate == name) {
            return Some(index);
        }
        if let Some(index) = self.lookup.iter().position(|variable| variable.name == name) {
            self.variable_names.push(name.to_string());
            self.variables.push(self.lookup[index].value);
            return Some(self.variables.len() - 1);
        }
        None
    }

    fn expect(&mut self, kind: TokenKind) -> Result<(), ParseError> {
        if self.token.kind == kind {
            self.advance_token();
            Ok(())
        } else {
            Err(self.error(&format!("expected {:?}", kind)))
        }
    }

    fn error(&self, message: &str) -> ParseError {
        ParseError { position: self.token.position, message: message.to_owned() }
    }

    fn advance_token(&mut self) {
        self.skip_whitespace();
        let position = self.position;
        let remainder = &self.input[self.position..];
        if remainder.is_empty() {
            self.token = Token { kind: TokenKind::End, text: String::new(), value: 0.0, position };
            return;
        }

        let mut chars = remainder.chars();
        let ch = chars.next().unwrap();

        let token = match ch {
            '+' => Token { kind: TokenKind::Plus, text: "+".into(), value: 0.0, position },
            '-' => Token { kind: TokenKind::Minus, text: "-".into(), value: 0.0, position },
            '*' => Token { kind: TokenKind::Asterisk, text: "*".into(), value: 0.0, position },
            '/' => Token { kind: TokenKind::Slash, text: "/".into(), value: 0.0, position },
            '%' => Token { kind: TokenKind::Percent, text: "%".into(), value: 0.0, position },
            '^' => Token { kind: TokenKind::Caret, text: "^".into(), value: 0.0, position },
            '(' => Token { kind: TokenKind::LParen, text: "(".into(), value: 0.0, position },
            ')' => Token { kind: TokenKind::RParen, text: ")".into(), value: 0.0, position },
            ',' => Token { kind: TokenKind::Comma, text: ",".into(), value: 0.0, position },
            '0'..='9' | '.' => self.read_number(position),
            'a'..='z' | 'A'..='Z' | '_' => self.read_identifier(position),
            _ => Token { kind: TokenKind::Unknown, text: ch.to_string(), value: 0.0, position },
        };

        self.token = token;
        self.position = position + self.token.text.len();
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.input[self.position..].chars().next() {
            if ch.is_whitespace() {
                self.position += ch.len_utf8();
            } else {
                break;
            }
        }
    }

    fn read_number(&self, start: usize) -> Token {
        let mut end = start;
        let chars = self.input[start..].chars();
        let mut text = String::new();
        let mut has_dot = false;
        let mut has_exp = false;
        let mut hex = false;

        for ch in chars {
            if text.is_empty() && ch == '0' {
                text.push(ch);
                end += 1;
                continue;
            }
            if text == "0" && (ch == 'x' || ch == 'X') {
                text.push(ch);
                hex = true;
                end += 1;
                continue;
            }
            if hex {
                if ch.is_ascii_hexdigit() {
                    text.push(ch);
                    end += 1;
                    continue;
                }
                break;
            }

            if ch == '.' {
                if has_dot || has_exp {
                    break;
                }
                has_dot = true;
                text.push(ch);
                end += 1;
            } else if ch == 'e' || ch == 'E' {
                if has_exp {
                    break;
                }
                has_exp = true;
                text.push(ch);
                end += 1;
            } else if ch == '+' || ch == '-' {
                if has_exp && text.ends_with('e') || text.ends_with('E') {
                    text.push(ch);
                    end += 1;
                } else {
                    break;
                }
            } else if ch.is_digit(10) {
                text.push(ch);
                end += 1;
            } else {
                break;
            }
        }

        let value = if hex {
            let parsed = i64::from_str_radix(text.trim_start_matches("0x").trim_start_matches("0X"), 16).unwrap_or(0);
            parsed as Value
        } else {
            Value::from_str(&text).unwrap_or(0.0)
        };

        Token { kind: TokenKind::Number, text, value, position: start }
    }

    fn read_identifier(&self, start: usize) -> Token {
        let mut end = start;
        let mut text = String::new();
        for ch in self.input[start..].chars() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                text.push(ch);
                end += ch.len_utf8();
            } else {
                break;
            }
        }
        Token { kind: TokenKind::Identifier, text, value: 0.0, position: start }
    }
}

fn lookup_constant(name: &str) -> Option<Value> {
    match name.to_lowercase().as_str() {
        "pi" => Some(std::f64::consts::PI),
        "e" => Some(std::f64::consts::E),
        _ => None,
    }
}

fn lookup_function(name: &str, arity: usize) -> Option<Function> {
    let normalized = name.to_lowercase();
    BUILTINS.iter().find_map(|entry| {
        if entry.name == normalized && entry.arity == arity {
            Some(entry.clone())
        } else {
            None
        }
    })
}

fn factorial(x: Value) -> Value {
    if x < 0.0 || x.fract() != 0.0 {
        return f64::NAN;
    }
    let mut result = 1.0;
    let mut value = x as u64;
    while value > 1 {
        result *= value as Value;
        value -= 1;
    }
    result
}

fn choose(n: Value, k: Value) -> Value {
    if n < 0.0 || k < 0.0 || n.fract() != 0.0 || k.fract() != 0.0 {
        return f64::NAN;
    }
    let n = n as u64;
    let k = k as u64;
    if k > n {
        return 0.0;
    }
    let k = if k > n - k { n - k } else { k };
    let mut numer = 1.0;
    let mut denom = 1.0;
    for i in 1..=k {
        numer *= (n - k + i) as Value;
        denom *= i as Value;
    }
    numer / denom
}

fn permutations(n: Value, k: Value) -> Value {
    if n < 0.0 || k < 0.0 || n.fract() != 0.0 || k.fract() != 0.0 {
        return f64::NAN;
    }
    let n = n as u64;
    let k = k as u64;
    if k > n {
        return 0.0;
    }
    let mut result = 1.0;
    for i in 0..k {
        result *= (n - i) as Value;
    }
    result
}

static BUILTINS: &[Function] = &[
    Function { name: "abs", arity: 1, callback: |args| args[0].abs(), pure: true },
    Function { name: "acos", arity: 1, callback: |args| args[0].acos(), pure: true },
    Function { name: "asin", arity: 1, callback: |args| args[0].asin(), pure: true },
    Function { name: "atan", arity: 1, callback: |args| args[0].atan(), pure: true },
    Function { name: "atan2", arity: 2, callback: |args| args[0].atan2(args[1]), pure: true },
    Function { name: "ceil", arity: 1, callback: |args| args[0].ceil(), pure: true },
    Function { name: "cos", arity: 1, callback: |args| args[0].cos(), pure: true },
    Function { name: "cosh", arity: 1, callback: |args| args[0].cosh(), pure: true },
    Function { name: "exp", arity: 1, callback: |args| args[0].exp(), pure: true },
    Function { name: "fac", arity: 1, callback: |args| factorial(args[0]), pure: true },
    Function { name: "floor", arity: 1, callback: |args| args[0].floor(), pure: true },
    Function { name: "ln", arity: 1, callback: |args| args[0].ln(), pure: true },
    Function { name: "log", arity: 1, callback: |args| args[0].ln(), pure: true },
    Function { name: "log10", arity: 1, callback: |args| args[0].log10(), pure: true },
    Function { name: "ncr", arity: 2, callback: |args| choose(args[0], args[1]), pure: true },
    Function { name: "npr", arity: 2, callback: |args| permutations(args[0], args[1]), pure: true },
    Function { name: "pow", arity: 2, callback: |args| args[0].powf(args[1]), pure: true },
    Function { name: "sin", arity: 1, callback: |args| args[0].sin(), pure: true },
    Function { name: "sinh", arity: 1, callback: |args| args[0].sinh(), pure: true },
    Function { name: "sqrt", arity: 1, callback: |args| args[0].sqrt(), pure: true },
    Function { name: "tan", arity: 1, callback: |args| args[0].tan(), pure: true },
    Function { name: "tanh", arity: 1, callback: |args| args[0].tanh(), pure: true },
];
