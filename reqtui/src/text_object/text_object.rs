use ropey::Rope;

#[derive(Debug, Clone, PartialEq)]
pub struct Readonly;
#[derive(Debug, Clone, PartialEq)]
pub struct Write;

#[derive(Debug, PartialEq, Clone)]
pub struct TextObject<State = Readonly> {
    content: Rope,
    state: std::marker::PhantomData<State>,
}

impl<State> Default for TextObject<State> {
    fn default() -> Self {
        let content = String::default();

        TextObject {
            content: Rope::from_str(&content),
            state: std::marker::PhantomData,
        }
    }
}

impl TextObject<Readonly> {
    pub fn from(content: &str) -> TextObject<Readonly> {
        let content = Rope::from_str(content);
        TextObject::<Readonly> {
            content,
            state: std::marker::PhantomData::<Readonly>,
        }
    }

    pub fn with_write(self) -> TextObject<Write> {
        TextObject::<Write> {
            content: self.content,
            state: std::marker::PhantomData,
        }
    }
}

impl<State> std::fmt::Display for TextObject<State> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.content.to_string())
    }
}
