use std::io::Write;


//pub trait DotLabels<A, T>{
//    fn dot_labels_from_container<F, S1, S2, W>(&self, writer: &mut W, dot_options: S1, f: F)
//        where
//            S1: AsRef<str>,
//            S2: AsRef<str>,
//            F: Fn(&A, usize) -> S2,
//            W: Write;
//
//    fn dot_labels_from_contained<F, S1, S2, W>(&self, writer: &mut W, dot_options: S1, f: F)
//        where
//            W: Write,
//            S1: AsRef<str>,
//            S2: AsRef<str>,
//            F: Fn(&T, usize) -> S2;
//}

/// # Trait, which enables you to write a dot file
/// * Dot files can be used to visualize a graph
/// * To visualize, you can use something like
/// ```dot
/// twopi dotfile.dot -Tpdf > dotfile.pdf
/// circo dotfile.dot -Tpdf > dotfile.pdf
/// ```
/// You can also try some of the other [roadmaps](https://www.graphviz.org/).
pub trait Dot {

    /// * use function to create labels depending on the index
    /// * for valid `dot_options` use `dot_options!` macro and take a look at module `dot_constants`
    fn dot_from_indices<F, W, S1, S2>(&self, writer: &mut W, dot_options: S1, f: F) -> Result<(), std::io::Error>
        where
            S1: AsRef<str>,
            S2: AsRef<str>,
            W: Write,
            F: FnMut(usize) -> S2;

    /// * use index as labels for the nodes
    /// * default implementation uses `dot_from_indices`
    fn dot_with_indices<S, W>(&self, dot_options: S, writer: &mut W) -> Result<(), std::io::Error>
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

    /// * create dot file with empty labels
    /// * default implementation uses `dot_from_indices`
    fn dot<S, W>(&self, dot_options: S, writer: &mut W) -> Result<(), std::io::Error>
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
}
