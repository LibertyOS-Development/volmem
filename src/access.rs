pub trait Readable {}
pub trait Writable {}

#[derive(Debug, Copy, Clone)]
pub struct ReadWrite;
impl Readable for ReadWrite {}
impl Writable for ReadWrite {}

#[derive(Debug, Copy, Clone)]
pub struct ReadOnly;
impl Readable for ReadOnly {}

#[derive(Debug, Copy, Clone)]
pub struct WriteOnly;
impl Writable for WriteOnly {}
