use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestId {
    pub value: Uuid,
}
