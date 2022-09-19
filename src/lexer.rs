use crate::constants::{COMMENT, DIRECTIVES, OPERAND_SEPARATOR};
use regex::Regex;
use std::collections::HashMap;
#[derive(Debug, Clone)]
pub enum RegisterType {
    Address,
    Data,
    SP,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Register(RegisterType, String),
    Immediate(String),
    Indirect {
        offset: String,
        operand: Box<Operand>,
    },
    IndirectWithDisplacement {
        offset: String,
        operands: Vec<Operand>,
    },
    PostIndirect(Box<Operand>),
    PreIndirect(Box<Operand>),
    Address(String),
    Label(String),
    Other(String),
}

#[derive(Debug, Clone)]
pub enum Line {
    Label {
        name: String,
        args: Vec<ArgSeparated>,
    },
    Directive {
        args: Vec<String>,
    },
    Instruction {
        name: String,
        operands: Vec<Operand>,
        size: Size,
    },
    Comment {
        content: String,
    },
    Empty,
    Unknown,
}
#[derive(Debug)]
pub enum OperandKind {
    Register,
    Immediate,
    Indirect,
    IndirectDisplacement,
    PostIndirect,
    PreIndirect,
    Label,
    Address,
}
#[derive(Debug, Clone)]
pub enum Size {
    Byte,
    Word,
    Long,
    Unspecified,
    Unknown,
}
#[derive(Debug, Clone)]
pub enum LineKind {
    Label,
    Directive,
    Instruction { size: Size, name: String },
    Comment,
    Empty,
    Unknown,
}

