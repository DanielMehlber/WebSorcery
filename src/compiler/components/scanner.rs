use std::{collections::LinkedList};

use crate::{compiler::{CompilationError, CompilationState, model::Template}, strings::{sourceview::SourceView, stringview::StringView}};
#[derive(Debug)]
enum FunctionName {
	IMPORT,
	INSERT,
	GET,
	IF,
	END
}
#[derive(Debug)]
enum KeyWordName {
	FROM,
	AS,
	TO,
	PARAMS
}
#[derive(Debug)]
struct InstructionToken<'this> {
	pub words: LinkedList<Token<'this>>
} impl<'this> InstructionToken<'this> {
	pub fn new(list: LinkedList<Token<'this>>) -> Self {
		Self{
			words: list
		}
	}
}

#[derive(Debug)]
enum TokenType<'this> {
	Function(FunctionName),
	KeyWord(KeyWordName),
	StringLiteral(StringView<'this>),
	Identifier,
	Instruction(InstructionToken<'this>),
	PlainText
} 

impl<'this> TokenType<'this> {
	pub fn from_instruction_word<'call>(view: &'call str) -> TokenType<'this> {
		match view {
			// Functions
			"IMPORT" => TokenType::Function(FunctionName::IMPORT),
			"INSERT" => TokenType::Function(FunctionName::INSERT),
			"GET" => TokenType::Function(FunctionName::GET),
			"IF" => TokenType::Function(FunctionName::IF),
			"END" => TokenType::Function(FunctionName::END),

			// Keyword
			"AS" => TokenType::KeyWord(KeyWordName::AS),
			"FROM" => TokenType::KeyWord(KeyWordName::FROM),
			"PARAMS" => TokenType::KeyWord(KeyWordName::PARAMS),
			"TO" => TokenType::KeyWord(KeyWordName::TO),
			// Identifier
			_ => TokenType::Identifier
		}
	}
}

#[derive(Debug)]
struct Token<'this> {
	pub view: StringView<'this>,
	ttype: TokenType<'this>
}
impl<'this> Token<'this> {

	pub fn word_from_source<'call>(view: &'call StringView<'this>) -> Token<'this> {
		Token {
			view: view.clone(),
			ttype: TokenType::from_instruction_word(view.cut())
		}
	}
}

#[derive(Debug)]
pub struct TokenModel<'this> {
	tokenstream: LinkedList<Token<'this>>
}
impl<'this> TokenModel<'this> {
	pub fn new() -> TokenModel<'this> {
		Self {
			tokenstream: LinkedList::new()
		}
	}

	pub fn append(&mut self, tok: Token<'this>) {
		self.tokenstream.push_back(tok);
	}
}


enum WordBegin {
	Identifier(usize), 
	StringLiteral(usize), 
	InstructionEnd(usize)
}

pub struct InputComponent<'this> {
	template: Template<'this>,
	pub tmodel: TokenModel<'this>,
	state: CompilationState<'this>
} 

