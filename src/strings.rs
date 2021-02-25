
pub mod stringview {
    use std::fmt;

    /**
     * StringView is used to operate on strings.
     */
    #[derive(PartialEq)]
    #[derive(Eq)]
    pub struct StringView<'this> {
        pub original: &'this str,
        pub from: usize,
        pub to: usize
    }

    impl<'this> Clone for StringView<'this> {
        fn clone(&self) -> StringView<'this> {
            Self {
                original: self.original,
                from: self.from,
                to: self.to
            }
        }

        fn clone_from(&mut self, source: &Self) {
            self.original = source.original;
            self.from = source.from;
            self.to = source.to;
        }
    }

    impl<'this> std::fmt::Debug for StringView<'this> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.show_slice(4))
        }
    }

    impl<'this> StringView<'this> {
        pub fn from_string(s: &'this str, from: usize, to: usize) -> StringView<'this> {
            StringView {
                original: s,
                from,
                to
            }
        }

        pub fn new(s: &'this str) -> StringView<'this> {
            if s == "" {
                StringView {
                    to: 0,
                    original: " ",
                    from: 0,
                }
            } else {
                StringView {
                    to: s.chars().count() - 1,
                    original: s,
                    from: 0,
                }
            }
            
        }

        pub fn substring<'call>(&self, from: usize, to: usize) -> Result<StringView<'this>, String> {
            let len = self.original.chars().count();
            let new_from = self.from + from;
            let new_to = self.from + to;
            if new_from >= len {
                Err(format!("Index out of bounds: string has length {} but 'from' is {}", len, new_from))
            } else if new_to >= len {
                Err(format!("Index out of bounds: string has length {} but 'to' is {}", len, new_to))
            } else if new_from > new_to {
                Err(format!("Indices incorrect: 'from' needs to be before 'to' but {} > {}", new_from, new_to))
            } else if new_to > self.to {
                Err(format!("substring tries to access string outside of bounds: {} > {}", new_to, self.to))
            } else {
                Ok(StringView {
                    original: self.original,
                    from: new_from,
                    to: new_to
                })
            }
        }

        pub fn show_slice(&self, margin: usize) -> String {
            let length = self.original.chars().count();
            let right = if self.to + margin > length {length} else {self.to + margin};
            let left = if self.from > margin {self.from-margin} else {0};
            format!("...{}[{}]{}...", &self.original[left..self.from], &self.original[self.from..=self.to], &self.original[self.to+1..right])
        }

        pub fn cut(&'this self) -> &'this str {
            &self.original[self.from..=self.to]
        }

        pub fn char_at(&self, index: usize) -> Option<char> {
            if self.from + index > self.to {
                None
            } else {
               self.original.chars().nth(self.from + index) 
            }
        }

        pub fn is_empty(&self) -> bool {
            self.to - self.from == 0
        }
    }


    #[cfg(test)]
    mod tests {
        use super::StringView;

        #[test]
        fn new() {
            let s = StringView::new("Hallo");

            assert_eq!(s.original, "Hallo");
            assert_eq!(s.from, 0);
            assert_eq!(s.to, "Hallo".chars().count()-1);
        }

        #[test]
        fn from_string() {
            let s = StringView::from_string("Hallo", 1, 3);
            
            assert_eq!(s.original, "Hallo");
            assert_eq!(s.from, 1);
            assert_eq!(s.to, 3);
        } 

        #[test]
        fn cut_full() {
            let s = "Hallo";
            let view = StringView::new(s);

            assert_eq!(s, view.cut());
        }

        #[test]
        fn equals() {
            let s1 = StringView::from_string("Hallo", 1, 3);
            let s2= StringView::from_string("Hallo", 1, 3);

            assert_eq!(s1, s2);
        }

        #[test]
        fn substring() {
            let string = "Hallo ich bin ein String";
            let stringview = StringView::new(string);
            let substringview1 = stringview.substring(5, 10).unwrap();
            let substring=&string[5..=10];

            assert_eq!(substringview1.cut(),  substring);
        }

    }

}

pub mod sourceview {

    use super::stringview::StringView;

    #[derive(Clone)]
    pub struct SourcePosition {
        column: usize,
        line: usize
    } impl SourcePosition {
        pub fn move_pos(&mut self, c: char) {
            match c {
                '\n' => {
                    self.line = self.line + 1;
                    self.column = 0;
                },
                _ => {
                    self.column = self.column + 1;
                } 
            }
        }
    }

    impl std::fmt::Display for SourcePosition {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt:: Result {
            write!(f, "{}:{}", self.line, self.column)
        }
    }

    pub struct SourceView<'t> {
        pub view: StringView<'t>,
        pub position: SourcePosition,
        pub cursor: i64,
        reached_end: bool
    } 

    impl<'t> Clone for SourceView<'t> {
        fn clone(&self) -> Self {
            Self {
                view: self.view.clone(),
                cursor: self.cursor,
                position: self.position.clone(),
                reached_end: self.reached_end
            }
        }

        fn clone_from(&mut self, source: &Self) {
            self.view = source.view.clone();
            self.cursor = source.cursor;
            self.position = source.position.clone();
        }
    }
    
    impl<'this> SourceView<'this> {

        pub fn from_string(source: &'this str) -> SourceView<'this> {
            Self {
                cursor: -1,
                position: SourcePosition {column: 0, line: 0},
                view: StringView::new(source),
                reached_end: false
            }
        }

        pub fn clone_ranged<'call>(&self, begin: usize, end: usize) -> SourceView<'this> {
            let subview = self.view.substring(begin, end);
            match subview {
                Ok(_view) => {
                    SourceView {
                        cursor: -1,
                        position: self.position.clone(),
                        view: _view,
                        reached_end: false
                    }
                },

                Err(s) => panic!(s)
            }
        }

        pub fn show_part(&self, from: usize, to: usize) -> String {
            match self.view.substring(from, to) {
                Ok(view) => view.show_slice(5),
                Err(err) => panic!(err)
            }
        }

        pub fn current(&self) -> Option<char> {
            if self.cursor < 0 {
                None
            } else {
                self.view.char_at(self.cursor as usize)
            }
        }

        pub fn reset(&mut self) {
            self.cursor = -1;
            self.reached_end = false;
        }
    }

    impl<'t> Iterator for SourceView<'t> {
        
        type Item = char;

        fn next(&mut self) -> Option<char> {
            self.cursor = self.cursor + 1;

            match self.view.char_at(self.cursor as usize) {
                Some(c) => {
                    self.position.move_pos(c);
                    Some(c)
                },

                None => {
                    if self.reached_end {
                        self.cursor = self.cursor - 1;
                    }
                    self.reached_end = true;
                    None
                }
            }

        }
    }

    #[cfg(test)]
    mod tests {
        use super::SourceView;

        #[test]
        fn next() {
            let mut source = SourceView::from_string("Hallo");
            assert_eq!(source.next().unwrap(), 'H');
            assert_eq!(source.next().unwrap(), 'a');
            assert_eq!(source.next().unwrap(), 'l');
            assert_eq!(source.next().unwrap(), 'l');
            assert_eq!(source.next().unwrap(), 'o');

            match source.next() {
                Some(_) => panic!("no character left. Sould be Option::None"),
                None => {}
            }
            
        }

        #[test]
        fn clone_ranged(){
            let source1 = SourceView::from_string("0123456789");
            let source2 = source1.clone_ranged(3, 6);
            let view = source1.view.substring(3, 6).unwrap();

            assert_eq!(view.cut(), source2.view.cut());
        }

        #[test]
        fn iterator() {
            let source = SourceView::from_string("Hallo");
            let mut counter = 0;
            for _ in source.into_iter() {
                counter = counter + 1;
            }

            assert_eq!(counter, 5);
        }
    }

}