#[derive(Debug, Clone)]
pub enum ArgSeparated {
    Comma(String),
    Space(String),
}
struct AsmRegex {
    directives_map: HashMap<String, bool>,
    register: Regex,
    immediate: Regex,
    indirect: Regex,
    indirect_displacement: Regex,
    post_indirect: Regex,
    address: Regex,
    pre_indirect: Regex,
    label_line: Regex,
    comment_line: Regex,
}
impl AsmRegex {
    pub fn new() -> Self {
        let directives = DIRECTIVES
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        let directives_hash_map = directives
            .iter()
            .map(|x| (x.to_string(), true))
            .collect::<HashMap<String, bool>>();
        AsmRegex {
            directives_map: directives_hash_map,
            register: Regex::new(r"^((d|a)\d|sp)$").unwrap(),
            immediate: Regex::new(r"^\#\S+$").unwrap(),
            indirect: Regex::new(r"^\S*\(((d|a)\d|sp)\)$").unwrap(),
            indirect_displacement: Regex::new(r"^((.+,)+.+)$").unwrap(),
            post_indirect: Regex::new(r"^\(\S+\)\+$").unwrap(),
            pre_indirect: Regex::new(r"^-\(\S+\)$").unwrap(),
            address: Regex::new(r"^\$\S*$").unwrap(),
            label_line: Regex::new(r"^\S+:.*").unwrap(),
            comment_line: Regex::new(r"^;.*").unwrap(),
        }
    }
    pub fn get_operand_kind(&self, operand: &String) -> OperandKind {
        match operand {
            _ if self.register.is_match(operand) => OperandKind::Register,
            _ if self.post_indirect.is_match(operand) => OperandKind::PostIndirect,
            _ if self.pre_indirect.is_match(operand) => OperandKind::PreIndirect,
            _ if self.immediate.is_match(operand) => OperandKind::Immediate,
            _ if self.indirect.is_match(operand) => OperandKind::Indirect,
            _ if self.indirect_displacement.is_match(operand) => OperandKind::IndirectDisplacement,
            _ if self.address.is_match(operand) => OperandKind::Address,
            _ => OperandKind::Label,
        }
    }
    pub fn split_instruction_and_size(&self, instruction: &String) -> (String, Size) {
        let instruction = instruction.to_string();
        let split = instruction.split('.').collect::<Vec<&str>>();
        match split[..] {
            [instruction] => (instruction.to_string(), Size::Unspecified),
            [instruction, size] => {
                let size = match size {
                    "b" => Size::Byte,
                    "w" => Size::Word,
                    "l" => Size::Long,
                    _ => Size::Unknown,
                };
                (instruction.to_string(), size)
            }
            _ => (instruction, Size::Unspecified),
        }
    }
    pub fn split_into_operand_args(&self, line: &str) -> Vec<String> {
        //split at line except if in parenthesis
        let mut args = vec![];
        let mut current_arg = String::new();
        //TODO maybe make it handle multiple parenthesis, shouldn't be needed for now
        let mut in_parenthesis = false;

        for c in line.chars() {
            match c {
                '(' => {
                    in_parenthesis = true;
                    current_arg.push(c);
                }
                ')' => {
                    in_parenthesis = false;
                    current_arg.push(c);
                }
                OPERAND_SEPARATOR => {
                    if in_parenthesis {
                        current_arg.push(c);
                    } else {
                        args.push(current_arg.trim().to_string());
                        current_arg = String::new();
                    }
                }
                _ => current_arg.push(c),
            }
        }
        args.push(current_arg.trim().to_string());
        args
    }
    pub fn split_into_separated_args(&self, line: &str) -> Vec<ArgSeparated> {
        let mut args = vec![];
        let mut current_arg = String::new();
        //TODO maybe count how many paranthesis it's in
        let mut in_parenthesis = false;
        let mut last_char = ' ';
        let mut last_separator = ' ';
        //TODO fix this, it doesn't work correctly but works in the context of the language
        for c in line.chars() {
            match c {
                '(' => {
                    in_parenthesis = true;
                    current_arg.push(c);
                }
                ')' => {
                    in_parenthesis = false;
                    current_arg.push(c);
                }
                ',' => {
                    if in_parenthesis {
                        current_arg.push(c);
                    } else {
                        args.push(ArgSeparated::Comma(current_arg.trim().to_string()));
                        current_arg = String::new();
                        last_separator = c;
                    }
                }
                ' ' => {
                    if last_char == ',' {
                        continue;
                    }
                    if in_parenthesis {
                        current_arg.push(c);
                    } else {
                        if current_arg == "" {
                            continue;
                        }
                        args.push(ArgSeparated::Space(current_arg.trim().to_string()));
                        current_arg = String::new();
                        last_separator = c;
                    }
                }
                _ => {
                    current_arg.push(c);
                }
            }
            last_char = c;
        }
        match current_arg.trim() {
            "" => args,
            _ => match last_separator {
                ',' => {
                    args.push(ArgSeparated::Comma(current_arg.trim().to_string()));
                    args
                }
                _ => {
                    args.push(ArgSeparated::Space(current_arg.trim().to_string()));
                    args
                }
            },
        }
    }
    pub fn split_at_spaces(&self, line: &str) -> Vec<String> {
        line.split(' ')
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    }
    pub fn get_line_kind(&self, line: &String) -> LineKind {
        let line = line.trim();
        let args = line
            .split_whitespace()
            .map(|x| x.trim().to_string())
            .collect::<Vec<String>>();
        match args[..] {
            [] => LineKind::Empty,
            _ if self.comment_line.is_match(line) => LineKind::Comment,
            _ if self.label_line.is_match(line) => LineKind::Label,
            _ if args.iter().any(|a| self.directives_map.contains_key(a)) => LineKind::Directive,
            [_, _, ..] => {
                let (instruction, size) = self.split_instruction_and_size(&args[0]);
                LineKind::Instruction {
                    size,
                    name: instruction,
                }
            }
            _ => LineKind::Unknown,
        }
    }
}