impl<'this> InputComponent<'this> {
	pub fn new(id: String) -> InputComponent<'this> {
		Self {
			template: Template::new(id),
			tmodel: TokenModel::new(),
			state: CompilationState::new()
		}
	}

	pub fn is_ok(&self) -> bool {
		self.state.is_ok()
	}

	pub fn get_error(&self) -> Option<&CompilationError> {
		self.state.get_error()
	}

	/**
	 * Converts source in tmodel
	 */
	pub fn scan<'call>(&mut self, mut source: SourceView<'this>) {
		let mut last_instruction_end_index = source.cursor;
		loop {
			match InputComponent::find_next_instruction_start(&mut source) {
				Some(instruction_start_index) => {
					// Append plain text token
					if instruction_start_index > (last_instruction_end_index + 1) as usize{
						self.tmodel.append(
							Token {
								ttype: TokenType::PlainText,
								view: source.view.substring((last_instruction_end_index + 1) as usize, instruction_start_index - 1).unwrap()
							}	
						);
					}
					match InputComponent::find_next_instruction_end(&mut source) {
						Ok(instruction_end_index) => {
							
							let instruction_view = source.clone_ranged(instruction_start_index, instruction_end_index);
							//																		    skip '{' ^                   
							match InputComponent::scan_instruction(instruction_view) {
								Ok(tok) => {
									self.tmodel.append(tok);
									last_instruction_end_index = instruction_end_index as i64;
								}
								Err(err) => {
									self.state.error(err);
									return;
								}
							}
						},

						Err(err) => {
							self.state.error(err);
							return;
						}
					}
				}

				None => {
					let total_end_index = source.view.original.chars().count() - 1;
					if total_end_index > (last_instruction_end_index + 1) as usize{
						self.tmodel.append(
							Token {
								ttype: TokenType::PlainText,
								view: source.view.substring((last_instruction_end_index + 1) as usize, total_end_index).unwrap()
							}	
						);
					}
					return;
				}
			}
			
		}
	}


	/// # Finds next `{` in source.
	/// ## Works
	/// Searches for next `{` character and returns its index.\
	/// A `{` under the cursor by the time this function is called will be skipped.
	/// ## Options
	/// * If `{` is found its index will be returned\
	/// * If source ends without any `{` then `None` will be returned
	fn find_next_instruction_start<'call>(source: &mut SourceView<'this>) -> Option<usize> {
		for c in source.into_iter() {
			if c == '{' {
				return Some(source.cursor as usize);
			}
		}

		None
	}

	/// # Finds next `}` in source.
	/// ## Works
	/// Searches for next `}` character and returns its index.\
	/// A `}` under the cursor by the time this function is called will be skipped.
	/// ## Exceptions
	/// This function-call means that an instruction is open, so it must be closed before the source ends.\
	/// If in any case the source end before a `}` has been found, an `CompilationError` will be thrown.
	fn find_next_instruction_end<'call>(source: &mut SourceView<'this>) -> Result<usize, CompilationError<'this>> {
		// Construction is case of error
		let instruction_start_position = source.position.clone();
		let instruction_start_view = source.view.substring(source.cursor as usize, source.cursor as usize + 1).unwrap();
		for c in source.into_iter() {
			if c == '}' {
				return Ok(source.cursor as usize)
			}
		}

		Err(
			CompilationError {
				name: "ERR_UNCLOSED_INSTRUCTION".to_string(),
				desc: "instruction start does not have an instruction end".to_string(),
				position: Some(instruction_start_position),
				view: Some(instruction_start_view),
				cause: "unwanted instruction start in plain text or missing close".to_string(),
				suggestion: "insert '}' or remove unwanted instruction start".to_string()
			}
		)
	}

	/// # Finds next WORD begin
	/// This could be an **identifier**, a **string literal** or an **instruction end**\
	/// Type and index will be returned (inclusive)
	/// ## Exceptions
	/// * invalid chars are not allowed
	/// * unexpected end of source
	fn find_instruction_word_begin(source: &mut SourceView<'this>) -> Result<WordBegin, CompilationError<'this>> {
		
		// Test if currently on instruction end
		match source.current() {
			Some(c) => if c == '}' {
				return Ok(WordBegin::InstructionEnd(source.cursor as usize));
			}

			None => {}
		}

		for c in source.into_iter() {
			match c {
				// Skip those chars
				' '|'\t'|'\n' => {},
				// valid WORD start chars
				'a'..='z'|'A'..='Z'|'_' => return Ok(WordBegin::Identifier(source.cursor as usize)),
				// detected instruction end
				'}' => return Ok(WordBegin::InstructionEnd(source.cursor as usize)),
				// detected string literal
				'\'' => return Ok(WordBegin::StringLiteral(source.cursor as usize)),
				// invalid chars
				_ => return Err(
					CompilationError {
						name: "ERR_INVALID_WORD_BEGIN".to_string(),
						desc: "cannot scan word in instruction".to_string(),
						cause: format!("invalid character '{}' found at begin of word", c),
						suggestion: format!("remove character '{}' from word beginning. Such chars are not allowed there", c),
						position: Some(source.position.clone()),
						view: Some(source.view.substring(source.cursor as usize, source.cursor as usize+1).unwrap())
					}
				)
			}
		}

		Err(
			CompilationError {
				name: "ERR_UNEXPECTED_SOURCE_END".to_string(),
				desc: "cannot scan for next word".to_string(),
				cause: "source ends mid instruction".to_string(),
				suggestion: "end instruction before source end".to_string(),
				position: Some(source.position.clone()),
				view: None
			}
		)
	}

	/// # Finds next WORD end
	/// This could be a **whitespace**, **tab**, **newline** or `}` (exclusive)
	/// ## In case of `}`
	/// If `}` is detected this method will return the index before.\
	/// In case: `{... WORD}`
	/// Index of `WOR[D]` will be returned, but cursor is over `}` so 
	/// the end of instruction `}` must be checked directly after this method, **not** at the follwing char.
	/// 
	fn find_identifier_end(source: &mut SourceView<'this>) -> Result<usize, CompilationError<'this>> {
		for c in source.into_iter() {
			match c {
				' '|'\t'|'\n'|'}' => return Ok(source.cursor as usize - 1),
				'a'..='z'|'A'..='Z'|'.'|'-'|'_' => {},
				_ => return Err(
					CompilationError {
						name: "ERR_INVALID_CHAR_IN_WORD".to_string(),
						desc: "cannot scan word in instruction".to_string(),
						cause: format!("invalid character '{}' found in word", c),
						suggestion: format!("remove character '{}' from word. Such chars are not allowed there", c),
						position: Some(source.position.clone()),
						view: Some(source.view.substring(source.cursor as usize, source.cursor as usize+1).unwrap())
					}
				)
			}
		}

		Err(
			CompilationError {
				name: "ERR_UNEXPECTED_SOURCE_END".to_string(),
				desc: "cannot scan word".to_string(),
				cause: "source ends mid word".to_string(),
				suggestion: "end instruction before source end".to_string(),
				position: Some(source.position.clone()),
				view: None
			}
		)
	}

	/// # Finds next string literal end `'`
	/// and will return its index (inclusive)
	/// ## Exceptions
	/// * unexpected end of source
	fn find_string_literal_end(source: &mut SourceView<'this>) -> Result<usize, CompilationError<'this>> {
		for c in source.into_iter() {
			match c {
				'\'' => return Ok(source.cursor as usize),
				_ => {}
			}
		}

		Err(
			CompilationError {
				name: "ERR_UNEXPECTED_STRING_LITERAL_END".to_string(),
				desc: "cannot scan string literal".to_string(),
				cause: "source ends mid literal".to_string(),
				suggestion: "end literal before source end".to_string(),
				position: Some(source.position.clone()),
				view: None
			}
		)
	}

	/// # Scans instruction to generate token
	/// * `source` - contains source inbetween {...} including '{' and '}'
	fn scan_instruction<'call>(mut source: SourceView<'this>) -> Result<Token<'this>, CompilationError<'this>> {
		let mut wordlist: LinkedList<Token> = LinkedList::new();
		// Skip '{'
		source.next();
		loop {
			match InputComponent::find_instruction_word_begin(&mut source)? {
				// End of instruction => return collected words
				WordBegin::InstructionEnd(start) => {
					return Ok(Token{
						ttype: TokenType::Instruction(InstructionToken {
							words: wordlist
						}),
						view: source.view
					})
				},
				WordBegin::StringLiteral(start) => {
					let string_literal_end = InputComponent::find_string_literal_end(&mut source)?;
					wordlist.push_back(
						Token {
							ttype: TokenType::StringLiteral(source.view.substring(start+1, string_literal_end-1).unwrap()),
							view: source.view.substring(start, string_literal_end).unwrap()
						}
					);
				},
				WordBegin::Identifier(start) => {
					let identifier_end = InputComponent::find_identifier_end(&mut source)?;
					wordlist.push_back(
						Token {
							ttype: TokenType::Identifier,
							view: source.view.substring(start, identifier_end).unwrap()
						}
					);
				}	
			}
		}
	}

	


}

#[cfg(test)]
	mod tests {
    use crate::strings::sourceview::SourceView;
    use super::{InputComponent, WordBegin};

	#[test]
	fn find_next_instruction_end() {
		let mut source = SourceView::from_string("hallo{instruction}hallo");
		source.next();

		match InputComponent::find_next_instruction_end(&mut source) {
			Ok(i) => assert_eq!(i, 17),
			Err(_) => panic!()
		}

		// No instruction end left, so error should be returned
		match InputComponent::find_next_instruction_end(&mut source) {
			Ok(_) => panic!(),
			Err(_) => {}
		}
	}

	#[test]
	fn find_next_instruction_start() {
		let mut source = SourceView::from_string("hallo{startishere{");

		match InputComponent::find_next_instruction_start(&mut source) {
			Some(i) => assert_eq!(i, 5),
			None => panic!()
		}

		match InputComponent::find_next_instruction_start(&mut source) {
			Some(i) => assert_eq!(i, 17),
			None => panic!()
		}

		// No instruction start left, so None should be returned
		match InputComponent::find_next_instruction_start(&mut source) {
			Some(_) => panic!(),
			None => {}
		}
	}

	#[test]
	fn find_instruction_word_begin() {
		let mut source = SourceView::from_string("W ' .WORD");

		match InputComponent::find_instruction_word_begin(&mut source) {
			Ok(opt) => match opt {
				WordBegin::Identifier(i) => assert_eq!(i, 0),
				_ => panic!()
			},

			Err(_) => panic!()
		};

		match InputComponent::find_instruction_word_begin(&mut source) {
			Ok(opt) => match opt {
				WordBegin::StringLiteral(i) => assert_eq!(2, i),
				_ => panic!()
			},

			Err(_) => panic!()
		}

		// Should be Err
		match InputComponent::find_instruction_word_begin(&mut source) {
			Ok(_) => panic!(),

			Err(_) => {}
		}

		source = SourceView::from_string("	  	");

		// Error because unexpected end of source
		match InputComponent::find_instruction_word_begin(&mut source) {
			Ok(_) => panic!(),
			Err(_) => {}
		}

	}

	#[test]
	fn find_identifier_end() {
		let mut source = SourceView::from_string("WORD ");
		match InputComponent::find_identifier_end(&mut source) {
			Ok(i) => assert_eq!(i, 3),
			Err(_) => panic!()
		}

		source = SourceView::from_string("W!ORD ");
		// Should fail becaue of invalid char '!'
		match InputComponent::find_identifier_end(&mut source) {
			Ok(_) => panic!(),
			Err(_) => {}
		}

		source= SourceView::from_string("WORD");
		// Should fail because of unexpected source end
		match InputComponent::find_identifier_end(&mut source) {
			Ok(_) => panic!(),
			Err(_) => {}
		}
	}

	#[test]
	fn find_string_literal_end() {
		let mut source = SourceView::from_string("hallo' ");

		match InputComponent::find_string_literal_end(&mut source) {
			Ok(i) => assert_eq!(i, 5),
			Err(_) => panic!()
		}

		// Error because of end of source
		match InputComponent::find_string_literal_end(&mut source) {
			Ok(_) => panic!(),
			Err(_) => {}
		}
	}
		
}