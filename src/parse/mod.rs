pub mod ast;

pub trait Parser<'i, T: 'i> {
    fn input(&mut self) -> &'i [T];
    fn index(&mut self) -> &mut usize;

    fn peek_one(&mut self) -> Option<&'i T> {
        self.input().get(*self.index())
    }

    fn peek_many(&mut self, count: usize) -> Option<&'i [T]> {
        let idx: usize = *self.index();

        if idx + count <= self.input().len() {
            Some(&self.input()[idx..idx + count])
        } else {
            None
        }
    }

    fn advance_one(&mut self) -> Option<&'i T> {
        let t = self.peek_one()?;
        *self.index() += 1;
        return Some(t);
    }

    fn advance_many(&mut self, count: usize) -> Option<&'i [T]> {
        let ts = self.peek_many(count)?;
        *self.index() += count;
        return Some(ts);
    }

    fn is_eof(&mut self) -> bool {
        *self.index() >= self.input().len()
    }

    fn take_while(&mut self, mut f: impl FnMut(&T) -> bool) -> &'i [T] {
        let start = *self.index();

        while let Some(t) = self.peek_one() {
            if f(t) {
                self.advance_one();
            } else {
                break;
            }
        }
        let end = *self.index();
        return &self.input()[start..end];
    }

    fn skip_trivia(&mut self)
    where
        T: HasTrivia,
    {
        while let Some(t) = self.peek_one() {
            if (*t).is_trivia() {
                self.advance_one();
            } else {
                break;
            }
        }
    }
}

pub trait HasTrivia {
    fn is_trivia(&self) -> bool;
}
