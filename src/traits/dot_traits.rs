use std::io::Write;

/// # Trait, which enables you to write a dot file
/// * similar to `Dot` trait, but lables are generated differently
/// * Dot files can be used to visualize a graph
/// * To visualize, you can use something like
/// ```dot
/// twopi dotfile.dot -Tpdf > dotfile.pdf
/// circo dotfile.dot -Tpdf > dotfile.pdf
/// ```
/// You can also try some of the other [roadmaps](https://www.graphviz.org/).
pub trait DotExtra<T, A>{

    /// * create a dot representation
    /// * you can use the indices and the container `A` (usually stored at each
    ///     index) to create the lables
    fn dot_from_container_index<F, S1, S2, W>(&self, writer: W, dot_options: S1, f: F)
        -> Result<(), std::io::Error>
        where
            S1: AsRef<str>,
            S2: AsRef<str>,
            F: FnMut(usize, &A) -> S2,
            W: Write;

    /// * same as `self.dot_string_from_container_index` but returns String instead
    fn dot_string_from_container_index<F, S1, S2>(&self, dot_options: S1, f: F)
        -> String
        where
            S1: AsRef<str>,
            S2: AsRef<str>,
            F: FnMut(usize, &A) -> S2
    {
        let mut s = Vec::new();
        self.dot_from_container_index(
            &mut s,
            dot_options,
            f
        ).unwrap();
        String::from_utf8(s).unwrap()
    }

    /// * create a dot representation
    /// * you can use the information of the container `A` (usually stored at each
    ///     index) to create the lables
    fn dot_from_container<F, S1, S2, W>(&self, writer: W, dot_options: S1, mut f: F)
        -> Result<(), std::io::Error>
        where
            S1: AsRef<str>,
            S2: AsRef<str>,
            F: FnMut(&A) -> S2,
            W: Write
    {
        self.dot_from_container_index(
            writer,
            dot_options,
            |_, a| f(a)
        )
    }

    /// * same as `self.dot_from_container` but returns String instead
    fn dot_string_from_container<F, S1, S2>(&self, dot_options: S1, f: F)
        -> String
        where
            S1: AsRef<str>,
            S2: AsRef<str>,
            F: FnMut(&A) -> S2
    {
        let mut s = Vec::<u8>::new();
        self.dot_from_container(
            &mut s,
            dot_options,
            f
        ).unwrap();
        String::from_utf8(s).unwrap()
    }

    /// * create a dot representation
    /// * you can use the indices and `T` (usually something contained in `A` (see `dot_from_container`)
    ///   and stored at each vertex) to create the lables
    fn dot_from_contained_index<F, S1, S2, W>(&self, writer: W, dot_options: S1, f: F)
        -> Result<(), std::io::Error>
        where
            W: Write,
            S1: AsRef<str>,
            S2: AsRef<str>,
            F: FnMut(usize, &T) -> S2;

    /// * same as `self.dot_from_contained` but returns String instead
    fn dot_string_from_contained_index<F, S1, S2>(&self, dot_options: S1, f: F)
        -> String
        where
            S1: AsRef<str>,
            S2: AsRef<str>,
            F: FnMut(usize, &T) -> S2
    {
        let mut s = Vec::<u8>::new();
        self.dot_from_contained_index(
            &mut s,
            dot_options,
            f
        ).unwrap();
        String::from_utf8(s).unwrap()
    }

    /// * create a dot representation
    /// * you can use `T` (usually something contained in `A` (see `dot_from_container`)
    ///   and stored at each vertex) to create the lables
    fn dot_from_contained<F, S1, S2, W>(&self, writer: W, dot_options: S1, mut f: F)
        -> Result<(), std::io::Error>
        where
            W: Write,
            S1: AsRef<str>,
            S2: AsRef<str>,
            F: FnMut(&T) -> S2
    {
        self.dot_from_contained_index(
            writer,
            dot_options,
            |_, c| f(c)
        )
    }

    /// * same as `self.dot_from_contained` but returns String
    fn dot_string_from_contained<F, S1, S2>(&self, dot_options: S1, f: F)
        -> String
        where
            S1: AsRef<str>,
            S2: AsRef<str>,
            F: FnMut(&T) -> S2
    {
        let mut s = Vec::<u8>::new();
        self.dot_from_contained(
            &mut s,
            dot_options,
            f
        ).unwrap();
        String::from_utf8(s).unwrap()
    }
}

/// # Trait, which enables you to write a dot file
/// * Dot files can be used to visualize a graph
/// * To visualize, you can use something like
/// ```dot
/// twopi dotfile.dot -Tpdf > dotfile.pdf
/// circo dotfile.dot -Tpdf > dotfile.pdf
/// ```
/// You can also try some of the other [roadmaps](https://www.graphviz.org/).
pub trait Dot {

    /// * use function `f` to create labels depending on the index
    /// * for valid `dot_options` use `dot_options!` macro and take a look at module `dot_constants`
    fn dot_from_indices<F, W, S1, S2>(&self, writer: W, dot_options: S1, f: F)
        -> Result<(), std::io::Error>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
        W: Write,
        F: FnMut(usize) -> S2;

    /// * same as `self.dot_from_indices` but returns String instead
    fn dot_string_from_indices<F, S1, S2>(&self, dot_options: S1, f: F)
        -> String
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
        F: FnMut(usize) -> S2
    {
        let mut s = Vec::new();
        self.dot_from_indices(
            &mut s,
            dot_options,
            f
        ).unwrap();
        String::from_utf8(s).unwrap()
    }

    /// * use index as labels for the nodes
    /// * default implementation uses `dot_from_indices`
    fn dot_with_indices<S, W>(
        &self, writer: W,
        dot_options: S
    ) -> Result<(), std::io::Error>
    where
        S: AsRef<str>,
        W: Write
    {
        self.dot_from_indices(
            writer,
            dot_options,
            |index| {
                index.to_string()
            }
        )
    }

    /// * same as `self.dot_with_indices` but returns String instead
    fn dot_string_with_indices<S>(&self, dot_options: S) -> String
        where
            S: AsRef<str>
    {
        let mut s = Vec::<u8>::new();
        self.dot_with_indices(
            &mut s,
            dot_options
        ).unwrap();
        String::from_utf8(s).unwrap()
    }

    /// * create dot file with empty labels
    /// * default implementation uses `dot_from_indices`
    fn dot<S, W>(&self, writer: W, dot_options: S) -> Result<(), std::io::Error>
        where
            S: AsRef<str>,
            W: Write
    {
        self.dot_from_indices(
            writer,
            dot_options,
            |_| {
                ""
            }
        )
    }

    /// * same as `self.dot()`, but returns a String instead
    fn dot_string<S>(&self, dot_options: S) -> String
        where
            S: AsRef<str>
    {
        let mut s = Vec::<u8>::new();
        self.dot(&mut s, dot_options).unwrap();
        String::from_utf8(s).unwrap()
    }
}
