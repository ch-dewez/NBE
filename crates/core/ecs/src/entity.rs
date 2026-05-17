pub type EntityId = usize;
pub type EntityVersion = usize;

#[derive(Clone, PartialEq, Eq, Copy, Hash)]
pub struct Entity{
    pub id: EntityId,
    pub version:EntityVersion 
}
