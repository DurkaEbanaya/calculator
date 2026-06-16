use std::fmt::Write;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum UnaryOp {
    Percent,
    Sqrt,
    Square,
    Reciprocal,
}

pub struct Calculator {
    /// What is currently shown on the display.
    pub display: String,
    /// The left-hand side operand (accumulated value).
    pub lhs: f64,
    /// Pending binary operator.
    pub op: Option<BinOp>,
    /// Whether the next digit should replace the current display.
    pub fresh: bool,
    /// Whether an error (e.g. division by zero) is active.
    pub error: bool,
    /// Memory register.
    pub memory: f64,
    /// Small expression label shown above the main display.
    pub expression: String,
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}

impl Calculator {
    pub fn new() -> Self {
        Self {
            display: "0".to_string(),
            lhs: 0.0,
            op: None,
            fresh: true,
            error: false,
            memory: 0.0,
            expression: String::new(),
        }
    }

    pub fn input_digit(&mut self, d: u8) {
        if self.error {
            self.clear_all();
        }
        if self.fresh {
            self.display = d.to_string();
            self.fresh = false;
        } else if self.display == "0" {
            self.display = d.to_string();
        } else if self.display.len() < 16 {
            self.display.push(char::from_digit(d as u32, 10).unwrap());
        }
    }

    pub fn input_decimal(&mut self) {
        if self.error {
            self.clear_all();
        }
        if self.fresh {
            self.display = "0.".to_string();
            self.fresh = false;
            return;
        }
        if !self.display.contains('.') {
            self.display.push('.');
        }
    }

    pub fn negate(&mut self) {
        if self.error {
            return;
        }
        if self.display.starts_with('-') {
            self.display.remove(0);
        } else if self.display != "0" {
            self.display.insert(0, '-');
        }
    }

    pub fn backspace(&mut self) {
        if self.error || self.fresh {
            return;
        }
        if self.display.len() > 1 {
            self.display.pop();
        } else {
            self.display = "0".to_string();
            self.fresh = true;
        }
    }

    pub fn clear_entry(&mut self) {
        self.display = "0".to_string();
        self.fresh = true;
        if self.error {
            self.clear_all();
        }
    }

    pub fn clear_all(&mut self) {
        self.display = "0".to_string();
        self.lhs = 0.0;
        self.op = None;
        self.fresh = true;
        self.error = false;
        self.expression.clear();
    }

    pub fn set_op(&mut self, op: BinOp) {
        if self.error {
            return;
        }
        let value = parse_display(&self.display);
        if let Some(pending) = self.op {
            if !self.fresh {
                self.lhs = apply(pending, self.lhs, value);
                self.display = format_number(self.lhs);
            }
        } else {
            self.lhs = value;
        }
        self.op = Some(op);
        self.fresh = true;
        self.update_expression();
    }

    pub fn evaluate(&mut self) {
        if self.error {
            return;
        }
        if let Some(op) = self.op {
            let rhs = parse_display(&self.display);
            let result = apply(op, self.lhs, rhs);
            self.lhs = result;
            self.display = format_number(result);
            self.op = None;
            self.fresh = true;
            self.expression.clear();
        }
    }

    pub fn unary_op(&mut self, op: UnaryOp) {
        if self.error {
            return;
        }
        let value = parse_display(&self.display);
        let result = match op {
            UnaryOp::Percent => value / 100.0,
            UnaryOp::Sqrt => {
                if value < 0.0 {
                    self.set_error();
                    return;
                }
                value.sqrt()
            }
            UnaryOp::Square => value * value,
            UnaryOp::Reciprocal => {
                if value == 0.0 {
                    self.set_error();
                    return;
                }
                1.0 / value
            }
        };
        self.display = format_number(result);
        self.fresh = true;
    }

    fn set_error(&mut self) {
        self.error = true;
        self.display = "Error".to_string();
        self.expression.clear();
        self.op = None;
    }

    fn update_expression(&mut self) {
        self.expression.clear();
        let _ = write!(
            self.expression,
            "{} {}",
            format_number(self.lhs),
            op_symbol(self.op.unwrap())
        );
    }

    pub fn memory_clear(&mut self) {
        self.memory = 0.0;
    }

    pub fn memory_recall(&mut self) {
        if self.error {
            self.clear_all();
        }
        self.display = format_number(self.memory);
        self.fresh = true;
    }

    pub fn memory_add(&mut self) {
        self.memory += parse_display(&self.display);
    }

    pub fn memory_sub(&mut self) {
        self.memory -= parse_display(&self.display);
    }

    pub fn memory_store(&mut self) {
        self.memory = parse_display(&self.display);
    }
}

fn parse_display(s: &str) -> f64 {
    s.parse().unwrap_or(0.0)
}

fn apply(op: BinOp, lhs: f64, rhs: f64) -> f64 {
    match op {
        BinOp::Add => lhs + rhs,
        BinOp::Sub => lhs - rhs,
        BinOp::Mul => lhs * rhs,
        BinOp::Div => {
            if rhs == 0.0 {
                f64::NAN
            } else {
                lhs / rhs
            }
        }
    }
}

fn op_symbol(op: BinOp) -> &'static str {
    match op {
        BinOp::Add => "+",
        BinOp::Sub => "−",
        BinOp::Mul => "×",
        BinOp::Div => "÷",
    }
}

fn format_number(v: f64) -> String {
    if v.is_nan() || v.is_infinite() {
        return "Error".to_string();
    }
    // Drop trailing .0 when it is an integer, otherwise keep up to 10 decimals.
    let s = format!("{:.10}", v);
    let s = s.trim_end_matches('0').trim_end_matches('.').to_string();
    if s.is_empty() || s == "-" {
        "0".to_string()
    } else {
        s
    }
}
