pub trait VecLike<V> {
    fn to_vec(&self) -> Vec<V>;
    fn append(&mut self, other: &mut Vec<V>);
    fn push(&mut self, value: V);
    fn pop(&mut self) -> Option<V>;

    //fn drain<R>(&mut self, range: R) -> Drain<'_, T, A>

    fn clear(&mut self);
    fn resize_with<F>(&mut self, new_len: usize, f: F)
    where
        F: FnMut() -> V;
    fn resize(&mut self, new_len: usize, value: V);
    fn extend_from_slice(&mut self, other: &[V]);
    fn dedup(&mut self);

    // Following api can not implement due to the `malke_buffer` issue
    // fn first(&self) -> Option<&V>
    // fn first_mut(&mut self) -> Option<&mut T>
    // fn last(&self) -> Option<&T>
    // fn last_mut(&mut self) -> Option<&mut T>
    // fn get_mut<I>(&mut self, index: I) -> Option<&mut <I as SliceIndex<[T]>>::Output>

    fn swap(&mut self, a: usize, b: usize);

    fn reverse(&mut self);

    // fn windows(&self, size: usize) -> Windows<'_, V>;
    // fn chunks(&self, chunk_size: usize) -> Chunks<'_, V>;
    // fn chunks_mut(&mut self, chunk_size: usize) -> ChunksMut<'_, V>;
    // fn chunks_exact(&self, chunk_size: usize) -> ChunksExact<'_, V>;
    // fn chunks_exact_mut(&mut self, chunk_size: usize) -> ChunksExactMut<'_, V>;

    fn contains(&self, x: &V) -> bool;
    fn starts_with(&self, needle: &[V]) -> bool;
    fn ends_with(&self, needle: &[V]) -> bool;
    // fn strip_prefix<P>(&self, prefix: &P) -> Option<&[V]>;
    // fn strip_suffix<P>(&self, suffix: &P) -> Option<&[V]>;
    fn rotate_left(&mut self, mid: usize);
    fn rotate_right(&mut self, k: usize);
    fn fill_with<F>(&mut self, f: F)
    where
        F: FnMut() -> V;
    fn copy_from_slice(&mut self, src: &[V])
    where
        V: Copy;
    fn sort(&mut self)
    where
        V: Ord;
    fn truncate(&mut self, len: usize);
    // fn concat<Item>(&self) -> <[V] as Concat<Item>>::Output Item: ?Sized, [V]: Concat<Item>,
    // fn join<Separator>(&self, sep: Separator) -> <[T] as Join<Separator>>::Output
}
