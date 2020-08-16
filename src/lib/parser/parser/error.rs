use failure::Fail;
use pest::error::Error as PestError;

use std::fmt;

use super::Rule;

#[derive(Debug, Fail)]
pub enum ParserError {
    /// Some syntax violation occured.
    InvalidSyntax(#[fail(reason)] PestError<Rule>),
    /// An undefined Label was referenced.
    UndefinedLabels(Vec<String>),
    /// More than 40 Labels have been used.
    TooManyLabels,
}

macro_rules! map {
    ( $error:expr; $( $($rule:expr),* => $str:expr );* ) => {
        {
            use pest::error::ErrorVariant as EV;

            fn match_until_found(e: &PestError<Rule>) -> String {
                match &e.variant {
                    EV::ParsingError{ negatives: _, positives } => {
                        $(
                            let mut contains_all = true;
                            let slice = [ $($rule,)* ];
                            for el in &slice {
                                if ! positives.contains(el) {
                                    contains_all = false;
                                    break;
                                }
                            }
                            if contains_all {
                                return format!("{}", $str);
                            }
                        )*
                        // If nothing matched, return a default string
                        let mut s = String::from("Expected ");
                        let positives = positives.clone();
                        if let Some(first_pos) = positives.first() {
                            s += &format!("{}", first_pos);
                        }
                        if positives.len() > 2 {
                            for pos in &positives[1..positives.len() - 1] {
                                s += &format!(", {}", pos.to_string());
                            }
                        }
                        if positives.len() > 1 {
                            let last_pos = positives.last().unwrap();
                            s += &format!(" or {}", last_pos.to_string());
                        }
                        s
                    },
                    EV::CustomError{ message } => return message.clone(),
                }
            }
            let mut e = $error;
            let s = match_until_found(&e);
            let variant = EV::CustomError { message: s };
            e.variant = variant;
            e
        }
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Rule::*;
        let s = match self {
            EOI => "nothing",
            eol => "end of line",
            colon => "':'",
            semicolon => ";",
            ws => "a tab or space",
            space => "tabs or spaces",
            comma => "','",
            sep_ip => "tabs or spaces",
            sep_pp => "', '",
            oparen => "'('",
            cparen => "')'",
            plus => "'+'",
            constant_bin => "a binary constant",
            constant_hex => "a hex constant",
            constant_dec => "a constant",
            constant => "a constant",
            word_bin => "a binary word",
            word_hex => "a hex word",
            word_dec => "a word",
            word => "a word",
            rest => "anything",
            raw_label => "a label",
            raw_stacksize => "16|32|48|64|NOSET",
            register => "a register",
            registerdi => "'(Rs+)'",
            registerddi => "'((Rs+))'",
            memory => "'(const)|(Rs)|(label)'",
            source => "'Rs|(Rs)|(Rs+)|((Rs+))|(adr)|const'",
            destination => "'Rs|(Rs)|(Rs+)|((Rs+))|(adr)'",
            org => ".ORG",
            byte => ".BYTE",
            db => ".DB",
            equ => ".EQU",
            stacksize => "*STACKSIZE",
            clr => "CLR",
            add => "ADD",
            adc => "ADC",
            sub => "SUB",
            mul => "MUL",
            div => "DIV",
            inc => "INC",
            dec => "DEC",
            neg => "NEG",
            and => "AND",
            or => "OR",
            xor => "XOR",
            com => "COM",
            bits => "BITS",
            bitc => "BITC",
            tst => "TST",
            cmp => "CMP",
            bitt => "BITT",
            lsr => "LSR",
            asr => "ASR",
            lsl => "LSL",
            rrc => "RRC",
            rlc => "RLC",
            mov => "MOV",
            ld_const => "LD",
            ld_memory => "LD",
            st => "ST",
            push => "PUSH",
            pop => "POP",
            pushf => "PUSHF",
            popf => "POP",
            ldsp => "LDSP",
            ldfr => "LDFR",
            jmp => "JMP",
            jcs => "JCS",
            jcc => "JCC",
            jzs => "JZS",
            jzc => "JZC",
            jns => "JNS",
            jnc => "JNC",
            jr => "JR",
            call => "CALL",
            ret => "RET",
            reti => "RETI",
            stop => "STOP",
            nop => "NOP",
            ei => "EI",
            di => "DI",
            instruction => "any instruction",
            comment => "a comment",
            label => "a label definition",
            header => "'#! mrasm'",
            line => "a comment, a label definition, any instruction",
            file => "an asm program",
        };
        write!(f, "{}", s)
    }
}

impl From<PestError<Rule>> for ParserError {
    fn from(e: PestError<Rule>) -> Self {
        use Rule::*;
        // TODO: More of these helpful messages!
        let e = map! { e;
            header => "All source files have to begin with '#! mrasm', followed by a newline";
            oparen => "Expected an openining parenthesis '('";
            sep_pp => "Parameter need to be seperated like this: ', '";
            cparen => "Expected a closing parenthesis ')'";
            source => "Expected a general source (i.e. 'R0', '(R1+)', '((R0+))', '(LABEL)', '0x34', etc)";
            register => "Expected a register ('R0' - 'R3')";
            constant => "Expected a number between 0 and 255. (i.e. '0xF0', '0b110', '13')";
            constant_bin => "Expected a binary number between 0 and 255. (i.e. '0b110')";
            EOI, instruction, comment, label => "Typo in instruction? Or missing colon after label?";
            EOI, eol, semicolon, ws => "Expected comment or end of line. Too many arguments?"

        };
        ParserError::InvalidSyntax(e)
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParserError::InvalidSyntax(inner) => write!(f, "{}", inner),
            ParserError::UndefinedLabels(labels) => {
                write!(f, "Undefined references! These labels are undefined:\n")?;
                for label in labels {
                    write!(f, "\t- {}", label)?;
                }
                Ok(())
            }
            ParserError::TooManyLabels => write!(
                f,
                "More than 40 Labels have been used. 'mcontrol' can't handle this!"
            ),
        }
    }
}