struct EquValue {
    pub name: String,
    pub replacement: String,
}
#[derive(Debug, Clone)]
pub struct ParsedLine {
    pub parsed: Line,
    pub line: String,
}
pub struct Lexer {
    lines: Vec<ParsedLine>,
    regex: AsmRegex,
}
impl Lexer {
    pub fn new() -> Self {
        Lexer {
            lines: Vec::new(),
            regex: AsmRegex::new(),
        }
    }
    pub fn apply_equ(&self, lines: Vec<String>) -> Vec<String> {
        let mut equs: Vec<EquValue> = Vec::new();
        let mut equ_map_indexes: HashMap<usize, bool> = HashMap::new();
        const EQU: &'static str = "equ";
        lines
            .iter()
            .map(|line| self.regex.split_at_spaces(line))
            .enumerate()
            .for_each(|(index, args)| {
                if args.len() >= 3 && args[1].eq(EQU) {
                    equs.push(EquValue {
                        name: args[0].to_string(),
                        replacement: args[2].to_string(),
                    });
                    equ_map_indexes.insert(index, true);
                }
            });

        lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                if equ_map_indexes.contains_key(&i) {
                    return line.to_string();
                }
                let split_at_comments = line.split(COMMENT).collect::<Vec<&str>>();
                match split_at_comments[..] {
                    [code, comment, ..] => {
                        let mut new_line = code.to_string();
                        for equ in equs.iter() {
                            if new_line.contains(&equ.name) {
                                new_line =
                                    new_line.replace(equ.name.as_str(), equ.replacement.as_str());
                            }
                        }
                        new_line + ";" + comment
                    }
                    _ => line.to_string(),
                }
            })
            .collect::<Vec<String>>()
    }
    pub fn parse_operands(&self, operands: Vec<String>) -> Vec<Operand> {
        operands
            .iter()
            .take_while(|o| !o.contains(COMMENT))
            .map(|o| self.parse_operand(o))
            .collect()
    }
    pub fn parse_operand(&self, operand: &String) -> Operand {
        let operand = operand.to_string();
        match self.regex.get_operand_kind(&operand) {
            OperandKind::Immediate => Operand::Immediate(operand),
            OperandKind::Register => {
                let register_type = match operand.chars().nth(0).unwrap() {
                    'd' => RegisterType::Data,
                    'a' => RegisterType::Address,
                    's' => RegisterType::SP,
                    _ => panic!("Invalid register type '{}'", operand),
                };
                Operand::Register(register_type, operand)
            }
            OperandKind::IndirectDisplacement | OperandKind::Indirect => {
                let split = operand.split('(').collect::<Vec<&str>>();
                match split[..] {
                    [displacement, args] => {
                        let args = args.replace(")", "");
                        let args = self.regex.split_into_operand_args(args.as_str());
                        let offset = displacement.trim().to_string();
                        let operands = self.parse_operands(args);
                        match &operands[..] {
                            [operand] => Operand::Indirect {
                                offset,
                                operand: Box::new(operand.clone()),
                            },
                            [_, ..] => Operand::IndirectWithDisplacement { offset, operands },
                            _ => panic!("Invalid indirect operand '{}'", operand),
                        }
                    }
                    _ => Operand::Other(operand),
                }
            }
            OperandKind::Address => Operand::Address(operand),
            OperandKind::PostIndirect => {
                let parsed_operand = operand.replace("(", "").replace(")+", "");
                let arg = self.parse_operand(&parsed_operand);
                Operand::PostIndirect(Box::new(arg))
            },
            OperandKind::PreIndirect => {
                let parsed_operand = operand.replace("-(", "").replace(")", "");
                let arg = self.parse_operand(&parsed_operand);
                Operand::PreIndirect(Box::new(arg))
            },
            OperandKind::Label => Operand::Label(operand),

        }
    }
    pub fn lex(&mut self, code: String) {
        let lines = code.lines().map(String::from).collect::<Vec<String>>();
        let lines = self.apply_equ(lines);
        self.lines = lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let line = line.trim();
                let kind = self.regex.get_line_kind(&line.to_string().to_lowercase());
                let args = self.regex.split_at_spaces(line);
                let parsed_line = match kind {
                    LineKind::Instruction { size, name } => {
                        let operands = self
                            .regex
                            .split_into_operand_args(args[1..].join(" ").as_str());
                        let operands = self.parse_operands(operands);
                        Line::Instruction {
                            name,
                            size,
                            operands,
                        }
                    }
                    LineKind::Comment => Line::Comment {
                        content: line.to_string(),
                    },
                    LineKind::Label => {
                        let name = args.get(0).unwrap().replace(":", "").to_string();
                        let args = self
                            .regex
                            .split_into_separated_args(args[1..].join(" ").as_str());
                        Line::Label { name, args }
                    }
                    LineKind::Directive => Line::Directive { args },
                    LineKind::Empty => Line::Empty,
                    LineKind::Unknown => Line::Unknown,
                };
                ParsedLine {
                    parsed: parsed_line,
                    line: line.to_string(),
                }
            })
            .collect();
    }
    pub fn get_lines(&self) -> Vec<ParsedLine> {
        self.lines.clone()
    }
}
